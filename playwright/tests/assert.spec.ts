import { test, expect, APIRequestContext } from '@playwright/test';
import { BamboozleClient, MatchKey, BamboozleAssertBuilder, HeaderAssertion, Operator, QueryAssertion, RouteAssertion } from '@bamboozle/sdk';

const bamboozleClient: BamboozleClient = new BamboozleClient({ baseUrl: "http://localhost:19090" });
let deleteState: MatchKey[] = [];


test.afterEach(async () => {
  for (let key of deleteState) {
    await bamboozleClient.clearCalls(key.verb, key.pattern);
    await bamboozleClient.deleteRoute(key.verb, key.pattern);
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
          .with(new HeaderAssertion('connection', Operator.Equals, 'keep-alive'))
          .and()
          .with(new QueryAssertion('queryParam1', Operator.Equals, 'true'))
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
          .with(new HeaderAssertion('connection', Operator.Equals, 'keep-alive'))
          .and()
          .with(new QueryAssertion('queryParam1', Operator.Equals, 'true'))
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
          .with(new RouteAssertion('param1', Operator.Equals, 'test'))
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
          .with(new RouteAssertion('param1', Operator.Equals, 'test2'))
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
          .with(new RouteAssertion('param1', Operator.Equals, 'testRouteParam'))
          .and()
          .with(new QueryAssertion('queryParam1', Operator.Equals, 'true'))
          .and()
          .with(new HeaderAssertion('accept-encoding', Operator.Contains, 'gzip'))
          .and()
          .with(new HeaderAssertion('host', Operator.StartsWith, 'localhost'))
          .and()
          .with(new HeaderAssertion('host', Operator.EndsWith, ':18080'))
      });
      expect(assertReq).toBeTruthy();
    });
  }
});

async function reqFactory(verb: string, location: string, request: APIRequestContext) {
  if (verb === 'GET') {
    return await request.get(location);
  } else if (verb === 'PUT') {
    return await request.put(location);
  } else if (verb === 'POST') {
    return await request.post(location);
  } else if (verb === 'PATCH') {
    return await request.patch(location);
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