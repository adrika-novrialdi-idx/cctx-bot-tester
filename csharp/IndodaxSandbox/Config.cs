using System.Text.Json;

namespace IndodaxSandbox;

/// <summary>
/// Holds Indodax API credentials after file and environment merge.
/// </summary>
public sealed class Credentials
{
    /// <summary>
    /// Gets the API key.
    /// </summary>
    public required string ApiKey { get; init; }

    /// <summary>
    /// Gets the API secret.
    /// </summary>
    public required string Secret { get; init; }
}

/// <summary>
/// Loads Indodax credentials from a JSON file, then applies non-empty environment overrides.
/// </summary>
public static class Config
{
    /// <summary>
    /// Loads credentials from a JSON file and environment variables.
    /// </summary>
    /// <param name="configPath">Path to the JSON file. Defaults to config.json in the cwd.</param>
    /// <returns>Merged API key and secret.</returns>
    /// <exception cref="CliException">When a required credential is missing after file and env merge.</exception>
    public static Credentials Load(string? configPath = null)
    {
        var path = string.IsNullOrWhiteSpace(configPath) ? "config.json" : configPath;
        var fileApiKey = "";
        var fileSecret = "";

        if (File.Exists(path))
        {
            var raw = File.ReadAllText(path);
            using var document = JsonDocument.Parse(raw);
            var root = document.RootElement;
            if (root.TryGetProperty("apiKey", out var apiKeyElement) && apiKeyElement.ValueKind == JsonValueKind.String)
            {
                fileApiKey = apiKeyElement.GetString() ?? "";
            }
            if (root.TryGetProperty("secret", out var secretElement) && secretElement.ValueKind == JsonValueKind.String)
            {
                fileSecret = secretElement.GetString() ?? "";
            }
        }

        var envApiKey = Environment.GetEnvironmentVariable("INDODAX_APIKEY") ?? "";
        var envSecret = Environment.GetEnvironmentVariable("INDODAX_SECRET") ?? "";

        var apiKey = envApiKey != "" ? envApiKey : fileApiKey;
        var secret = envSecret != "" ? envSecret : fileSecret;

        if (apiKey == "")
        {
            throw new CliException("missing INDODAX_APIKEY", 2);
        }
        if (secret == "")
        {
            throw new CliException("missing INDODAX_SECRET", 2);
        }

        return new Credentials
        {
            ApiKey = apiKey,
            Secret = secret,
        };
    }
}

/// <summary>
/// Application error with a preferred process exit code.
/// </summary>
public sealed class CliException : Exception
{
    /// <summary>
    /// Gets the process exit code for this error.
    /// </summary>
    public int ExitCode { get; }

    /// <summary>
    /// Creates a CLI error with a message and exit code.
    /// </summary>
    /// <param name="message">Error message written to stderr.</param>
    /// <param name="exitCode">Process exit code.</param>
    public CliException(string message, int exitCode)
        : base(message)
    {
        ExitCode = exitCode;
    }
}
