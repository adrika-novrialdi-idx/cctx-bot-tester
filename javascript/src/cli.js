#!/usr/bin/env node
import { pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';
import { loadCredentials } from './config.js';
import { createExchange } from './exchange.js';
import { runBot } from './bot.js';

/**
 * Redacts credential values from an error message before writing to stderr.
 * @param {unknown} err
 * @param {{ apiKey?: string, secret?: string }} [credentials]
 * @returns {string}
 */
function formatError(err, credentials = {}) {
  let message = err instanceof Error ? err.message : String(err);
  if (credentials.apiKey) {
    message = message.split(credentials.apiKey).join('[REDACTED]');
  }
  if (credentials.secret) {
    message = message.split(credentials.secret).join('[REDACTED]');
  }
  return message;
}

/**
 * Parses CLI flags and runs the sandbox bot.
 * @returns {Promise<void>}
 */
async function main() {
  const { values } = parseArgs({
    options: {
      trade: { type: 'boolean', default: false },
      verbose: { type: 'boolean', default: false },
      config: { type: 'string' },
    },
    allowPositionals: true,
  });

  let credentials;
  try {
    credentials = await loadCredentials(values.config);
  } catch (err) {
    console.error(formatError(err));
    process.exitCode = err && typeof err === 'object' && err.code === 'CONFIG_MISSING' ? 2 : 1;
    return;
  }

  try {
    const exchange = createExchange(credentials, values.verbose === true);
    await runBot(exchange, { trade: values.trade === true });
  } catch (err) {
    console.error(formatError(err, credentials));
    process.exitCode = err && typeof err === 'object' && err.code === 'NOTIONAL_REFUSED' ? 2 : 1;
  }
}

const entry = process.argv[1] ? pathToFileURL(process.argv[1]).href : '';
if (import.meta.url === entry) {
  await main();
}
