using ccxt;

namespace IndodaxSandbox;

/// <summary>
/// Creates a CCXT Indodax client pointed at the d-idx.com sandbox hosts.
/// </summary>
public static class ExchangeFactory
{
    /// <summary>
    /// Builds an Indodax exchange instance for the sandbox hosts.
    /// </summary>
    /// <param name="credentials">API key and secret.</param>
    /// <param name="verbose">Whether to enable CCXT verbose logging.</param>
    /// <returns>Configured Indodax exchange.</returns>
    public static Indodax Create(Credentials credentials, bool verbose = false)
    {
        return new Indodax(new Dictionary<string, object>
        {
            { "apiKey", credentials.ApiKey },
            { "secret", credentials.Secret },
            { "enableRateLimit", true },
            { "verbose", verbose },
            {
                "urls", new Dictionary<string, object>
                {
                    {
                        "api", new Dictionary<string, object>
                        {
                            { "public", "https://d-idx.com" },
                            { "private", "https://d-idx.com/tapi" },
                        }
                    },
                }
            },
            {
                "options", new Dictionary<string, object>
                {
                    { "tapiVersion", "2" },
                    { "sandboxUrl", "https://api.d-idx.com" },
                }
            },
        });
    }
}
