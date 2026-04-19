import { test, expect, APIRequestContext } from '@playwright/test';
import { BamboozleClient, MatchKey, BamboozleAssertBuilder } from '@bamboozle/sdk';

const bamboozleClient: BamboozleClient = new BamboozleClient({ baseUrl: "http://localhost:19090" });
let deleteState: MatchKey[] = [];


test.afterEach(async () => {
  for (let key of deleteState) {
    try {
      await bamboozleClient.clearCalls(key.verb, key.pattern);
      await bamboozleClient.deleteRoute(key.verb, key.pattern);
    }
    catch { }
  }
  deleteState = [];
})

test.describe('assert header query match', () => {
  const verbs: string[] = ['GET', 'PUT', 'POST', 'PATCH', 'DELETE'];
  for (let verb of verbs) {
    test(verb, async ({ request }) => {
      const key: MatchKey = { verb: verb, pattern: 'playwright/assert/header/query/match' };
      deleteState.push(key);
      await addRoute(key);
      const initReq = await reqFactory(verb, `http://localhost:18080/${key.pattern}?queryParam1=true&queryParam2=false`, request);
      const init = await initReq.json();
      expect(init).toBeDefined();
      expect(init).toHaveLength(2);
      const assertReq = await bamboozleClient.assert(key.verb, key.pattern, {
        expression: new BamboozleAssertBuilder()
          .with(({ header }) => header.connection.equals('keep-alive'))
          .and()
          .with(({ query }) => query.queryParam1.equals('true'))
      });
      expect(assertReq).toBeTruthy();
    });
  }
});

test.describe('assert header query nonmatch', () => {
  const verbs: string[] = ['GET', 'PUT', 'POST', 'PATCH', 'DELETE'];
  for (let verb of verbs) {
    test(verb, async ({ request }) => {
      const key: MatchKey = { verb: verb, pattern: 'playwright/assert/header/query/nonmatch' };
      deleteState.push(key);
      await addRoute(key);
      const initReq = await reqFactory(verb, `http://localhost:18080/${key.pattern}?queryParam2=false`, request);
      const init = await initReq.json();
      expect(init).toBeDefined();
      expect(init).toHaveLength(1);
      const assertReq = await bamboozleClient.assert(key.verb, key.pattern, {
        expression: new BamboozleAssertBuilder()
          .with(({ header }) => header.connection.equals('keep-alive'))
          .and()
          .with(({ query }) => query.queryParam1.equals('true'))
      });
      expect(assertReq).toBeFalsy();
    });
  }
});

test.describe('assert route match', () => {
  const verbs: string[] = ['GET', 'PUT', 'POST', 'PATCH', 'DELETE'];
  for (let verb of verbs) {
    test(verb, async ({ request }) => {
      const key: MatchKey = { verb: verb, pattern: 'playwright/assert/route/{param1}/match' };
      deleteState.push(key);
      await addRoute(key);
      const route: string = handleRouteParams(key.pattern, [["{param1}", "test"]]);
      const initReq = await reqFactory(verb, `http://localhost:18080/${route}?queryParam1=true&queryParam2=false`, request);
      const init = await initReq.json();
      expect(init).toBeDefined();
      expect(init).toHaveLength(2);
      const assertReq = await bamboozleClient.assert(key.verb, key.pattern, {
        expression: new BamboozleAssertBuilder()
          .with(({ route }) => route.param1.equals('test'))
      });
      expect(assertReq).toBeTruthy();
    });
  }
});

test.describe('assert route nonmatch', () => {
  const verbs: string[] = ['GET', 'PUT', 'POST', 'PATCH', 'DELETE'];
  for (let verb of verbs) {
    test(verb, async ({ request }) => {
      const key: MatchKey = { verb: verb, pattern: 'playwright/assert/route/{param1}/nonmatch/{param2}' };
      deleteState.push(key);
      await addRoute(key);
      const route: string = handleRouteParams(key.pattern, [["{param1}", "test"], ["{param2}", "test2"]]);
      const initReq = await reqFactory(verb, `http://localhost:18080/${route}?queryParam1=true&queryParam2=false`, request);
      const init = await initReq.json();
      expect(init).toBeDefined();
      expect(init).toHaveLength(2);
      const assertReq = await bamboozleClient.assert(key.verb, key.pattern, {
        expression: new BamboozleAssertBuilder()
          .with(({ route }) => route.param1.equals('test2'))
      });
      expect(assertReq).toBeFalsy();
    });
  }
});

test.describe('assert route header query match', () => {
  const verbs: string[] = ['GET', 'PUT', 'POST', 'PATCH', 'DELETE'];
  for (let verb of verbs) {
    test(verb, async ({ request }) => {
      const key: MatchKey = { verb: verb, pattern: 'playwright/assert/route/{param1}/header/query/match' };
      deleteState.push(key);
      await addRoute(key);
      const route: string = handleRouteParams(key.pattern, [["{param1}", "testRouteParam"]]);
      const initReq = await reqFactory(verb, `http://localhost:18080/${route}?queryParam1=true&queryParam2=false`, request);
      const init = await initReq.json();
      expect(init).toBeDefined();
      expect(init).toHaveLength(2);
      const assertReq = await bamboozleClient.assert(key.verb, key.pattern, {
        expression: new BamboozleAssertBuilder()
          .with(({ route }) => route.param1.equals('testRouteParam'))
          .and()
          .with(({ query }) => query.queryParam1.equals('true'))
          .and()
          .with(({ header }) => header['accept-encoding'].contains('gzip'))
          .and()
          .with(({ header }) => header.host.startsWith('localhost'))
          .and()
          .with(({ header }) => header.host.endsWith(':18080'))
      });
      expect(assertReq).toBeTruthy();
    });
  }
});

