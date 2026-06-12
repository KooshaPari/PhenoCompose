import { describe, expect, it } from "vitest";

import { createPhenoError, fromPromise, fromThrowable, isPhenoError, toPhenoError } from "../src/index.js";

describe("pheno-error", () => {
  it("wraps successful sync functions", () => {
    const wrapped = fromThrowable((input: number) => input * 2);

    const result = wrapped(21);

    expect(result.isOk()).toBe(true);
    expect(result._unsafeUnwrap()).toBe(42);
  });

  it("wraps failing sync functions", () => {
    const wrapped = fromThrowable(() => {
      throw new Error("boom");
    });

    const result = wrapped();

    expect(result.isErr()).toBe(true);
    expect(result._unsafeUnwrapErr()).toMatchObject({
      code: "UNKNOWN",
      message: "boom",
    });
  });

  it("wraps successful promises", async () => {
    const result = await fromPromise(Promise.resolve("ok"));

    expect(result.isOk()).toBe(true);
    expect(result._unsafeUnwrap()).toBe("ok");
  });

  it("wraps rejected promises", async () => {
    const result = await fromPromise(Promise.reject("badness"), {
      code: "IO_ERROR",
    });

    expect(result.isErr()).toBe(true);
    expect(result._unsafeUnwrapErr()).toMatchObject({
      code: "IO_ERROR",
      message: "badness",
    });
  });

  it("preserves shaped PhenoError objects", () => {
    const original = createPhenoError("configured", { code: "INVALID_STATE" });

    expect(toPhenoError(original)).toBe(original);
    expect(isPhenoError(original)).toBe(true);
  });

  it("normalizes arbitrary objects", () => {
    const normalized = toPhenoError({ reason: "opaque" }, { code: "PARSE_ERROR" });

    expect(normalized).toMatchObject({
      code: "PARSE_ERROR",
      message: "Unknown error",
      cause: { reason: "opaque" },
    });
  });
});

