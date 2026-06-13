export class AppError extends Error {
  constructor(
    public readonly kind: 'domain' | 'not_found' | 'conflict' | 'validation' | 'storage',
    message: string,
  ) {
    super(message);
    this.name = 'AppError';
  }

  static domain(msg: string): AppError {
    return new AppError('domain', msg);
  }

  static notFound(entity: string, id: string): AppError {
    return new AppError('not_found', `not found: ${entity} ${id}`);
  }

  static conflict(msg: string): AppError {
    return new AppError('conflict', msg);
  }

  static validation(msg: string): AppError {
    return new AppError('validation', msg);
  }

  static storage(msg: string): AppError {
    return new AppError('storage', msg);
  }
}

export type AppResult<T> = T | AppError;
