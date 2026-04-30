/**
 * Integration tests for the blog post's central claim:
 *
 *   "Your unit tests prove your code handles 503s. Your code does NOT
 *    handle the failure modes that actually take down production."
 *
 * Each test below corresponds to a route in routes/routes.yaml and a
 * section of the blog post.
 *
 * --- A note on rigor ---
 *
 * An earlier draft of these tests used loose assertions like
 * ".rejects.toThrow(/failed/)". Those passed for any failure mode — including
 * "Bamboozle wasn't even running and the request 404'd." That's not a test;
 * that's a placebo.
 *
 * This version is stricter in two ways:
 *
 *   1. Each test PROBES the relevant Bamboozle route directly with `fetch`
 *      first, asserting that Bamboozle is in fact injecting the expected
 *      fault. If the probe fails, the test fails at the probe — with a
 *      message that points at the real problem (route mis-loaded, fault
 *      type wrong, etc.) instead of a misleading downstream symptom.
 *
 *   2. naiveFetch and robustFetch throw FetchError with a `kind`
 *      discriminator. Tests assert on `kind` (and `cause` where useful),
 *      not on regex matches against `message`. A network-layer failure
 *      produces `kind: "network"`; an empty-body failure produces
 *      `kind: "invalid_body"`. They cannot be confused.
 *
 * Run against a live Bamboozle container:
 *
 *     npm run bamboozle:up
 *     npm test
 *     npm run bamboozle:down
 */

import { beforeAll, beforeEach, describe, expect, it } from "vitest";
import {
  FetchError,
  naiveFetch,
  robustFetch,
} from "../src/payment-client.js";

const MOCK_URL = "http://localhost:8080"; // service-under-test calls this
const CONTROL_URL = "http://localhost:9090"; // tests configure & assert via this

// ---------------------------------------------------------------------------
// Control-API helpers (raw HTTP per docs/how-to/manage-routes.md
// and docs/how-to/assert-calls.md).
// ---------------------------------------------------------------------------

interface RouteDef {
  match: { verb: string; pattern: string };
  response: {
    status: string;
    content?: string;
    headers?: Record<string, string>;
  };
  simulation?: {
    delay?: {
      type: string;
      ms?: number;
      minMs?: number;
      maxMs?: number;
      meanMs?: number;
      stdDevMs?: number;
    };
    fault?: { type: string; probability?: number };
  };
}

async function putRoute(route: RouteDef): Promise<void> {
  const res = await fetch(`${CONTROL_URL}/control/routes`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(route),
  });
  if (!res.ok) {
    throw new Error(
      `failed to register route ${route.match.verb} ${route.match.pattern}: ${res.status} ${await res.text()}`,
    );
  }
}

function encodePattern(pattern: string): string {
  const trimmed = pattern.startsWith("/") ? pattern.slice(1) : pattern;
  return encodeURIComponent(trimmed);
}

async function clearCalls(verb: string, pattern: string): Promise<void> {
  await fetch(
    `${CONTROL_URL}/control/routes/${verb}/${encodePattern(pattern)}/calls`,
    { method: "DELETE" },
  );
}

async function assertCalledExactly(
  verb: string,
  pattern: string,
  n: number,
): Promise<void> {
  const url = `${CONTROL_URL}/control/routes/${verb}/${encodePattern(pattern)}/assert?called_exactly=${n}`;
  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: "{}",
  });
  if (res.status === 200) return;
  if (res.status === 406) {
    const unmatched = await fetch(`${CONTROL_URL}/control/unmatched`)
      .then((r) => r.text())
      .catch(() => "<could not fetch unmatched>");
    throw new Error(
      `assertion failed: ${verb} ${pattern} was not called exactly ${n} times. ` +
        `Unmatched requests on the mock listener: ${unmatched}`,
    );
  }
  throw new Error(
    `unexpected status ${res.status} from assert endpoint: ${await res.text()}`,
  );
}

// ---------------------------------------------------------------------------
// Probe helpers — confirm Bamboozle is actually injecting the fault we
// expect, BEFORE we test the client against it. If the probe fails, we
// know the test setup is broken; the client behavior tests below it would
// be meaningless.
// ---------------------------------------------------------------------------

/**
 * Probe a route once with raw fetch and return what happened at the
 * transport / HTTP layer — without any retry logic in the way.
 */
