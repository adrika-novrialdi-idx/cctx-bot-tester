namespace IndodaxSandbox;

/// <summary>
/// CLI entrypoint for the Indodax sandbox bot.
/// </summary>
public static class Program
{
    /// <summary>
    /// Parses flags, loads credentials, and runs the sandbox bot.
    /// </summary>
    /// <param name="args">Command-line arguments.</param>
    /// <returns>Process exit code.</returns>
    public static async Task<int> Main(string[] args)
    {
        var trade = false;
        var verbose = false;
        string? configPath = null;

        for (var i = 0; i < args.Length; i++)
        {
            switch (args[i])
            {
                case "--trade":
                    trade = true;
                    break;
                case "--verbose":
                    verbose = true;
                    break;
                case "--config":
                    if (i + 1 >= args.Length)
                    {
                        Console.Error.WriteLine("missing value for --config");
                        return 2;
                    }
                    configPath = args[++i];
                    break;
                default:
                    Console.Error.WriteLine($"unknown argument: {args[i]}");
                    return 2;
            }
        }

        Credentials? credentials = null;
        try
        {
            credentials = Config.Load(configPath);
        }
        catch (CliException ex)
        {
            Console.Error.WriteLine(FormatError(ex.Message, credentials));
            return ex.ExitCode;
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine(FormatError(ex.Message, credentials));
            return 1;
        }

        try
        {
            var exchange = ExchangeFactory.Create(credentials, verbose);
            await Bot.RunAsync(exchange, trade);
            return 0;
        }
        catch (CliException ex)
        {
            Console.Error.WriteLine(FormatError(ex.Message, credentials));
            return ex.ExitCode;
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine(FormatError(ex.Message, credentials));
            return 1;
        }
    }

    /// <summary>
    /// Redacts credential values from an error message before writing to stderr.
    /// </summary>
    private static string FormatError(string message, Credentials? credentials)
    {
        var result = message;
        if (credentials is not null)
        {
            if (!string.IsNullOrEmpty(credentials.ApiKey))
            {
                result = result.Replace(credentials.ApiKey, "[REDACTED]", StringComparison.Ordinal);
            }
            if (!string.IsNullOrEmpty(credentials.Secret))
            {
                result = result.Replace(credentials.Secret, "[REDACTED]", StringComparison.Ordinal);
            }
        }
        return result;
    }
}
