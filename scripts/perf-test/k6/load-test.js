import http from 'k6/http';
import { check, group } from 'k6';
import { Rate } from 'k6/metrics';

const BASE_URL = __ENV.BASE_URL || 'http://localhost:18080';

const LOAD_PROFILE = { vus: 3, duration: '30s' };

const errorRateHello     = new Rate('errors_hello');
const errorRateSomething = new Rate('errors_something');
const errorRateMulti     = new Rate('errors_multi');

export const options = {
  scenarios: {
    hello: {
      executor: 'constant-vus', vus: LOAD_PROFILE.vus,
      duration: LOAD_PROFILE.duration, exec: 'testHello',
    },
    something: {
      executor: 'constant-vus', vus: LOAD_PROFILE.vus,
      duration: LOAD_PROFILE.duration, exec: 'testSomething',
    },
    multiRoute: {
      executor: 'constant-vus', vus: LOAD_PROFILE.vus,
      duration: LOAD_PROFILE.duration, exec: 'testMultiRoute',
    },
  },
  thresholds: {
    // Placeholders — never fail the build. To activate real thresholds:
    // replace 'rate<=1' with 'rate<0.01' and 'p(95)<999999' with 'p(95)<200'
    'http_req_failed{scenario:hello}':        ['rate<=1'],      // activate: rate<0.01
    'http_req_failed{scenario:something}':    ['rate<=1'],      // activate: rate<0.01
    'http_req_failed{scenario:multiRoute}':   ['rate<=1'],      // activate: rate<0.01
    'http_req_duration{scenario:hello}':      ['p(95)<999999'], // activate: p(95)<200
    'http_req_duration{scenario:something}':  ['p(95)<999999'], // activate: p(95)<200
    'http_req_duration{scenario:multiRoute}': ['p(95)<999999'], // activate: p(95)<200
    errors_hello:     ['rate<=1'], // activate: rate<0.01
    errors_something: ['rate<=1'], // activate: rate<0.01
    errors_multi:     ['rate<=1'], // activate: rate<0.01
  },
};

// Route 1: GET /test/hello?status=200 (test-config-1.yml)
export function testHello() {
  group('route1_hello', () => {
    const res = http.get(`${BASE_URL}/test/hello?status=200`);
    const ok = check(res, {
      'hello: status 200': (r) => r.status === 200,
      'hello: JSON array': (r) => { try { return Array.isArray(JSON.parse(r.body)); } catch { return false; } },
    });
    errorRateHello.add(!ok);
  });
}

// Route 2: GET /something/{name}/{version} (test-config-2.json)
export function testSomething() {
  group('route2_something', () => {
    const res = http.get(`${BASE_URL}/something/widget/v1`);
    const ok = check(res, {
      'something: status 200':  (r) => r.status === 200,
      'something: id field':    (r) => { try { return JSON.parse(r.body).id === 'widget'; } catch { return false; } },
      'something: value field': (r) => { try { return JSON.parse(r.body).value === 'test-value-widget'; } catch { return false; } },
    });
    errorRateSomething.add(!ok);
  });
}

// Route 3: GET /multi-route/{p1?}/{p2?}/{p3?} (test-config-3.yaml)
// Tests full path (3 parts → 200) and single part (1 part → 200)
export function testMultiRoute() {
  group('route3_multi_all', () => {
    const ok = check(http.get(`${BASE_URL}/multi-route/item1/item2/item3`), {
      'multi (all): status 200': (r) => r.status === 200,
    });
    errorRateMulti.add(!ok);
  });
  group('route3_multi_one', () => {
    const ok = check(http.get(`${BASE_URL}/multi-route/item1`), {
      'multi (one): status 200': (r) => r.status === 200,
    });
    errorRateMulti.add(!ok);
  });
}
