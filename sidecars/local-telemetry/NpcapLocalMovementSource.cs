using System.Diagnostics;
using System.Net.NetworkInformation;
using System.Threading.Channels;
using PacketDotNet;
using SharpPcap;

namespace IslemapThienvyma.Telemetry;

public sealed class NpcapLocalMovementSource : ILocalMovementSource
{
    public const string DefaultGameProcessName = "TheIsleClient-Win64-Shipping";
    private static readonly TimeSpan ProcessPollInterval = TimeSpan.FromSeconds(1);
    private static readonly TimeSpan MinimumObservationInterval = TimeSpan.FromMilliseconds(25);
    private const int CaptureReadTimeoutMilliseconds = 25;

    private readonly string _processName;
    private readonly WindowsUdpPortOwnerResolver _portResolver;
    private readonly LocalMovementTracker _tracker;
    private readonly object _movementTrackerGate = new();
    private readonly CancellationTokenSource _disposeCancellation = new();
    private int _watchStarted;
    private int _disposed;

    public NpcapLocalMovementSource(
        string processName = DefaultGameProcessName,
        WindowsUdpPortOwnerResolver? portResolver = null,
        LocalMovementTracker? tracker = null)
    {
        _processName = processName;
        _portResolver = portResolver ?? new WindowsUdpPortOwnerResolver();
        _tracker = tracker ?? new LocalMovementTracker();
    }

    public async IAsyncEnumerable<LocalMovementObservation> WatchAsync(
        [System.Runtime.CompilerServices.EnumeratorCancellation]
        CancellationToken cancellationToken = default)
    {
        if (Interlocked.Exchange(ref _watchStarted, 1) != 0)
        {
            throw new InvalidOperationException("A local movement source can only be watched once.");
        }

        using var linkedCancellation = CancellationTokenSource.CreateLinkedTokenSource(
            cancellationToken,
            _disposeCancellation.Token);
        var channel = Channel.CreateBounded<LocalMovementObservation>(
            new BoundedChannelOptions(1)
            {
                FullMode = BoundedChannelFullMode.DropOldest,
                SingleReader = true,
                SingleWriter = false
            });
        var captureTask = RunAsync(channel.Writer, linkedCancellation.Token);

        await foreach (var observation in channel.Reader
                           .ReadAllAsync(linkedCancellation.Token)
                           .ConfigureAwait(false))
        {
            yield return observation;
            await Task.Delay(MinimumObservationInterval, linkedCancellation.Token)
                .ConfigureAwait(false);
        }

        await captureTask.ConfigureAwait(false);
    }

    public ValueTask DisposeAsync()
    {
        if (Interlocked.Exchange(ref _disposed, 1) != 0)
        {
            return ValueTask.CompletedTask;
        }

        _disposeCancellation.Cancel();
        _disposeCancellation.Dispose();
        return ValueTask.CompletedTask;
    }

    private async Task RunAsync(
        ChannelWriter<LocalMovementObservation> writer,
        CancellationToken cancellationToken)
    {
        int? trackedProcessId = null;
        try
        {
            while (!cancellationToken.IsCancellationRequested)
            {
                var processId = FindGameProcessId();
                var ports = processId is null
                    ? new HashSet<int>()
                    : _portResolver.GetOwnedPorts(processId.Value);
                if (processId is null || ports.Count == 0)
                {
                    if (processId is null && trackedProcessId is not null)
                    {
                        ResetTracker();
                        trackedProcessId = null;
                    }

                    await Task.Delay(ProcessPollInterval, cancellationToken).ConfigureAwait(false);
                    continue;
                }

                if (trackedProcessId != processId)
                {
                    ResetTracker();
                    trackedProcessId = processId;
                }

                await CaptureUntilEndpointChangesAsync(
                        processId.Value,
                        ports,
                        writer,
                        cancellationToken)
                    .ConfigureAwait(false);
            }
        }
        catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
        {
        }
        catch (Exception exception)
        {
            writer.TryComplete(exception);
            return;
        }

        writer.TryComplete();
    }

    private async Task CaptureUntilEndpointChangesAsync(
        int processId,
        IReadOnlySet<int> ports,
        ChannelWriter<LocalMovementObservation> writer,
        CancellationToken cancellationToken)
    {
        var rawPackets = new BoundedPacketIntake();
        var decoderTask = ProcessPacketQueueAsync(
            rawPackets,
            writer,
            cancellationToken);
        var devices = OpenCaptureDevices(ports, rawPackets);
        try
        {
            while (!cancellationToken.IsCancellationRequested)
            {
                await Task.Delay(ProcessPollInterval, cancellationToken).ConfigureAwait(false);
                if (FindGameProcessId() != processId
                    || !_portResolver.GetOwnedPorts(processId).SetEquals(ports))
                {
                    return;
                }
            }
        }
        finally
        {
            foreach (var capture in devices)
            {
                capture.Device.OnPacketArrival -= capture.Handler;
                try { capture.Device.StopCapture(); } catch { }
                try { capture.Device.Close(); } catch { }
            }

            rawPackets.Complete();
            try
            {
                await decoderTask.ConfigureAwait(false);
            }
            catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
            {
            }
        }
    }