type ProbeResult =
  | { kind: "ok"; status: number; bodyLength: number; body: string }
  | { kind: "network_error"; cause: unknown };

async function probe(method: string, url: string): Promise<ProbeResult> {
  try {
    const res = await fetch(url, { method });
    const body = await res.text();
    return { kind: "ok", status: res.status, bodyLength: body.length, body };
  } catch (err) {
    return { kind: "network_error", cause: err };
  }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("blog post: 'Simulating faults your unit tests can't catch'", () => {
  beforeAll(async () => {
    // Sanity check: is Bamboozle reachable?
    try {
      const res = await fetch(`${CONTROL_URL}/control/routes`);
      if (!res.ok) {
        throw new Error(`control API returned ${res.status}`);
      }
    } catch (err) {
      throw new Error(
        `Bamboozle is not reachable at ${CONTROL_URL}: ${String(err)}. ` +
          `Did you run \`npm run bamboozle:up\`?`,
      );
    }
  });

  // -------------------------------------------------------------------------
  // Section: control case
  // -------------------------------------------------------------------------
  describe("the happy path (control)", () => {
    it("Bamboozle returns a real JSON body on /happy-path", async () => {
      // Probe first: prove the route exists and returns what we expect.
      const result = await probe("GET", `${MOCK_URL}/happy-path`);
      expect(result.kind).toBe("ok");
      if (result.kind !== "ok") return;
      expect(result.status).toBe(200);
      expect(JSON.parse(result.body)).toEqual({
        id: "abc-123",
        amount: 4200,
        status: "ok",
      });
    });

    it("both clients work fine when the dependency behaves", async () => {
      const naive = await naiveFetch(MOCK_URL, "/happy-path");
      const robust = await robustFetch(MOCK_URL, "/happy-path");
      expect(naive).toEqual({ id: "abc-123", amount: 4200, status: "ok" });
      expect(robust).toEqual({ id: "abc-123", amount: 4200, status: "ok" });
    });
  });

  // -------------------------------------------------------------------------
  // Section: "A 503 is the easy case"
  //
  // Both clients pass this. This is what your existing unit tests prove.
  // -------------------------------------------------------------------------
  describe("a 503 (the easy case)", () => {
    it("Bamboozle returns 503 on /server-error", async () => {
      const result = await probe("GET", `${MOCK_URL}/server-error`);
      expect(result.kind).toBe("ok");
      if (result.kind !== "ok") return;
      expect(result.status).toBe(503);
    });

    it("naiveFetch retries 5xx and reports the last status", async () => {
      const err = await naiveFetch(MOCK_URL, "/server-error").catch((e) => e);
      expect(err).toBeInstanceOf(FetchError);
      expect((err as FetchError).kind).toBe("retries_exhausted");
      expect((err as FetchError).lastStatus).toBe(503);
    });

    it("robustFetch reports server_error after exhausting retries", async () => {
      const err = await robustFetch(MOCK_URL, "/server-error").catch((e) => e);
      expect(err).toBeInstanceOf(FetchError);
      expect((err as FetchError).kind).toBe("server_error");
      expect((err as FetchError).lastStatus).toBe(503);
    });
  });

  // -------------------------------------------------------------------------
  // Section: "The TCP reset that took down the service"
  //
  // The headline scenario. We register the route at runtime and then probe
  // it with raw fetch FIRST to confirm Bamboozle is actually resetting the
  // connection. Without that probe, a misconfigured route returning 404
  // would also make the naive client throw — but for the wrong reason.
  // -------------------------------------------------------------------------
  describe("a TCP connection reset", () => {
    beforeAll(async () => {
      await putRoute({
        match: { verb: "GET", pattern: "/reset-me" },
        response: { status: "200" },
        simulation: { fault: { type: "connectionReset" } },
      });
    });

    it("Bamboozle actually resets the connection on /reset-me", async () => {
      // The crucial probe. Raw fetch should fail at the TRANSPORT layer,
      // not return a 404 or 500 status. If this assertion fails, the
      // Bamboozle route isn't doing what we claimed it does — and every
      // test below would be testing something else.
      const result = await probe("GET", `${MOCK_URL}/reset-me`);
      if (result.kind === "ok") {
        throw new Error(
          `expected /reset-me to fail at the network layer, but got ` +
            `HTTP ${result.status} (body: ${JSON.stringify(result.body.slice(0, 200))}). ` +
            `Bamboozle's connectionReset fault is not being applied — check the route registration.`,
        );
      }
      expect(result.kind).toBe("network_error");
      // Note we don't pin the exact error code here — the connectionReset
      // fault legitimately surfaces as ECONNRESET, "socket hang up", or
      // "fetch failed" depending on Node version and runtime. The CLAIM is
      // that it's a transport-layer failure, not a status code; the kind
      // discriminator captures that.
    });

    it("naiveFetch leaks a network-layer exception (not a FetchError)", async () => {
      // The blog post's central failure mode: naiveFetch's retry loop
      // doesn't catch network errors at all. The exception that escapes
      // is therefore NOT our structured FetchError — it's a raw
      // AxiosError or similar from the HTTP client. That's the bug.
      const err = await naiveFetch(MOCK_URL, "/reset-me").catch((e) => e);
      expect(err).toBeInstanceOf(Error);
      // Specifically, it is NOT a FetchError. If it were, naiveFetch
      // would have been handling network errors, contradicting the
      // whole premise.
      expect(err).not.toBeInstanceOf(FetchError);
      // And the message reflects a transport-layer failure, not an HTTP one.
      expect((err as Error).message).toMatch(
        /ECONNRESET|socket hang up|aborted|read ECONN|fetch failed/i,
      );
    });

    it("robustFetch catches the reset and reports kind=network", async () => {
      const err = await robustFetch(MOCK_URL, "/reset-me").catch((e) => e);
      expect(err).toBeInstanceOf(FetchError);
      // The kind is what proves robustFetch caught a TRANSPORT-layer
      // failure (vs. an empty body, vs. a 5xx, vs. a timeout).
      expect((err as FetchError).kind).toBe("network");
    });
  });

  // -------------------------------------------------------------------------
  // Section: "Faults that mirror reality, not fixtures" — empty response
  // -------------------------------------------------------------------------
  describe("an empty 200 response", () => {
    it("Bamboozle returns 200 with an empty body on /empty-response", async () => {
      const result = await probe("GET", `${MOCK_URL}/empty-response`);
      expect(result.kind).toBe("ok");
      if (result.kind !== "ok") return;
      expect(result.status).toBe(200);
      expect(result.bodyLength).toBe(0);
    });

    it("naiveFetch returns the parsed empty body without complaint", async () => {
      // The bug: naiveFetch trusted the 2xx status code and returned
      // whatever axios parsed the empty body to. We assert that what
      // came back is NOT a valid PaymentResult — i.e., naiveFetch did
      // NOT raise the alarm even though the data is unusable.
      const result = await naiveFetch(MOCK_URL, "/empty-response");

      // Stronger than "not equal to the happy-path payload": prove the
      // returned value is missing the fields a real PaymentResult has.
      // axios on an empty body with no Content-Type generally returns ""
      // (a string) or null. Either way, it's not the shape we'd persist.
      const looksValid =
        typeof result === "object" &&
        result !== null &&
        typeof (result as Partial<typeof result>).id === "string" &&
        typeof (result as Partial<typeof result>).amount === "number";

      expect(looksValid).toBe(false);
      // naiveFetch returned successfully — that IS the silent failure.
      // No exception was raised; the caller has no idea anything is wrong.
    });

    it("robustFetch validates the body and reports kind=invalid_body", async () => {
      const err = await robustFetch(MOCK_URL, "/empty-response").catch(
        (e) => e,
      );
      expect(err).toBeInstanceOf(FetchError);
      expect((err as FetchError).kind).toBe("invalid_body");
    });
  });

  // -------------------------------------------------------------------------
  // Section: "Transient faults"
  //
  // Probe the route with N raw fetches first; assert a meaningful fraction
  // failed at the transport layer. THEN run the clients and check that
  // the success-rate gap matches what the retry policy should produce.
  // -------------------------------------------------------------------------
  describe("transient connection resets (~30% probability)", () => {
    it("Bamboozle injects faults on a meaningful fraction of /flaky calls", async () => {
      const N = 50;
      let networkErrors = 0;
      let okResponses = 0;
      for (let i = 0; i < N; i++) {
        const result = await probe("GET", `${MOCK_URL}/flaky`);
        if (result.kind === "network_error") networkErrors++;
        else if (result.status === 200) okResponses++;
      }
      // Expected fault rate is 0.3 with N=50 — std dev ≈ 0.065. Loose
      // bounds (0.10 / 0.55) reliably contain the binomial distribution
      // while still catching "fault never fires" (0%) and "fault always
      // fires" (100%) — both of which would invalidate the tests below.
      expect(networkErrors / N).toBeGreaterThan(0.1);
      expect(networkErrors / N).toBeLessThan(0.55);
      expect(okResponses + networkErrors).toBe(N); // nothing weirder happened
    });

    it("robustFetch's success rate is high — retries hide most faults", async () => {
      const N = 50;
      let successes = 0;
      let failures = 0;
      let networkFailures = 0;
      for (let i = 0; i < N; i++) {
        try {
          await robustFetch(MOCK_URL, "/flaky", { maxAttempts: 3 });
          successes++;
        } catch (err) {
          failures++;
          // When robustFetch DOES fail, prove it failed for the right
          // reason. If we saw a server_error or invalid_body here,
          // something else is wrong with the test setup.
          if (err instanceof FetchError && err.kind === "network") {
            networkFailures++;
          }
        }
      }
      // p=0.3, 3 attempts → P(all fail) ≈ 0.027 → ~97% success expected.
      expect(successes / N).toBeGreaterThan(0.85);
      // Whatever failures occurred should be network-shaped.
      expect(networkFailures).toBe(failures);
    });

    it("naiveFetch's success rate matches the raw fault rate", async () => {
      const N = 50;
      let successes = 0;
      let nonNetworkFailures = 0;
      for (let i = 0; i < N; i++) {
        try {
          await naiveFetch(MOCK_URL, "/flaky", { maxAttempts: 3 });
          successes++;
        } catch (err) {
          // naiveFetch leaks raw exceptions on network errors. If we
          // see a FetchError here, something else broke (the route
          // started returning 5xx, etc.).
          if (err instanceof FetchError) nonNetworkFailures++;
        }
      }
      // naiveFetch doesn't retry network errors, so success rate hovers
      // around 1 - 0.3 = 0.7. The bound below catches "naive client
      // accidentally became robust" — both bounds matter.
      expect(successes / N).toBeLessThan(0.85);
      expect(successes / N).toBeGreaterThan(0.45);
      expect(nonNetworkFailures).toBe(0);
    });
  });

  // -------------------------------------------------------------------------
  // Section: "Latency injection"
  //
  // We don't probe latency directly — measuring the distribution would
  // need a hundred samples and complicate the test. Instead, the assertions
  // are sharp on cause: a tight timeout produces kind=timeout, a generous
  // one produces a successful payload. If Bamboozle weren't injecting
  // latency, the tight-timeout test would still need ~50ms of round-trip
  // to fail, and on localhost it should not.
  // -------------------------------------------------------------------------
  describe("latency injection", () => {
    it("a too-tight timeout fails with kind=timeout", async () => {
      const err = await robustFetch(MOCK_URL, "/slow", {
        timeoutMs: 50,
        maxAttempts: 1,
      }).catch((e) => e);
      expect(err).toBeInstanceOf(FetchError);
      // The kind discriminates timeout from network — a localhost socket
      // failure that wasn't actually a timeout would show as "network",
      // and this assertion would (correctly) fail.
      expect((err as FetchError).kind).toBe("timeout");
    });

    it("a generous timeout lets the request through with a real payload", async () => {
      const result = await robustFetch(MOCK_URL, "/slow", {
        timeoutMs: 2000,
        maxAttempts: 1,
      });
      // Sharp shape check, not just `.status === "ok"` — proves the
      // request actually completed and returned the expected JSON.
      expect(result).toEqual({
        id: "abc-123",
        amount: 4200,
        status: "ok",
      });
    });
  });

  // -------------------------------------------------------------------------
  // Section: assertions via the control API
  //
  // (Per-route history clear via DELETE — does NOT use POST /control/reset,
  // which would also wipe the routes loaded from routes.yaml.)
  // -------------------------------------------------------------------------
  describe("asserting on calls via the control API", () => {
    beforeEach(async () => {
      await clearCalls("GET", "/happy-path");
    });

    it("counts how many times /happy-path was called", async () => {
      await naiveFetch(MOCK_URL, "/happy-path");
      await naiveFetch(MOCK_URL, "/happy-path");
      await naiveFetch(MOCK_URL, "/happy-path");

      await assertCalledExactly("GET", "/happy-path", 3);
    });
  });
});
