<?php

declare(strict_types=1);

namespace IndodaxSandbox;

use ccxt\Exchange;
use Throwable;

/**
 * Runs the Indodax sandbox read-only path, and the optional trade path when requested.
 */
final class Bot
{
    private const SYMBOL = 'BTC/USDT';
    private const TARGET_NOTIONAL_USD = 10.0;
    private const MAX_NOTIONAL_USD = 25.0;
    private const PUBLIC_HOST = 'https://d-idx.com';
    private const PRIVATE_HOST = 'https://api.d-idx.com';

    /**
     * Executes the sandbox bot against the given exchange.
     *
     * @param Exchange $exchange CCXT exchange instance.
     * @param bool $trade When true, place and cancel one limit buy after the read-only path.
     * @return void
     * @throws CliError When the ticker or notional guard refuses the trade.
     * @throws Throwable When an exchange call fails.
     */
    public static function run(Exchange $exchange, bool $trade = false): void
    {
        $serverTime = $exchange->fetch_time();
        fwrite(STDOUT, 'server time: ' . $serverTime . PHP_EOL);

        $exchange->fetch_balance();
        fwrite(STDOUT, 'ok public=' . self::PUBLIC_HOST . ' private=' . self::PRIVATE_HOST . PHP_EOL);

        if (!$trade) {
            return;
        }

        $exchange->load_markets();

        $ticker = $exchange->fetch_ticker(self::SYMBOL);
        $last = isset($ticker['last']) ? (float) $ticker['last'] : 0.0;
        if (!is_finite($last) || $last <= 0.0) {
            throw new CliError('ticker last price is missing or not greater than 0', 'NOTIONAL_REFUSED');
        }

        $rawPrice = $last * 0.5;
        $rawAmount = self::TARGET_NOTIONAL_USD / $rawPrice;
        $precisePrice = $exchange->price_to_precision(self::SYMBOL, $rawPrice);
        $preciseAmount = $exchange->amount_to_precision(self::SYMBOL, $rawAmount);
        $price = (float) $precisePrice;
        $amount = (float) $preciseAmount;
        $notional = $amount * $price;

        if (!is_finite($amount) || $amount <= 0.0 || !is_finite($notional) || $notional >= self::MAX_NOTIONAL_USD) {
            $value = is_finite($notional) ? (string) $notional : 'NaN';
            throw new CliError('refused: notional ' . $value . ' USD is not below 25', 'NOTIONAL_REFUSED');
        }

        $orderId = null;
        try {
            $order = $exchange->create_order(self::SYMBOL, 'limit', 'buy', $amount, $price);
            if (isset($order['id']) && $order['id'] !== null && $order['id'] !== '') {
                $orderId = $order['id'];
                fwrite(STDOUT, 'order id: ' . $orderId . PHP_EOL);
            }
        } finally {
            if ($orderId !== null && $orderId !== '') {
                $exchange->cancel_order($orderId, self::SYMBOL);
                fwrite(STDOUT, 'canceled order id: ' . $orderId . PHP_EOL);
            }
        }
    }
}
