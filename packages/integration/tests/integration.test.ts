import { describe, it, expect } from 'vitest';
import { PhenoComposeApp, AppError } from '../src/index.js';

describe('PhenoComposeApp', () => {
  it('app_loads_config', () => {
    process.env.PHENOCOMPOSE_NAME = 'test-app';
    const app = new PhenoComposeApp();
    const result = app.loadConfig();
    expect(result).toEqual({ name: 'test-app' });
    delete process.env.PHENOCOMPOSE_NAME;
  });

  it('app_initializes_tracing', () => {
    const app = new PhenoComposeApp();
    app.initializeTracing();
    expect(app.tracingInitialized).toBe(true);
  });

  it('app_loads_config_returns_error_when_prefix_missing', () => {
    const app = new PhenoComposeApp();
    const result = app.loadConfig('MISSINGPREFIX_');
    expect(result).toBeInstanceOf(AppError);
    expect((result as AppError).kind).toBe('domain');
  });
});
