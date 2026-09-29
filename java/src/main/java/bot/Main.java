package bot;

/**
 * CLI entrypoint for the Indodax sandbox bot.
 */
public final class Main {
    private Main() {
    }

    /**
     * Parses CLI flags, loads credentials, and runs the sandbox bot.
     *
     * @param args command-line arguments
     */
    public static void main(String[] args) {
        boolean trade = false;
        boolean verbose = false;
        String configPath = null;

        try {
            for (int i = 0; i < args.length; i++) {
                String arg = args[i];
                if ("--trade".equals(arg)) {
                    trade = true;
                    continue;
                }
                if ("--verbose".equals(arg)) {
                    verbose = true;
                    continue;
                }
                if ("--config".equals(arg)) {
                    if (i + 1 >= args.length) {
                        throw new IllegalArgumentException("missing value for --config");
                    }
                    configPath = args[++i];
                    continue;
                }
                if (arg.startsWith("--config=")) {
                    configPath = arg.substring("--config=".length());
                    continue;
                }
                throw new IllegalArgumentException("unknown argument: " + arg);
            }
        } catch (IllegalArgumentException err) {
            System.err.println(formatError(err, null, null));
            System.exit(1);
            return;
        }

        String apiKey;
        String secret;
        try {
            String[] credentials = Config.load(configPath);
            apiKey = credentials[0];
            secret = credentials[1];
        } catch (IllegalArgumentException err) {
            System.err.println(formatError(err, null, null));
            System.exit(2);
            return;
        } catch (Exception err) {
            System.err.println(formatError(err, null, null));
            System.exit(1);
            return;
        }

        try {
            var exchange = ExchangeFactory.create(apiKey, secret, verbose);
            Bot.run(exchange, trade);
        } catch (IllegalArgumentException err) {
            System.err.println(formatError(err, apiKey, secret));
            System.exit(2);
        } catch (Exception err) {
            System.err.println(formatError(err, apiKey, secret));
            System.exit(1);
        }
    }

    private static String formatError(Throwable err, String apiKey, String secret) {
        String message = err.getMessage();
        if (message == null || message.isEmpty()) {
            message = err.toString();
        }
        if (apiKey != null && !apiKey.isEmpty()) {
            message = message.replace(apiKey, "[REDACTED]");
        }
        if (secret != null && !secret.isEmpty()) {
            message = message.replace(secret, "[REDACTED]");
        }
        return message;
    }
}
