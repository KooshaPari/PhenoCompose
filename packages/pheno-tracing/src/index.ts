export interface TracingOptions {
  format?: 'pretty' | 'json';
  logDir?: string;
}

export function init(options?: TracingOptions): void {
  // Placeholder: tracing initialization
  console.log(`[pheno-tracing] init format=${options?.format ?? 'pretty'}`);
}

export function initJson(): void {
  init({ format: 'json' });
}

export function initWithFile(logDir: string): void {
  init({ format: 'pretty', logDir });
}
