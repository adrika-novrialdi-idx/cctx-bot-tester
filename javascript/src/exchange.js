import ccxt from 'ccxt';

/**
 * @typedef {object} Credentials
 * @property {string} apiKey
 * @property {string} secret
 */

/**
 * Creates a CCXT Indodax client pointed at the d-idx.com sandbox hosts.
 * @param {Credentials} credentials
 * @param {boolean} [verbose=false]
 * @returns {import('ccxt').indodax}
 */
export function createExchange(credentials, verbose = false) {
  return new ccxt.indodax({
    apiKey: credentials.apiKey,
    secret: credentials.secret,
    enableRateLimit: true,
    verbose,
    urls: {
      api: {
        public: 'https://d-idx.com',
        private: 'https://d-idx.com/tapi',
      },
    },
    options: {
      tapiVersion: '2',
      sandboxUrl: 'https://api.d-idx.com',
    },
  });
}
