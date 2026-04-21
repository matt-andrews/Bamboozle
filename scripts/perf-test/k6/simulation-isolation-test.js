/**
 * Simulation isolation test.
 *
 * Verifies that a route configured with a long fixed delay does not bleed
 * latency into an unrelated route running concurrently on the same server.
 *
 * How it works:
 *   - "slow" route: 300 ms fixed delay, hammered by SLOW_VUS virtual users
 *   - "fast" route: no simulation, hammered by FAST_VUS virtual users in parallel
 *
 * Both scenarios run for the full duration simultaneously. If tokio's async
 * sleep bleeds into other tasks the fast route's p95 will creep toward DELAY_MS.
 * The threshold enforces that it stays well below it.
 */

import http from 'k6/http';
import { check, group } from 'k6';
import { Trend, Rate } from 'k6/metrics';

const CONTROL_URL = __ENV.CONTROL_URL || 'http://localhost:19090';
const BASE_URL     = __ENV.BASE_URL    || 'http://localhost:18080';

const SLOW_PATTERN = 'k6/simulation/slow';
const FAST_PATTERN = 'k6/simulation/fast';
const DELAY_MS     = 300;
const SLOW_VUS     = 5;
const FAST_VUS     = 5;

const fastDuration = new Trend('sim_fast_duration_ms', true);
const fastErrors   = new Rate('sim_fast_errors');
const slowErrors   = new Rate('sim_slow_errors');

export const options = {
  scenarios: {
    slow_route_load: {
      executor: 'constant-vus',
      vus:      SLOW_VUS,
      duration: '20s',
      exec:     'testSlowRoute',
    },
    fast_route_isolation: {
      executor: 'constant-vus',
      vus:      FAST_VUS,
      duration: '20s',
      exec:     'testFastRoute',
    },
  },
  thresholds: {
    // Fast route must not be affected by the slow route's delay.
    // p95 must stay well below DELAY_MS even while slow VUs are sleeping.
    'sim_fast_duration_ms': [`p(95)<${Math.floor(DELAY_MS / 3)}`],
    'sim_fast_errors':      ['rate<0.01'],
    'sim_slow_errors':      ['rate<0.01'],
  },
};

export function setup() {
  const headers = { 'Content-Type': 'application/json' };

  const slowRes = http.put(
    `${CONTROL_URL}/control/routes`,
    JSON.stringify({
      match:      { verb: 'GET', pattern: SLOW_PATTERN },
      response:   { status: '200', content: 'slow' },
      simulation: { delay: { type: 'fixed', ms: DELAY_MS } },
    }),
    { headers },
  );
  check(slowRes, { 'slow route registered': (r) => r.status < 300 });

  const fastRes = http.put(
    `${CONTROL_URL}/control/routes`,
    JSON.stringify({
      match:    { verb: 'GET', pattern: FAST_PATTERN },
      response: { status: '200', content: 'fast' },
    }),
    { headers },
  );
  check(fastRes, { 'fast route registered': (r) => r.status < 300 });
}

export function testSlowRoute() {
  group('slow_route', () => {
    const res = http.get(`${BASE_URL}/${SLOW_PATTERN}`);
    const ok = check(res, {
      'slow: status 200':     (r) => r.status === 200,
      'slow: delay applied':  (r) => r.timings.duration >= DELAY_MS,
    });
    slowErrors.add(!ok);
  });
}

export function testFastRoute() {
  group('fast_route', () => {
    const res = http.get(`${BASE_URL}/${FAST_PATTERN}`);
    const ok = check(res, {
      'fast: status 200':    (r) => r.status === 200,
      'fast: correct body':  (r) => r.body === 'fast',
    });
    fastDuration.add(res.timings.duration);
    fastErrors.add(!ok);
  });
}

export function teardown() {
  http.del(`${CONTROL_URL}/control/routes/GET/${encodeURIComponent(SLOW_PATTERN)}`);
  http.del(`${CONTROL_URL}/control/routes/GET/${encodeURIComponent(FAST_PATTERN)}`);
}
