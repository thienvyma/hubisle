using System.Threading.Channels;

namespace IslemapThienvyma.Telemetry;

internal readonly record struct CapturedUdpDatagram(
    DateTimeOffset ObservedAt,
    string? SourceAddress,
    int SourcePort,
    string? DestinationAddress,
    int DestinationPort,
    byte[] Payload,
    bool Outbound = false);

internal sealed class BoundedPacketIntake
{
    private const int DefaultPacketCapacity = 8_192;
    private const long DefaultByteCapacity = 32L * 1024 * 1024;

    private readonly Channel<CapturedUdpDatagram> _channel;
    private readonly long _byteCapacity;
    private long _queuedBytes;
    private int _queuedPackets;

    public BoundedPacketIntake(
        int packetCapacity = DefaultPacketCapacity,
        long byteCapacity = DefaultByteCapacity)
    {
        _byteCapacity = byteCapacity;
        _channel = Channel.CreateBounded<CapturedUdpDatagram>(
            new BoundedChannelOptions(packetCapacity)
            {
                FullMode = BoundedChannelFullMode.Wait,
                SingleReader = true,
                SingleWriter = false,
                AllowSynchronousContinuations = false
            });
    }

    public bool TryEnqueue(CapturedUdpDatagram datagram)
    {
        var byteCount = datagram.Payload.Length;
        if (byteCount <= 0 || byteCount > _byteCapacity)
        {
            return false;
        }

        var queuedBytes = Interlocked.Add(ref _queuedBytes, byteCount);
        if (queuedBytes > _byteCapacity)
        {
            Interlocked.Add(ref _queuedBytes, -byteCount);
            return false;
        }

        if (!_channel.Writer.TryWrite(datagram))
        {
            Interlocked.Add(ref _queuedBytes, -byteCount);
            return false;
        }

        Interlocked.Increment(ref _queuedPackets);
        return true;
    }

    public async IAsyncEnumerable<CapturedUdpDatagram> ReadAllAsync(
        [System.Runtime.CompilerServices.EnumeratorCancellation]
        CancellationToken cancellationToken)
    {
        await foreach (var datagram in _channel.Reader
                           .ReadAllAsync(cancellationToken)
                           .ConfigureAwait(false))
        {
            Interlocked.Decrement(ref _queuedPackets);
            Interlocked.Add(ref _queuedBytes, -datagram.Payload.Length);
            yield return datagram;
        }
    }

    public void Complete() => _channel.Writer.TryComplete();
}
