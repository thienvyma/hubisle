using System.Buffers;
using System.ComponentModel;
using System.Diagnostics;
using System.Net.Http;
using System.Security.Cryptography;
using System.Security.Cryptography.X509Certificates;

namespace IslePulse.LocalTelemetry;

public enum NpcapSetupStage
{
    Downloading,
    Verifying,
    Installing,
    Checking
}

public readonly record struct NpcapSetupProgress(
    NpcapSetupStage Stage,
    int? Percent,
    string Message);

public enum NpcapSetupOutcome
{
    Ready,
    Cancelled,
    RebootRequired,
    Failed
}

public sealed record NpcapSetupResult(NpcapSetupOutcome Outcome, string Message);

public sealed class NpcapSetupService
{
    public const string InstallerVersion = "1.88";
    public const string OfficialInstallerSha256 =
        "A2F4EC1E5EA353FF67EFD24B2EBF081BA44532410FAE8D5E146AF0310AA4F56B";
    public static readonly Uri OfficialInstallerUri = new(
        $"https://npcap.com/dist/npcap-{InstallerVersion}.exe");

    private const long MaximumInstallerBytes = 8 * 1024 * 1024;
    private const string OfficialSignerName = "Nmap Software LLC";
    private const string OfficialSignerThumbprint = "0629C303220B256580AABA536A1A3C060B87E3A2";
    private static readonly HttpClient SharedHttpClient = CreateHttpClient();

