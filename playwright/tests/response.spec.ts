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
            expect(await bamboozleClient.assert(key.verb, key.pattern, { expect: 1 })).toBeTruthy();
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
                    headers: {
                        "Content-Type": "application/json"
                    },
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
            console.log(await bamboozleClient.getCalls(key.verb, key.pattern));
            expect(await bamboozleClient.assert(key.verb, key.pattern, { expect: 1 })).toBeTruthy();
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