using System.Globalization;
using ccxt;

namespace IndodaxSandbox;

/// <summary>
/// Runs the Indodax sandbox read-only path, and the optional trade path when requested.
/// </summary>
public static class Bot
{
    private const string Symbol = "BTC/USDT";
    private const double TargetNotionalUsd = 10.0;
    private const double MaxNotionalUsd = 25.0;
    private const string PublicHost = "https://d-idx.com";
    private const string PrivateHost = "https://api.d-idx.com";

    /// <summary>
    /// Executes the sandbox bot against the given exchange.
    /// </summary>
    /// <param name="exchange">CCXT Indodax exchange instance.</param>
    /// <param name="trade">When true, place and cancel one limit buy after the read-only path.</param>
    public static async Task RunAsync(Indodax exchange, bool trade = false)
    {
        var serverTime = await exchange.FetchTime();
        Console.WriteLine($"server time: {serverTime}");

        await exchange.FetchBalance();
        Console.WriteLine($"ok public={PublicHost} private={PrivateHost}");

        if (!trade)
        {
            return;
        }

        var ticker = await exchange.FetchTicker(Symbol);
        var last = ticker.last;
        if (last is null || last <= 0)
        {
            throw new CliException("ticker last price is missing or not greater than 0", 2);
        }

        await exchange.LoadMarkets();

        var rawPrice = last.Value * 0.5;
        var rawAmount = TargetNotionalUsd / rawPrice;
        var precisePrice = exchange.priceToPrecision(Symbol, rawPrice);
        var preciseAmount = exchange.amountToPrecision(Symbol, rawAmount);
        if (precisePrice is null || preciseAmount is null)
        {
            throw new CliException("refused: notional NaN USD is not below 25", 2);
        }

        if (!double.TryParse(precisePrice, NumberStyles.Float, CultureInfo.InvariantCulture, out var price)
            || !double.TryParse(preciseAmount, NumberStyles.Float, CultureInfo.InvariantCulture, out var amount))
        {
            throw new CliException("refused: notional NaN USD is not below 25", 2);
        }

        var notional = amount * price;
        if (amount <= 0 || !double.IsFinite(notional) || notional >= MaxNotionalUsd)
        {
            var value = double.IsFinite(notional) ? notional.ToString(CultureInfo.InvariantCulture) : "NaN";
            throw new CliException($"refused: notional {value} USD is not below 25", 2);
        }

        string? orderId = null;
        try
        {
            var order = await exchange.CreateOrder(Symbol, "limit", "buy", amount, price);
            if (!string.IsNullOrEmpty(order.id))
            {
                orderId = order.id;
                Console.WriteLine($"order id: {orderId}");
            }
        }
        finally
        {
            if (!string.IsNullOrEmpty(orderId))
            {
                await exchange.CancelOrder(orderId, Symbol);
                Console.WriteLine($"canceled order id: {orderId}");
            }
        }
    }
}
