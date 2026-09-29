package bot;

import java.util.HashMap;
import java.util.Map;

import io.github.ccxt.exchanges.Indodax;

/**
 * Creates a CCXT Indodax client pointed at the d-idx.com sandbox hosts.
 */
public final class ExchangeFactory {
    private ExchangeFactory() {
    }

    /**
     * Builds an Indodax exchange instance for the sandbox hosts.
     *
     * @param apiKey Indodax API key
     * @param secret Indodax API secret
     * @param verbose whether to enable CCXT verbose logging
     * @return a configured Indodax exchange
     */
    public static Indodax create(String apiKey, String secret, boolean verbose) {
        Map<String, Object> urlsApi = new HashMap<>();
        urlsApi.put("public", "https://d-idx.com");
        urlsApi.put("private", "https://d-idx.com/tapi");

        Map<String, Object> urls = new HashMap<>();
        urls.put("api", urlsApi);

        Map<String, Object> options = new HashMap<>();
        options.put("tapiVersion", "2");
        options.put("sandboxUrl", "https://api.d-idx.com");

        Map<String, Object> config = new HashMap<>();
        config.put("apiKey", apiKey);
        config.put("secret", secret);
        config.put("enableRateLimit", true);
        config.put("verbose", verbose);
        config.put("urls", urls);
        config.put("options", options);

        return new Indodax(config);
    }
}
