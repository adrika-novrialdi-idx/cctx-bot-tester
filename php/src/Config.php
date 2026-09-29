<?php

declare(strict_types=1);

namespace IndodaxSandbox;

/**
 * Loads Indodax credentials from a JSON file, then applies non-empty environment overrides.
 */
final class Config
{
    /**
     * Loads credentials from a JSON file and environment variables.
     *
     * @param string $configPath Path to the JSON file. Defaults to config.json in the cwd.
     * @return array{apiKey: string, secret: string}
     * @throws CliError When a required credential is missing after file and env merge.
     */
    public static function load(string $configPath = 'config.json'): array
    {
        $fileApiKey = '';
        $fileSecret = '';

        if (is_file($configPath)) {
            $raw = file_get_contents($configPath);
            if ($raw === false) {
                throw new CliError('unable to read config file: ' . $configPath, 'CONFIG_MISSING');
            }
            $parsed = json_decode($raw, true);
            if (!is_array($parsed)) {
                throw new CliError('invalid config JSON: ' . $configPath, 'CONFIG_MISSING');
            }
            if (isset($parsed['apiKey']) && is_string($parsed['apiKey'])) {
                $fileApiKey = $parsed['apiKey'];
            }
            if (isset($parsed['secret']) && is_string($parsed['secret'])) {
                $fileSecret = $parsed['secret'];
            }
        }

        $envApiKey = getenv('INDODAX_APIKEY');
        $envSecret = getenv('INDODAX_SECRET');
        $envApiKey = ($envApiKey !== false && $envApiKey !== '') ? $envApiKey : '';
        $envSecret = ($envSecret !== false && $envSecret !== '') ? $envSecret : '';

        $apiKey = $envApiKey !== '' ? $envApiKey : $fileApiKey;
        $secret = $envSecret !== '' ? $envSecret : $fileSecret;

        if ($apiKey === '') {
            throw new CliError('missing INDODAX_APIKEY', 'CONFIG_MISSING');
        }
        if ($secret === '') {
            throw new CliError('missing INDODAX_SECRET', 'CONFIG_MISSING');
        }

        return [
            'apiKey' => $apiKey,
            'secret' => $secret,
        ];
    }
}
