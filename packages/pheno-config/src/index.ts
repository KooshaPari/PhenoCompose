export class ConfigError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ConfigError';
  }
}

export function loadFromEnv<T>(prefix: string): T {
  const upperPrefix = prefix.toUpperCase();
  const map: Record<string, unknown> = {};

  for (const [key, value] of Object.entries(process.env)) {
    if (key && key.startsWith(upperPrefix)) {
      const stripped = key.slice(upperPrefix.length).replace(/^_/, '');
      const lowerKey = stripped.toLowerCase();
      map[lowerKey] = parseValue(value);
    }
  }

  if (Object.keys(map).length === 0) {
    throw new ConfigError(`environment variable prefix not found: ${upperPrefix}`);
  }

  return map as T;
}

function parseValue(raw: string | undefined): unknown {
  if (raw === undefined) return null;
  try {
    const parsed = JSON.parse(raw);
    if (parsed !== null) return parsed;
  } catch { /* not JSON */ }

  if (raw.toLowerCase() === 'true') return true;
  if (raw.toLowerCase() === 'false') return false;
  if (/^-?\d+$/.test(raw)) return parseInt(raw, 10);
  if (/^-?\d+\.\d+$/.test(raw)) return parseFloat(raw);
  return raw;
}
