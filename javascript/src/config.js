import { readFile } from 'node:fs/promises';

/**
 * @typedef {object} Credentials
 * @property {string} apiKey
 * @property {string} secret
 */

/**
 * Loads Indodax credentials from a JSON file, then applies non-empty environment overrides.
 * @param {string} [configPath] Path to the JSON file. Defaults to config.json in the cwd.
 * @returns {Promise<Credentials>}
 * @throws {Error} When a required credential is missing after file and env merge.
 */
export async function loadCredentials(configPath = 'config.json') {
  let fileApiKey = '';
  let fileSecret = '';

  try {
    const raw = await readFile(configPath, 'utf8');
    const parsed = JSON.parse(raw);
    if (typeof parsed.apiKey === 'string') {
      fileApiKey = parsed.apiKey;
    }
    if (typeof parsed.secret === 'string') {
      fileSecret = parsed.secret;
    }
  } catch (err) {
    if (err && typeof err === 'object' && 'code' in err && err.code !== 'ENOENT') {
      throw err;
    }
  }

  const envApiKey = process.env.INDODAX_APIKEY ?? '';
  const envSecret = process.env.INDODAX_SECRET ?? '';

  const apiKey = envApiKey !== '' ? envApiKey : fileApiKey;
  const secret = envSecret !== '' ? envSecret : fileSecret;

  if (apiKey === '') {
    const error = new Error('missing INDODAX_APIKEY');
    error.code = 'CONFIG_MISSING';
    throw error;
  }
  if (secret === '') {
    const error = new Error('missing INDODAX_SECRET');
    error.code = 'CONFIG_MISSING';
    throw error;
  }

  return { apiKey, secret };
}
