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
});
test('check for previous context', async ({ request }) => {
    const key: MatchKey = { verb: 'GET', pattern: 'playwright/check/for/previous/context' };
    deleteState.push(key);
    await bamboozleClient.addRoute({
        match: key,
        response: {
            status: "200",
            content: `{% if previousContext != null %}OK{% endif %}`
        }
    });
    const initReq = await request.get(`http://localhost:18080/${key.pattern}`);
    expect(await initReq.text()).toEqual("");

    const secondReq = await request.get(`http://localhost:18080/${key.pattern}`);
    expect(await secondReq.text()).toEqual("OK");
});

test('check for no previous previous context', async ({ request }) => {
    const key: MatchKey = { verb: 'GET', pattern: 'playwright/check/for/no/previous/previous/context' };
    deleteState.push(key);
    await bamboozleClient.addRoute({
        match: key,
        response: {
            status: "200",
            content: `{% if previousContext.previousContext != null %}BAD{% endif %}`
        }
    });
    //these are likely passing because of https://github.com/matt-andrews/Bamboozle/issues/12
    //once 12 is fixed we can probably do an assertion on real values instead of empty
    const initReq = await request.get(`http://localhost:18080/${key.pattern}`);
    expect(await initReq.text()).toEqual("");

    const secondReq = await request.get(`http://localhost:18080/${key.pattern}`);
    expect(await secondReq.text()).toEqual("");

    const thirdReq = await request.get(`http://localhost:18080/${key.pattern}`);
    expect(await thirdReq.text()).toEqual("");
});

test('test for state carry forward', async ({ request }) => {
    const key: MatchKey = { verb: 'GET', pattern: 'test/for/state/carry/forward' };
    deleteState.push(key);
    await bamboozleClient.addRoute({
        match: key,
        response: {
            status: "200",
            headers: {
                "my-state": '{% if previousContext == null %}0{% else %}'
                    + '{% if previousContext["routeModel"]["response"]["headers"]["my-state"] == null %}0{% else %}'
                    + '{% assign stateCount = previousContext["routeModel"]["response"]["headers"]["my-state"] %}{{ stateCount }}{% endif %}{% endif %}'
            },
            content: `OK`
        }
    });

    const initReq = await request.get(`http://localhost:18080/${key.pattern}`);
    expect(await initReq.text()).toEqual("OK");
    expect(initReq.headers()["my-state"]).toEqual('0');

    const secondReq = await request.get(`http://localhost:18080/${key.pattern}`);
    expect(await secondReq.text()).toEqual("OK");
    expect(secondReq.headers()["my-state"]).toEqual('1');

    const thirdReq = await request.get(`http://localhost:18080/${key.pattern}`);
    expect(await thirdReq.text()).toEqual("OK");
    expect(thirdReq.headers()["my-state"]).toEqual('2');
});