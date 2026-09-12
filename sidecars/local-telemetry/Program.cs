using System.Text.Json;
using System.Text.Json.Serialization;
using System.Diagnostics;
using IslePulse.LocalTelemetry;

Console.OutputEncoding = System.Text.Encoding.UTF8;
var json = new JsonSerializerOptions
{
    DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    PropertyNamingPolicy = JsonNamingPolicy.CamelCase
};

if (args.Contains("--install-npcap", StringComparer.OrdinalIgnoreCase))
{
    var progress = new Progress<NpcapSetupProgress>(update =>
        Write(new SetupProgressMessage(
            "setup-progress",
            update.Stage.ToString().ToLowerInvariant(),
            update.Percent,
            update.Message)));
    var result = await new NpcapSetupService().InstallAsync(progress);
    Write(new SetupResultMessage(
        "setup-result",
        result.Outcome.ToString().ToLowerInvariant(),
        result.Message));
    return result.Outcome switch
    {
        NpcapSetupOutcome.Ready => 0,
        NpcapSetupOutcome.Cancelled => 3,
        NpcapSetupOutcome.RebootRequired => 4,
        _ => 5
    };
}

var availability = NpcapAvailabilityProbe.Check();
if (!availability.IsAvailable)
{
    Write(new StatusMessage("error", "npcap-required", availability.ErrorMessage));
    return 2;
}

using var shutdown = new CancellationTokenSource();
Console.CancelKeyPress += (_, e) =>
{
    e.Cancel = true;
    shutdown.Cancel();
};

var parentProcessId = ReadParentProcessId(args);
if (parentProcessId is not null)
{
    _ = WatchParentAsync(parentProcessId.Value, shutdown);
}

await using var source = new NpcapLocalMovementSource();
try
{
    Write(new StatusMessage("status", "starting", null));
    await foreach (var observation in source.WatchAsync(shutdown.Token))
    {
        var movement = observation.Movement;
        Write(new MovementMessage(
            "movement",
            movement.Location.X,
            movement.Location.Y,
            movement.Location.Z,
            movement.MapHeadingDegrees,
            movement.UnrealYawDegrees,
            observation.ObservedAt.ToUnixTimeMilliseconds(),
            observation.ServerEndpoint));
    }
}
catch (OperationCanceledException) when (shutdown.IsCancellationRequested)
{
}
catch (LocalPacketCaptureUnavailableException exception)
{
    Write(new StatusMessage("error", "capture-unavailable", exception.Message));
    return 2;
}
catch (Exception exception)
{
    Write(new StatusMessage("error", "faulted", exception.Message));
    return 1;
}

return 0;

void Write<T>(T message)
{
    Console.WriteLine(JsonSerializer.Serialize(message, json));
    Console.Out.Flush();
}

static int? ReadParentProcessId(IReadOnlyList<string> arguments)
{
    for (var index = 0; index + 1 < arguments.Count; index++)
    {
        if (arguments[index] == "--parent-pid"
            && int.TryParse(arguments[index + 1], out var processId)
            && processId > 0)
        {
            return processId;
        }
    }

    return null;
}

static async Task WatchParentAsync(int processId, CancellationTokenSource shutdown)
{
    try
    {
        using var parent = Process.GetProcessById(processId);
        while (!shutdown.IsCancellationRequested && !parent.HasExited)
        {
            await Task.Delay(TimeSpan.FromSeconds(1), shutdown.Token);
        }
    }
    catch (OperationCanceledException) when (shutdown.IsCancellationRequested)
    {
        return;
    }
    catch (ArgumentException)
    {
        // The parent exited before the sidecar could attach to it.
    }

    shutdown.Cancel();
}

internal sealed record MovementMessage(
    string Type,
    double X,
    double Y,
    double Z,
    double HeadingDeg,
    double YawDeg,
    long ObservedAtMs,
    string? ServerEndpoint);

internal sealed record StatusMessage(
    string Type,
    string Status,
    string? Message);

internal sealed record SetupProgressMessage(
    string Type,
    string Stage,
    int? Percent,
    string Message);

internal sealed record SetupResultMessage(
    string Type,
    string Outcome,
    string Message);
