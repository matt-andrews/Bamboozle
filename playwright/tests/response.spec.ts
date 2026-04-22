import { test, expect, APIRequestContext } from '@playwright/test';
import { BamboozleClient, MatchKey, BamboozleAssertBuilder } from '@bamboozle/sdk';

const bamboozleClient: BamboozleClient = new BamboozleClient({ baseUrl: "http://localhost:19090" });

test('response json is real json', async ({ request }) => {
    const response = await request.get('http://localhost:18080/something/something/darkside');
    const json = await response.json();
    expect(json.id).toEqual("something");
    expect(json.value).toEqual("test-value-something")
})

test.describe('request body can do complex json', () => {
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
    const verbs: string[] = ['PUT', 'POST', 'patch'];
    for (let verb of verbs) {
        test(verb, async ({ request }) => {
            const key: MatchKey = { verb: verb, pattern: 'playwright/request/body/can/do/complex/json' };
            deleteState.push(key);
            verb = verb.toUpperCase();
            await bamboozleClient.addRoute({
                match: key,
                response: {
                    status: "200",
                    headers: {
                        "Content-Type": "application/json"
                    },
                    content: `
                    {
                        "hello": "{{body.hello}}",
                        "number": "{{body.number}}"
                    }
                `
                }
            });
            const initReq = await reqFactory(verb, `http://localhost:18080/${key.pattern}`, request, {
                hello: "world",
                number: 24
            });
            const init = await initReq.json();
            expect(init).toBeDefined();
            expect(init.hello).toEqual("world");
            expect(init.number).toEqual("24"); //in the LT content we convert to string
            expect(await bamboozleClient.assert(key.verb, key.pattern, { calledExactly: 1 })).toBeTruthy();
            expect(initReq.headers()["content-type"]).toBe("application/json");
        });
    }
});

test.describe('request body loopback should match original', () => {
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
    const verbs: string[] = ['PUT', 'POST', 'PATCH'];
    for (let verb of verbs) {
        test(verb, async ({ request }) => {
            const key: MatchKey = { verb: verb, pattern: 'playwright/request/BODY/loopback/should/match/original' };
            deleteState.push(key);
            await bamboozleClient.addRoute({
                match: key,
                response: {
                    status: "200",
                    loopback: true
                }
            });
            const initReq = await reqFactory(verb, `http://localhost:18080/${key.pattern}`, request, {
                hello: "world",
                number: 24
            });
            const init = await initReq.json();
            expect(init).toBeDefined();
            expect(init.hello).toEqual("world");
            expect(init.number).toEqual(24);
            expect(await bamboozleClient.assert(key.verb, key.pattern, { calledExactly: 1 })).toBeTruthy();
            expect(initReq.headers()["content-type"]).toBe("application/json");
        });
    }
});

test.describe('route matching', () => {
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
    test('route matching with strongly typed route params int with string val', async ({ request }) => {
        const key1: MatchKey = { verb: 'GET', pattern: 'playwright/route/matching/strong/typed/params/int/on/string/val/{param1:int}' };
        deleteState.push(key1);
        await bamboozleClient.addRoute({
            match: key1,
            response: {
                status: "200",
                headers: {
                },
                content: "{{routeValues.param1}}"
            }
        });
        const response1 = await request.get('http://localhost:18080/playwright/route/matching/strong/typed/params/int/on/string/val/24');
        const response2 = await request.get('http://localhost:18080/playwright/route/matching/strong/typed/params/int/on/string/val/twentyfour');

        expect(await response1.text()).toBe("24");
        expect(response2.status()).toBe(404);
        expect(response2.headers()["content-type"]).toBe(undefined);
    });
    test('route matching with strongly typed route params int vs string', async ({ request }) => {
        const key1: MatchKey = { verb: 'GET', pattern: 'playwright/route/matching/strong/typed/params/int/vs/string/{param1:int}' };
        deleteState.push(key1);
        await bamboozleClient.addRoute({
            match: key1,
            response: {
                status: "200",
                headers: {
                },
                content: "int: {{routeValues.param1}}"
            }
        });
        const key2: MatchKey = { verb: 'GET', pattern: 'playwright/route/matching/strong/typed/params/int/vs/string/{param1:string}' };
        deleteState.push(key2);
        await bamboozleClient.addRoute({
            match: key2,
            response: {
                status: "200",
                headers: {
                },
                content: "string: {{routeValues.param1}}"
            }
        });
        const response1 = await request.get('http://localhost:18080/playwright/route/matching/strong/typed/params/int/vs/string/24');
        const response2 = await request.get('http://localhost:18080/playwright/route/matching/strong/typed/params/int/vs/string/twentyfour');

        expect(await response1.text()).toBe("int: 24");
        expect(await response2.text()).toBe("string: twentyfour");
    });
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