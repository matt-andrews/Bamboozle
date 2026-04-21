import { test, expect } from '@playwright/test';
import { BamboozleClient, MatchKey } from '@bamboozle/sdk';

const bamboozleClient = new BamboozleClient({ baseUrl: 'http://localhost:19090' });

function makeKey(suffix: string): MatchKey {
    return { verb: 'GET', pattern: `playwright/sessions/${suffix}` };
}

async function cleanup(keys: MatchKey[]) {
    for (const key of keys) {
        try { await bamboozleClient.clearCalls(key.verb, key.pattern); } catch { }
        try { await bamboozleClient.deleteRoute(key.verb, key.pattern); } catch { }
    }
}

test.describe(() => {
    const keys: MatchKey[] = [];
    test.afterEach(() => cleanup(keys));

    test('can assert with session', async ({ request }) => {
        const key = makeKey('can/assert/with/session');
        keys.push(key);

        await bamboozleClient.addRoute({
            match: key,
            response: { status: '200', content: 'ok' },
            simulation: { delay: { type: 'fixed', ms: 200 } },
        });

        const start = Date.now();
        await request.get(`http://localhost:18081/${key.pattern}`);
        await request.get(`http://localhost:18082/${key.pattern}`);
        await request.get(`http://localhost:18083/${key.pattern}`);

        const calls = await bamboozleClient.getCalls(key.verb, key.pattern);
        expect(calls).toHaveLength(3);

        expect(calls.find(f => f.port == "18081")).toBeDefined();
        expect(calls.find(f => f.port == "18082")).toBeDefined();
        expect(calls.find(f => f.port == "18083")).toBeDefined();

        expect(await bamboozleClient.assert(key.verb, key.pattern, {
            port: "18081"
        })).toBeTruthy();
        expect(await bamboozleClient.assert(key.verb, key.pattern, {
            port: "18082"
        })).toBeTruthy();
        expect(await bamboozleClient.assert(key.verb, key.pattern, {
            port: "18083"
        })).toBeTruthy();
    });
});