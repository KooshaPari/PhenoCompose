import { Result, ResultAsync, err, fromThrowable as ntFromThrowable, ok } from "neverthrow";

export type PhenoErrorCode =
  | "UNKNOWN"
  | "INVALID_ARGUMENT"
  | "INVALID_STATE"
  | "IO_ERROR"
  | "PARSE_ERROR";

export interface PhenoError {
  readonly code: PhenoErrorCode;
  readonly message: string;
  readonly details?: Readonly<Record<string, unknown>>;
  readonly cause?: unknown;
}

export interface CreatePhenoErrorOptions {
  readonly code?: PhenoErrorCode;
  readonly details?: Readonly<Record<string, unknown>>;
  readonly cause?: unknown;
}

export function createPhenoError(
  message: string,
  options: CreatePhenoErrorOptions = {},
): PhenoError {
  return {
    code: options.code ?? "UNKNOWN",
    message,
    ...(options.details ? { details: options.details } : {}),
    ...(Object.prototype.hasOwnProperty.call(options, "cause") ? { cause: options.cause } : {}),
  };
}

export function toPhenoError(
  error: unknown,
  fallback: CreatePhenoErrorOptions = {},
): PhenoError {
  const base = {
    ...(fallback.code ? { code: fallback.code } : {}),
    ...(fallback.details ? { details: fallback.details } : {}),
  } satisfies CreatePhenoErrorOptions;

  if (isPhenoError(error)) {
    return error;
  }

  if (error instanceof Error) {
    return createPhenoError(error.message, { ...base, cause: error });
  }

  if (typeof error === "string") {
    return createPhenoError(error, { ...base, cause: error });
  }

  return createPhenoError("Unknown error", { ...base, cause: error });
}

export function fromThrowable<TArgs extends readonly unknown[], TResult>(
  fn: (...args: TArgs) => TResult,
  fallback: CreatePhenoErrorOptions = {},
): (...args: TArgs) => Result<TResult, PhenoError> {
  return ntFromThrowable(fn, (error: unknown) => toPhenoError(error, fallback));
}

export function fromPromise<TResult>(
  promise: Promise<TResult>,
  fallback: CreatePhenoErrorOptions = {},
): ResultAsync<TResult, PhenoError> {
  return ResultAsync.fromPromise(promise, (error: unknown) => toPhenoError(error, fallback));
}

export function isPhenoError(value: unknown): value is PhenoError {
  return typeof value === "object" && value !== null && "code" in value && "message" in value;
}

export { Result, ResultAsync, err, ok };