    public async Task<NpcapSetupResult> InstallAsync(
        IProgress<NpcapSetupProgress>? progress = null,
        CancellationToken cancellationToken = default)
    {
        var downloadDirectory = Path.Combine(Path.GetTempPath(), "IslePulseOverlay", "Npcap");
        var installerPath = Path.Combine(
            downloadDirectory,
            $"npcap-{InstallerVersion}-{Guid.NewGuid():N}.exe");

        try
        {
            Directory.CreateDirectory(downloadDirectory);
            await DownloadInstallerAsync(installerPath, progress, cancellationToken)
                .ConfigureAwait(false);
            progress?.Report(new NpcapSetupProgress(
                NpcapSetupStage.Verifying,
                null,
                "Verifying the official Npcap installer."));

            if (!await HasExpectedHashAsync(installerPath, cancellationToken).ConfigureAwait(false)
                || !HasOfficialSigner(installerPath))
            {
                return new NpcapSetupResult(
                    NpcapSetupOutcome.Failed,
                    "The downloaded Npcap installer did not pass verification.");
            }

            cancellationToken.ThrowIfCancellationRequested();
            progress?.Report(new NpcapSetupProgress(
                NpcapSetupStage.Installing,
                null,
                "Waiting for the Npcap installer."));
            var exitCode = await RunInstallerAsync(installerPath).ConfigureAwait(false);

            progress?.Report(new NpcapSetupProgress(
                NpcapSetupStage.Checking,
                null,
                "Checking Npcap after installation."));
            if (NpcapAvailabilityProbe.Check(refresh: true).IsAvailable)
            {
                return new NpcapSetupResult(NpcapSetupOutcome.Ready, "Npcap is ready.");
            }

            return exitCode switch
            {
                1 => new NpcapSetupResult(NpcapSetupOutcome.Cancelled, "Npcap installation was cancelled."),
                3010 or 350 => new NpcapSetupResult(
                    NpcapSetupOutcome.RebootRequired,
                    "Npcap was installed and Windows must restart."),
                1618 => new NpcapSetupResult(
                    NpcapSetupOutcome.Failed,
                    "Another Windows installer is already running."),
                1633 => new NpcapSetupResult(
                    NpcapSetupOutcome.Failed,
                    "This Windows version is not supported by Npcap."),
                _ => new NpcapSetupResult(
                    NpcapSetupOutcome.Failed,
                    "Npcap is still unavailable. Reinstall it or restart Windows.")
            };
        }
        catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            return new NpcapSetupResult(NpcapSetupOutcome.Cancelled, "Npcap download was cancelled.");
        }
        catch (OperationCanceledException)
        {
            return new NpcapSetupResult(NpcapSetupOutcome.Failed, "Npcap download timed out.");
        }
        catch (Win32Exception exception) when (exception.NativeErrorCode == 1223)
        {
            return new NpcapSetupResult(
                NpcapSetupOutcome.Cancelled,
                "Administrator permission for Npcap was declined.");
        }
        catch (HttpRequestException)
        {
            return new NpcapSetupResult(
                NpcapSetupOutcome.Failed,
                "Npcap could not be downloaded from its official site.");
        }
        catch (Exception exception) when (
            exception is IOException
                or UnauthorizedAccessException
                or CryptographicException
                or InvalidOperationException)
        {
            return new NpcapSetupResult(NpcapSetupOutcome.Failed, exception.Message);
        }
        finally
        {
            TryDeleteInstaller(installerPath);
        }
    }

    private static async Task DownloadInstallerAsync(
        string installerPath,
        IProgress<NpcapSetupProgress>? progress,
        CancellationToken cancellationToken)
    {
        progress?.Report(new NpcapSetupProgress(
            NpcapSetupStage.Downloading,
            0,
            $"Downloading Npcap {InstallerVersion} from npcap.com."));

        using var response = await SharedHttpClient.GetAsync(
                OfficialInstallerUri,
                HttpCompletionOption.ResponseHeadersRead,
                cancellationToken)
            .ConfigureAwait(false);
        response.EnsureSuccessStatusCode();

        var finalUri = response.RequestMessage?.RequestUri;
        if (finalUri is null
            || !string.Equals(finalUri.Scheme, Uri.UriSchemeHttps, StringComparison.OrdinalIgnoreCase)
            || !string.Equals(finalUri.IdnHost, "npcap.com", StringComparison.OrdinalIgnoreCase))
        {
            throw new HttpRequestException("Npcap download redirected outside npcap.com.");
        }

        var expectedLength = response.Content.Headers.ContentLength;
        if (expectedLength > MaximumInstallerBytes)
        {
            throw new HttpRequestException("Npcap installer exceeded the download size limit.");
        }

        await using var input = await response.Content
            .ReadAsStreamAsync(cancellationToken)
            .ConfigureAwait(false);
        await using var output = new FileStream(
            installerPath,
            FileMode.CreateNew,
            FileAccess.Write,
            FileShare.None,
            81_920,
            FileOptions.Asynchronous | FileOptions.SequentialScan);
        var buffer = ArrayPool<byte>.Shared.Rent(81_920);
        long received = 0;
        try
        {
            int read;
            while ((read = await input.ReadAsync(buffer, cancellationToken).ConfigureAwait(false)) > 0)
            {
                received += read;
                if (received > MaximumInstallerBytes)
                {
                    throw new HttpRequestException("Npcap installer exceeded the download size limit.");
                }
                await output.WriteAsync(buffer.AsMemory(0, read), cancellationToken)
                    .ConfigureAwait(false);
                int? percent = expectedLength is > 0
                    ? (int)Math.Min(100, received * 100 / expectedLength.Value)
                    : null;
                progress?.Report(new NpcapSetupProgress(
                    NpcapSetupStage.Downloading,
                    percent,
                    $"Downloading Npcap {InstallerVersion} from npcap.com."));
            }
        }
        finally
        {
            ArrayPool<byte>.Shared.Return(buffer);
        }
    }

    private static async Task<bool> HasExpectedHashAsync(
        string installerPath,
        CancellationToken cancellationToken)
    {
        await using var stream = new FileStream(
            installerPath,
            FileMode.Open,
            FileAccess.Read,
            FileShare.Read,
            81_920,
            FileOptions.Asynchronous | FileOptions.SequentialScan);
        var actual = await SHA256.HashDataAsync(stream, cancellationToken).ConfigureAwait(false);
        var expected = Convert.FromHexString(OfficialInstallerSha256);
        return CryptographicOperations.FixedTimeEquals(actual, expected);
    }

    private static bool HasOfficialSigner(string installerPath)
    {
        using var certificate = X509Certificate.CreateFromSignedFile(installerPath);
        return certificate.Subject.Contains(
                   $"CN={OfficialSignerName}",
                   StringComparison.OrdinalIgnoreCase)
               && string.Equals(
                   certificate.GetCertHashString(),
                   OfficialSignerThumbprint,
                   StringComparison.OrdinalIgnoreCase);
    }

    private static async Task<int> RunInstallerAsync(string installerPath)
    {
        using var process = Process.Start(new ProcessStartInfo(installerPath)
        {
            // These are ordinary graphical-installer defaults. The user still
            // sees and controls the official Npcap setup window. Explicitly
            // request boot-start and unrestricted capture so Isle Pulse keeps
            // working after a Windows restart and without elevation.
            Arguments = "/npf_startup=yes /admin_only=no",
            UseShellExecute = true,
            Verb = "runas"
        }) ?? throw new InvalidOperationException("Windows could not start the Npcap installer.");
        await process.WaitForExitAsync().ConfigureAwait(false);
        return process.ExitCode;
    }

    private static HttpClient CreateHttpClient() => new(new HttpClientHandler
    {
        AllowAutoRedirect = true,
        MaxAutomaticRedirections = 3
    })
    {
        Timeout = TimeSpan.FromMinutes(2)
    };

    private static void TryDeleteInstaller(string installerPath)
    {
        try
        {
            if (File.Exists(installerPath))
            {
                File.Delete(installerPath);
            }
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException)
        {
            // Windows will eventually clear the user's temporary directory.
        }
    }
}