    private IReadOnlyList<OpenedCapture> OpenCaptureDevices(
        IReadOnlySet<int> ports,
        BoundedPacketIntake rawPackets)
    {
        CaptureDeviceList devices;
        try
        {
            devices = CaptureDeviceList.Instance;
        }
        catch (Exception exception) when (
            exception is DllNotFoundException
                or TypeInitializationException
                or PcapException)
        {
            throw new LocalPacketCaptureUnavailableException(
                "Npcap is not installed or its packet-capture driver is unavailable.",
                exception);
        }

        var candidates = SelectActiveDevices(devices).ToArray();
        var opened = new List<OpenedCapture>();
        var filter = BuildCaptureFilter(ports);
        foreach (var device in candidates)
        {
            PacketArrivalEventHandler handler = (_, packetCapture) =>
                EnqueuePacket(packetCapture, ports, rawPackets);
            try
            {
                device.Open(DeviceModes.None, read_timeout: CaptureReadTimeoutMilliseconds);
                device.Filter = filter;
                device.OnPacketArrival += handler;
                device.StartCapture();
                opened.Add(new OpenedCapture(device, handler));
            }
            catch
            {
                device.OnPacketArrival -= handler;
                try { device.Close(); } catch { }
            }
        }

        if (opened.Count == 0)
        {
            throw new LocalPacketCaptureUnavailableException(
                "No active network adapter could be opened through Npcap.");
        }

        return opened;
    }

    private void EnqueuePacket(
        PacketCapture packetCapture,
        IReadOnlySet<int> ports,
        BoundedPacketIntake rawPackets)
    {
        try
        {
            var rawPacket = packetCapture.GetPacket();
            var packet = Packet.ParsePacket(rawPacket.LinkLayerType, rawPacket.Data);
            var ip = packet.Extract<IPPacket>();
            var udp = packet.Extract<UdpPacket>();
            if (udp is null)
            {
                return;
            }

            var payload = udp.PayloadData;
            if (payload is null || payload.Length == 0)
            {
                return;
            }

            var outbound = ports.Contains(udp.SourcePort);
            if (!outbound)
            {
                return;
            }

            _ = rawPackets.TryEnqueue(new CapturedUdpDatagram(
                DateTimeOffset.UtcNow,
                ip?.SourceAddress.ToString(),
                udp.SourcePort,
                ip?.DestinationAddress.ToString(),
                udp.DestinationPort,
                payload.ToArray(),
                Outbound: true));
        }
        catch
        {
            // Malformed or unrelated UDP traffic must not stop local tracking.
        }
    }

    private async Task ProcessPacketQueueAsync(
        BoundedPacketIntake rawPackets,
        ChannelWriter<LocalMovementObservation> writer,
        CancellationToken cancellationToken)
    {
        await foreach (var packet in rawPackets
                           .ReadAllAsync(cancellationToken)
                           .ConfigureAwait(false))
        {
            ProcessPacket(packet, writer);
        }
    }

    private void ProcessPacket(
        CapturedUdpDatagram packet,
        ChannelWriter<LocalMovementObservation> writer)
    {
        try
        {
            UnrealMovementCandidate movement;
            lock (_movementTrackerGate)
            {
                if (!_tracker.TryTrack(packet.Payload, packet.ObservedAt, out movement))
                {
                    return;
                }
            }

            var serverEndpoint = packet.DestinationAddress is null
                ? null
                : $"{packet.DestinationAddress}:{packet.DestinationPort}";
            writer.TryWrite(new LocalMovementObservation(
                packet.ObservedAt,
                movement,
                serverEndpoint));
        }
        catch
        {
            // One malformed datagram must not stop the ordered worker.
        }
    }

    private void ResetTracker()
    {
        lock (_movementTrackerGate)
        {
            _tracker.Reset();
        }
    }

    private int? FindGameProcessId()
    {
        var processes = Process.GetProcessesByName(_processName);
        try
        {
            return processes.Length == 0 ? null : processes[0].Id;
        }
        finally
        {
            foreach (var process in processes)
            {
                process.Dispose();
            }
        }
    }

    private static IEnumerable<ILiveDevice> SelectActiveDevices(CaptureDeviceList devices)
    {
        var activeDescriptions = NetworkInterface.GetAllNetworkInterfaces()
            .Where(network => IsEligibleCaptureNetwork(
                network.OperationalStatus,
                network.NetworkInterfaceType))
            .Select(network => network.Description)
            .ToHashSet(StringComparer.OrdinalIgnoreCase);
        var active = devices
            .Where(device =>
                !string.IsNullOrWhiteSpace(device.Description)
                && activeDescriptions.Contains(device.Description))
            .ToArray();
        return active.Length > 0 ? active : devices;
    }

    internal static bool IsEligibleCaptureNetwork(
        OperationalStatus operationalStatus,
        NetworkInterfaceType networkInterfaceType) =>
        operationalStatus == OperationalStatus.Up
        && networkInterfaceType != NetworkInterfaceType.Loopback;

    internal static string BuildCaptureFilter(IEnumerable<int> ports) =>
        $"udp and ({string.Join(" or ", ports.Order().Select(port => $"src port {port}"))})";

    private sealed record OpenedCapture(
        ILiveDevice Device,
        PacketArrivalEventHandler Handler);
}
