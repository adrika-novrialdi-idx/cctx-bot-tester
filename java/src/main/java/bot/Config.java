package bot;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Loads Indodax credentials from a JSON file, then applies non-empty environment overrides.
 */
public final class Config {
    private static final Pattern API_KEY_PATTERN = Pattern.compile(
            "\"apiKey\"\\s*:\\s*\"((?:\\\\.|[^\"\\\\])*)\"");
    private static final Pattern SECRET_PATTERN = Pattern.compile(
            "\"secret\"\\s*:\\s*\"((?:\\\\.|[^\"\\\\])*)\"");

    private Config() {
    }

    /**
     * Loads credentials from a JSON file and environment variables.
     *
     * @param configPath path to the JSON file; defaults to config.json in the cwd when null or blank
     * @return an array of two strings: apiKey at index 0, secret at index 1
     * @throws IllegalArgumentException when a required credential is missing after file and env merge
     * @throws IOException when the config file cannot be read
     */
    public static String[] load(String configPath) throws IOException {
        String path = (configPath == null || configPath.isBlank()) ? "config.json" : configPath;
        String fileApiKey = "";
        String fileSecret = "";

        Path file = Path.of(path);
        if (Files.isRegularFile(file)) {
            String raw = Files.readString(file, StandardCharsets.UTF_8);
            fileApiKey = extractStringField(raw, API_KEY_PATTERN);
            fileSecret = extractStringField(raw, SECRET_PATTERN);
        }

        String envApiKey = System.getenv("INDODAX_APIKEY");
        String envSecret = System.getenv("INDODAX_SECRET");
        if (envApiKey == null) {
            envApiKey = "";
        }
        if (envSecret == null) {
            envSecret = "";
        }

        String apiKey = !envApiKey.isEmpty() ? envApiKey : fileApiKey;
        String secret = !envSecret.isEmpty() ? envSecret : fileSecret;

        if (apiKey.isEmpty()) {
            throw new IllegalArgumentException("missing INDODAX_APIKEY");
        }
        if (secret.isEmpty()) {
            throw new IllegalArgumentException("missing INDODAX_SECRET");
        }

        return new String[] { apiKey, secret };
    }

    private static String extractStringField(String json, Pattern pattern) {
        Matcher matcher = pattern.matcher(json);
        if (!matcher.find()) {
            return "";
        }
        return unescapeJsonString(matcher.group(1));
    }

    private static String unescapeJsonString(String value) {
        StringBuilder result = new StringBuilder(value.length());
        for (int i = 0; i < value.length(); i++) {
            char current = value.charAt(i);
            if (current == '\\' && i + 1 < value.length()) {
                char next = value.charAt(++i);
                switch (next) {
                    case '"', '\\', '/' -> result.append(next);
                    case 'b' -> result.append('\b');
                    case 'f' -> result.append('\f');
                    case 'n' -> result.append('\n');
                    case 'r' -> result.append('\r');
                    case 't' -> result.append('\t');
                    default -> {
                        result.append('\\');
                        result.append(next);
                    }
                }
            } else {
                result.append(current);
            }
        }
        return result.toString();
    }
}
