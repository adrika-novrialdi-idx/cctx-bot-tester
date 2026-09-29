<?php

declare(strict_types=1);

namespace IndodaxSandbox;

use RuntimeException;

/**
 * CLI failure with a stable string code for exit-status mapping.
 */
final class CliError extends RuntimeException
{
    /** @var string */
    public string $errorCode;

    /**
     * @param string $message Human-readable error message.
     * @param string $errorCode Stable code such as CONFIG_MISSING or NOTIONAL_REFUSED.
     */
    public function __construct(string $message, string $errorCode)
    {
        parent::__construct($message);
        $this->errorCode = $errorCode;
    }
}
