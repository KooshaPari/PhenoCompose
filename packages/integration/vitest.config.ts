import { defineConfig } from 'vitest/config';
import { resolve } from 'path';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    include: ['tests/**/*.test.ts', 'tests/**/*.spec.ts'],
    testTimeout: 30000,
  },
  resolve: {
    alias: {
      '@phenotype/pheno-errors': resolve(__dirname, '../pheno-errors/src/index.ts'),
      '@phenotype/pheno-tracing': resolve(__dirname, '../pheno-tracing/src/index.ts'),
      '@phenotype/pheno-config': resolve(__dirname, '../pheno-config/src/index.ts'),
    },
  },
});
