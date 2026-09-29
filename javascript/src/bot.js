const SYMBOL = 'BTC/USDT';
const TARGET_NOTIONAL_USD = 10;
const MAX_NOTIONAL_USD = 25;
const PUBLIC_HOST = 'https://d-idx.com';
const PRIVATE_HOST = 'https://api.d-idx.com';

/**
 * @typedef {object} BotOptions
 * @property {boolean} [trade]
 */

/**
 * Runs the Indodax sandbox read-only path, and the optional trade path when requested.
 * @param {import('ccxt').Exchange} exchange
 * @param {BotOptions} [options]
 * @returns {Promise<void>}
 */
export async function runBot(exchange, options = {}) {
  const trade = options.trade === true;

  const serverTime = await exchange.fetchTime();
  console.log(`server time: ${serverTime}`);

  await exchange.fetchBalance();
  console.log(`ok public=${PUBLIC_HOST} private=${PRIVATE_HOST}`);

  if (!trade) {
    return;
  }

  await exchange.loadMarkets();

  const ticker = await exchange.fetchTicker(SYMBOL);
  const last = Number(ticker?.last);
  if (!Number.isFinite(last) || last <= 0) {
    const error = new Error('ticker last price is missing or not greater than 0');
    error.code = 'NOTIONAL_REFUSED';
    throw error;
  }

  const rawPrice = last * 0.5;
  const rawAmount = TARGET_NOTIONAL_USD / rawPrice;
  const precisePrice = exchange.priceToPrecision(SYMBOL, rawPrice);
  const preciseAmount = exchange.amountToPrecision(SYMBOL, rawAmount);
  const price = Number(precisePrice);
  const amount = Number(preciseAmount);
  const notional = amount * price;

  if (!Number.isFinite(amount) || amount <= 0 || !Number.isFinite(notional) || notional >= MAX_NOTIONAL_USD) {
    const value = Number.isFinite(notional) ? notional : 'NaN';
    const error = new Error(`refused: notional ${value} USD is not below 25`);
    error.code = 'NOTIONAL_REFUSED';
    throw error;
  }

  let orderId;
  try {
    const order = await exchange.createOrder(SYMBOL, 'limit', 'buy', amount, price);
    orderId = order?.id;
    if (orderId !== undefined && orderId !== null && orderId !== '') {
      console.log(`order id: ${orderId}`);
    }
  } finally {
    if (orderId !== undefined && orderId !== null && orderId !== '') {
      await exchange.cancelOrder(orderId, SYMBOL);
      console.log(`canceled order id: ${orderId}`);
    }
  }
}
