<?php

declare(strict_types=1);

namespace IndodaxSandbox;

use ccxt\indodax;

/**
 * Creates a CCXT Indodax client pointed at the d-idx.com sandbox hosts.
 */
final class ExchangeFactory
{
    /**
     * Builds an Indodax exchange instance for the sandbox hosts.
     *
     * @param array{apiKey: string, secret: string} $credentials
     * @param bool $verbose Whether to enable CCXT verbose logging.
     * @return indodax
     */
    public static function create(array $credentials, bool $verbose = false): indodax
    {
        return new indodax([
            'apiKey' => $credentials['apiKey'],
            'secret' => $credentials['secret'],
            'enableRateLimit' => true,
            'verbose' => $verbose,
            'urls' => [
                'api' => [
                    'public' => 'https://d-idx.com',
                    'private' => 'https://d-idx.com/tapi',
                ],
            ],
            'options' => [
                'tapiVersion' => '2',
                'sandboxUrl' => 'https://api.d-idx.com',
            ],
        ]);
    }
}
