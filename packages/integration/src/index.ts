import { loadFromEnv, ConfigError } from '@phenotype/pheno-config';
import { init, initJson, type TracingOptions } from '@phenotype/pheno-tracing';
import { AppError } from '@phenotype/pheno-errors';

export interface PhenoComposeConfig {
  name?: string;
  log_level?: string;
}

export class PhenoComposeApp {
  public config?: PhenoComposeConfig;
  public tracingInitialized = false;

  loadConfig(prefix = 'PHENOCOMPOSE_'): PhenoComposeConfig | AppError {
    try {
      this.config = loadFromEnv<PhenoComposeConfig>(prefix);
      return this.config;
    } catch (err) {
      if (err instanceof ConfigError) {
        return AppError.domain(err.message);
      }
      return AppError.domain(String(err));
    }
  }

  initializeTracing(options?: TracingOptions): void {
    init(options);
    this.tracingInitialized = true;
  }

  initializeTracingJson(): void {
    initJson();
    this.tracingInitialized = true;
  }
}

export { AppError, ConfigError, loadFromEnv, init, initJson };
export type { TracingOptions };
