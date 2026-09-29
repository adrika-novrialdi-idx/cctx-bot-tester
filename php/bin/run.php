#!/usr/bin/env php
<?php

declare(strict_types=1);

require_once dirname(__DIR__) . '/vendor/autoload.php';

use IndodaxSandbox\Bot;
use IndodaxSandbox\CliError;
use IndodaxSandbox\Config;
use IndodaxSandbox\ExchangeFactory;

/**
 * Redacts credential values from an error message before writing to stderr.
 *
 * @param Throwable|string $err
 * @param array{apiKey?: string, secret?: string} $credentials
 * @return string
 */
function format_error($err, array $credentials = []): string
{
    $message = $err instanceof Throwable ? $err->getMessage() : (string) $err;
    if (!empty($credentials['apiKey'])) {
        $message = str_replace($credentials['apiKey'], '[REDACTED]', $message);
    }
    if (!empty($credentials['secret'])) {
        $message = str_replace($credentials['secret'], '[REDACTED]', $message);
    }
    return $message;
}

/**
 * Parses CLI flags from $argv.
 *
 * @param list<string> $argv
 * @return array{trade: bool, verbose: bool, config: string|null}
 */
function parse_args(array $argv): array
{
    $trade = false;
    $verbose = false;
    $config = null;

    $args = array_slice($argv, 1);
    $count = count($args);
    for ($i = 0; $i < $count; $i++) {
        $arg = $args[$i];
        if ($arg === '--trade') {
            $trade = true;
            continue;
        }
        if ($arg === '--verbose') {
            $verbose = true;
            continue;
        }
        if ($arg === '--config') {
            if ($i + 1 >= $count) {
                throw new RuntimeException('missing value for --config');
            }
            $i++;
            $config = $args[$i];
            continue;
        }
        if (str_starts_with($arg, '--config=')) {
            $config = substr($arg, strlen('--config='));
            continue;
        }
        throw new RuntimeException('unknown argument: ' . $arg);
    }

    return [
        'trade' => $trade,
        'verbose' => $verbose,
        'config' => $config,
    ];
}

/**
 * Parses CLI flags and runs the sandbox bot.
 *
 * @param list<string> $argv
 * @return int Process exit code.
 */
function main(array $argv): int
{
    try {
        $flags = parse_args($argv);
    } catch (Throwable $err) {
        fwrite(STDERR, format_error($err) . PHP_EOL);
        return 1;
    }

    $credentials = null;
    try {
        $credentials = Config::load($flags['config'] ?? 'config.json');
    } catch (Throwable $err) {
        fwrite(STDERR, format_error($err) . PHP_EOL);
        return ($err instanceof CliError && $err->errorCode === 'CONFIG_MISSING') ? 2 : 1;
    }

    try {
        $exchange = ExchangeFactory::create($credentials, $flags['verbose']);
        Bot::run($exchange, $flags['trade']);
    } catch (Throwable $err) {
        fwrite(STDERR, format_error($err, $credentials) . PHP_EOL);
        return ($err instanceof CliError && $err->errorCode === 'NOTIONAL_REFUSED') ? 2 : 1;
    }

    return 0;
}

exit(main($argv));
