package bot;

import io.github.ccxt.exchanges.Indodax;
import io.github.ccxt.types.Order;
import io.github.ccxt.types.Ticker;

/**
 * Runs the Indodax sandbox read-only path, and the optional trade path when requested.
 */
public final class Bot {
    private static final String SYMBOL = "BTC/USDT";
    private static final double TARGET_NOTIONAL_USD = 10.0;
    private static final double MAX_NOTIONAL_USD = 25.0;
    private static final String PUBLIC_HOST = "https://d-idx.com";
    private static final String PRIVATE_HOST = "https://api.d-idx.com";

    private Bot() {
    }

    /**
     * Executes the sandbox bot against the given exchange.
     *
     * @param exchange configured Indodax exchange
     * @param trade when true, place and cancel one limit buy after the read-only path
     * @throws IllegalArgumentException when the ticker or notional guard refuses the trade
     */
    public static void run(Indodax exchange, boolean trade) {
        Long serverTime = exchange.fetchTime();
        System.out.println("server time: " + serverTime);

        exchange.fetchBalance();
        System.out.println("ok public=" + PUBLIC_HOST + " private=" + PRIVATE_HOST);

        if (!trade) {
            return;
        }

        exchange.loadMarkets(true);

        Ticker ticker = exchange.fetchTicker(SYMBOL);
        Double last = ticker == null ? null : ticker.last;
        if (last == null || !Double.isFinite(last) || last <= 0.0) {
            throw new IllegalArgumentException("ticker last price is missing or not greater than 0");
        }

        double rawPrice = last * 0.5;
        double rawAmount = TARGET_NOTIONAL_USD / rawPrice;
        Object precisePrice = exchange.priceToPrecision(SYMBOL, rawPrice);
        Object preciseAmount = exchange.amountToPrecision(SYMBOL, rawAmount);
        double price = Double.parseDouble(String.valueOf(precisePrice));
        double amount = Double.parseDouble(String.valueOf(preciseAmount));
        double notional = amount * price;

        if (!Double.isFinite(amount) || amount <= 0.0 || !Double.isFinite(notional) || notional >= MAX_NOTIONAL_USD) {
            String value = Double.isFinite(notional) ? String.valueOf(notional) : "NaN";
            throw new IllegalArgumentException("refused: notional " + value + " USD is not below 25");
        }

        String orderId = null;
        try {
            Order order = exchange.createOrder(SYMBOL, "limit", "buy", amount, price);
            if (order != null && order.id != null && !order.id.isEmpty()) {
                orderId = order.id;
                System.out.println("order id: " + orderId);
            }
        } finally {
            if (orderId != null && !orderId.isEmpty()) {
                exchange.cancelOrder(orderId, SYMBOL);
                System.out.println("canceled order id: " + orderId);
            }
        }
    }
}