test.describe('assert match verb nonmatch pattern', () => {
  const verbs: string[] = ['GET', 'PUT', 'POST', 'PATCH', 'DELETE'];
  for (let verb of verbs) {
    test(verb, async ({ request }) => {
      const key: MatchKey = { verb: verb, pattern: 'playwright/assert/match/verb/nonmatch/pattern' };
      deleteState.push(key);
      await addRoute(key);
      const initReq = await reqFactory(verb, `http://localhost:18080/${key.pattern}`, request);
      const init = await initReq.json();
      expect(init).toBeDefined();
      expect(init).toHaveLength(0);
      const assertReq = await bamboozleClient.assert(key.verb, key.pattern, {
        expression: new BamboozleAssertBuilder()
          .with(({ context }) => context.verb.equals(verb))
          .or()
          .with(({ context }) => context.pattern.equals('wrong'))
      });
      expect(assertReq).toBeTruthy();
    });
  }
});

test.describe('assert nonmatch verb match pattern', () => {
  const verbs: string[] = ['GET', 'PUT', 'POST', 'PATCH', 'DELETE'];
  for (let verb of verbs) {
    test(verb, async ({ request }) => {
      const key: MatchKey = { verb: verb, pattern: 'playwright/assert/nonmatch/verb/match/pattern' };
      deleteState.push(key);
      await addRoute(key);
      const initReq = await reqFactory(verb, `http://localhost:18080/${key.pattern}`, request);
      const init = await initReq.json();
      expect(init).toBeDefined();
      expect(init).toHaveLength(0);
      const assertReq = await bamboozleClient.assert(key.verb, key.pattern, {
        expression: new BamboozleAssertBuilder()
          .with(({ context }) => context.verb.equals('wrong'))
          .or()
          .with(({ context }) => context.pattern.equals(key.pattern))
      });
      expect(assertReq).toBeTruthy();
    });
  }
});

test.describe('assert match with no expression', () => {
  const verbs: string[] = ['GET', 'PUT', 'POST', 'PATCH', 'DELETE'];
  for (let verb of verbs) {
    test(verb, async ({ request }) => {
      const key: MatchKey = { verb: verb, pattern: 'playwright/assert/match/with/no/expression' };
      deleteState.push(key);
      await addRoute(key);
      const initReq = await reqFactory(verb, `http://localhost:18080/${key.pattern}`, request);
      const init = await initReq.json();
      expect(init).toBeDefined();
      expect(init).toHaveLength(0);
      const assertReq = await bamboozleClient.assert(key.verb, key.pattern);
      expect(assertReq).toBeTruthy();
    });
  }
});

test.describe('assert match complex body', () => {
  const verbs: string[] = ['PUT', 'POST', 'PATCH'];
  for (let verb of verbs) {
    test(verb, async ({ request }) => {
      const key: MatchKey = { verb: verb, pattern: 'playwright/assert/match/complex/body' };
      deleteState.push(key);
      await addRoute(key);
      const initReq = await reqFactory(verb, `http://localhost:18080/${key.pattern}`, request, {
        hello: "world",
        number: 24
      });
      const init = await initReq.json();
      expect(init).toBeDefined();
      expect(init).toHaveLength(0);
      const assertReq = await bamboozleClient.assert(key.verb, key.pattern, {
        expression: new BamboozleAssertBuilder()
          .with(({ body }) => body.hello.equals("world"))
          .and()
          .with(({ body }) => body.number.equals(24))
          .and()
          .with(({ body }) => body.number.notEquals("24"))
          .and()
          .with(({ body }) => body.number.greaterThan(23))
          .and()
          .with(({ body }) => body.number.lessThan(25))
          .and()
          .with(({ body }) => body.number.greaterThan(3))
          .and()
          .with(({ body }) => body.number.lessThan(100))
      });
      expect(assertReq).toBeTruthy();
    });
  }
});

async function reqFactory(verb: string, location: string, request: APIRequestContext, body: any = {}) {
  if (verb === 'GET') {
    return await request.get(location);
  } else if (verb === 'PUT') {
    return await request.put(location, {
      data: body
    });
  } else if (verb === 'POST') {
    return await request.post(location, {
      data: body
    });
  } else if (verb === 'PATCH') {
    return await request.patch(location, {
      data: body
    });
  } else if (verb === 'DELETE') {
    return await request.delete(location);
  }
  throw new Error();
}

function handleRouteParams(template: string, matches: string[][]): string {
  let response: string = template;
  for (let match of matches) {
    response = response.replace(match[0], match[1]);
  }
  return response;
}

async function addRoute(key: MatchKey) {
  await bamboozleClient.addRoute({
    match: key,
    response: {
      status: "200",
      headers: {
        "Content-Type": "application/json"
      },
      content: `
        [
          {% for kvp in queryParams %} 
            "{{kvp[0]}}={{kvp[1]}}"{% unless forloop.last %}, {% endunless %}
          {% endfor %}
        ]
      `
    }
  });
}