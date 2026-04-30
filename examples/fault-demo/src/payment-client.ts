/**
 * The service-under-test
 *
 * Two clients, deliberately different:
 *
 *   naiveFetch  — retries only on 5xx status codes. This is what most teams
 *                 write first. It passes unit tests that mock at the function
 *                 boundary because mocks always fail with status codes.
 *
 *   robustFetch — also retries on network-layer errors (ECONNRESET, EPIPE,
 *                 timeouts) and validates the response body. This is what
 *                 you end up writing AFTER an incident — or, ideally, after
 *                 running the tests in this repo.
 *
 * Both clients throw `FetchError` with a `kind` field so tests can assert
 * on the *cause* of failure, not just the message text. This matters: an
 * assertion like ".rejects.toThrow(/failed/)" passes for any failure mode,
 * including "Bamboozle wasn't running and the request 404'd." Tagged errors
 * make the tests prove what they claim to prove.
 */

import axios, { AxiosError, AxiosInstance } from "axios";

// ---------------------------------------------------------------------------
// Structured error type. `kind` is the discriminator the tests assert on.
// ---------------------------------------------------------------------------

export type FetchErrorKind =
  | "network" //  ECONNRESET, socket hang up, EPIPE — transport-layer fault
  | "timeout" //   client-side timeout fired
  | "server_error" // 5xx from upstream
  | "client_error" // 4xx from upstream (do not retry)
  | "invalid_body" // 2xx but body wasn't usable
  | "retries_exhausted"; // retry loop gave up; see `cause` for last reason

export class FetchError extends Error {
  constructor(
    public readonly kind: FetchErrorKind,
    message: string,
    public readonly cause?: unknown,
    public readonly lastStatus?: number,
  ) {
    super(message);
    this.name = "FetchError";
  }
}

export interface FetchOptions {
  maxAttempts?: number;
  timeoutMs?: number;
}

export interface PaymentResult {
  id: string;
  amount: number;
  status: string;
}

const DEFAULTS: Required<FetchOptions> = {
  maxAttempts: 3,
  timeoutMs: 1000,
};

function buildAxios(baseURL: string, timeoutMs: number): AxiosInstance {
  return axios.create({
    baseURL,
    timeout: timeoutMs,
    // Don't let axios throw on non-2xx — we want to inspect the status
    // ourselves and decide whether to retry.
    validateStatus: () => true,
  });
}

// ---------------------------------------------------------------------------
// naiveFetch — the "we handle 503s in our unit tests" version.
//
// Deliberately broken in two ways:
//   1. No try/catch around the request. Network-layer exceptions (ECONNRESET,
//      socket hang up, etc.) escape unwrapped — the caller sees an AxiosError
//      or a raw Error with no `kind`, which is the whole point.
//   2. Trusts the 2xx status code without validating the body. An empty
//      200 returns whatever axios parsed it to (`""`, `null`, `{}` depending
//      on Content-Type) cast to PaymentResult. Garbage in, garbage out.
// ---------------------------------------------------------------------------

export async function naiveFetch(
  baseURL: string,
  path: string,
  opts: FetchOptions = {},
): Promise<PaymentResult> {
  const { maxAttempts, timeoutMs } = { ...DEFAULTS, ...opts };
  const http = buildAxios(baseURL, timeoutMs);

  let lastStatus = 0;
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    // No try/catch on purpose — see header comment.
    const res = await http.get<PaymentResult>(path);

    if (res.status >= 200 && res.status < 300) {
      return res.data;
    }

    lastStatus = res.status;
    if (res.status < 500) break;
  }

  throw new FetchError(
    "retries_exhausted",
    `request failed after retries (last status ${lastStatus})`,
    undefined,
    lastStatus,
  );
}

// ---------------------------------------------------------------------------
// robustFetch — the version you wish you'd shipped the first time.
//
// Throws FetchError with a kind that names the *actual* cause of failure:
//   - "network"        for retry-exhausted ECONNRESET / EPIPE / etc.
//   - "timeout"        for retry-exhausted client-side timeouts
//   - "invalid_body"   for retry-exhausted empty-body responses
//   - "server_error"   for retry-exhausted 5xx
//   - "client_error"   for any 4xx (no retry)
// ---------------------------------------------------------------------------

export async function robustFetch(
  baseURL: string,
  path: string,
  opts: FetchOptions = {},
): Promise<PaymentResult> {
  const { maxAttempts, timeoutMs } = { ...DEFAULTS, ...opts };
  const http = buildAxios(baseURL, timeoutMs);

  // Track the most recent reason for failure so the final FetchError can
  // tell the caller WHY all attempts failed, not just that they did.
  let lastFailureKind: FetchErrorKind = "retries_exhausted";
  let lastCause: unknown;
  let lastStatus: number | undefined;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      const res = await http.get<PaymentResult>(path);

      if (res.status >= 500) {
        lastFailureKind = "server_error";
        lastStatus = res.status;
        lastCause = new Error(`server returned ${res.status}`);
        continue;
      }

      if (res.status >= 400) {
        // Don't retry — client errors are not transient.
        throw new FetchError(
          "client_error",
          `client error ${res.status}`,
          undefined,
          res.status,
        );
      }

      // 2xx — but is the body usable? Empty 200 from a JSON endpoint is
      // a fault, not a success.
      if (res.data == null || typeof res.data !== "object") {
        lastFailureKind = "invalid_body";
        lastStatus = res.status;
        lastCause = new Error("empty or non-object response body");
        continue;
      }

      return res.data;
    } catch (err) {
      // Don't swallow our own thrown FetchError (e.g. client_error above).
      if (err instanceof FetchError) throw err;

      const networkKind = classifyNetworkError(err);
      if (networkKind) {
        lastFailureKind = networkKind;
        lastCause = err;
        continue;
      }
      throw err;
    }
  }

  throw new FetchError(
    lastFailureKind,
    `request failed after ${maxAttempts} attempts: ${describeError(lastCause)}`,
    lastCause,
    lastStatus,
  );
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/**
 * Map a thrown error to a FetchErrorKind, or null if it isn't a recognized
 * network-layer error. Distinguishes timeouts from connection resets so the
 * tests can assert on the right one.
 */
function classifyNetworkError(err: unknown): FetchErrorKind | null {
  if (err instanceof AxiosError) {
    // axios uses ECONNABORTED for its own timeout; ETIMEDOUT for the OS
    // socket-level timeout. Both are timeouts from the caller's perspective.
    if (err.code === "ECONNABORTED" || err.code === "ETIMEDOUT") {
      return "timeout";
    }
    if (
      err.code === "ECONNRESET" ||
      err.code === "EPIPE" ||
      err.code === "ECONNREFUSED" ||
      err.code === "EHOSTUNREACH" ||
      err.code === "ENETUNREACH"
    ) {
      return "network";
    }
  }
  // Some Node / axios versions surface socket-hang-up as a plain Error
  // with a message rather than a code.
  if (err instanceof Error && /socket hang up|ECONNRESET/i.test(err.message)) {
    return "network";
  }
  return null;
}

function describeError(err: unknown): string {
  if (err instanceof AxiosError) {
    return `${err.code ?? "AxiosError"} ${err.message}`;
  }
  if (err instanceof Error) return err.message;
  return String(err);
}
