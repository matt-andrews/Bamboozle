import { test, expect } from '@playwright/test';
import { BamboozleClient, MatchKey } from '@bamboozle/sdk';

const bamboozleClient = new BamboozleClient({ baseUrl: 'http://localhost:19090' });

function makeKey(suffix: string): MatchKey {
    return { verb: 'GET', pattern: `playwright/simulation/${suffix}` };
}

async function cleanup(keys: MatchKey[]) {
    for (const key of keys) {
        try { await bamboozleClient.clearCalls(key.verb, key.pattern); } catch { }
        try { await bamboozleClient.deleteRoute(key.verb, key.pattern); } catch { }
    }
}

test.describe('fixed delay', () => {
    const keys: MatchKey[] = [];
    test.afterEach(() => cleanup(keys));

    test('response is delayed by at least the configured milliseconds', async ({ request }) => {
        const key = makeKey('fixed-delay');
        keys.push(key);

        await bamboozleClient.addRoute({
            match: key,
            response: { status: '200', content: 'ok' },
            simulation: { delay: { type: 'fixed', ms: 200 } },
        });

        const start = Date.now();
        const response = await request.get(`http://localhost:18080/${key.pattern}`);
        const elapsed = Date.now() - start;

        expect(response.status()).toBe(200);
        expect(elapsed).toBeGreaterThanOrEqual(200);
    });
});

test.describe('random delay', () => {
    const keys: MatchKey[] = [];
    test.afterEach(() => cleanup(keys));

    test('response is delayed within the configured range', async ({ request }) => {
        const key = makeKey('random-delay');
        keys.push(key);

        await bamboozleClient.addRoute({
            match: key,
            response: { status: '200', content: 'ok' },
            simulation: { delay: { type: 'random', minMs: 100, maxMs: 400 } },
        });

        const start = Date.now();
        const response = await request.get(`http://localhost:18080/${key.pattern}`);
        const elapsed = Date.now() - start;

        expect(response.status()).toBe(200);
        expect(elapsed).toBeGreaterThanOrEqual(100);
    });
});

test.describe('emptyResponse fault', () => {
    const keys: MatchKey[] = [];
    test.afterEach(() => cleanup(keys));

    test('always returns 200 with empty body when probability is 1', async ({ request }) => {
        const key = makeKey('empty-response-always');
        keys.push(key);

        await bamboozleClient.addRoute({
            match: key,
            response: { status: '200', content: 'this should not appear' },
            simulation: { fault: { type: 'emptyResponse', probability: 1.0 } },
        });

        const response = await request.get(`http://localhost:18080/${key.pattern}`);
        expect(response.status()).toBe(200);
        expect(await response.text()).toBe('');
    });

    test('never triggers when probability is 0', async ({ request }) => {
        const key = makeKey('empty-response-never');
        keys.push(key);

        await bamboozleClient.addRoute({
            match: key,
            response: { status: '200', content: 'normal response' },
            simulation: { fault: { type: 'emptyResponse', probability: 0.0 } },
        });

        for (let i = 0; i < 5; i++) {
            const response = await request.get(`http://localhost:18080/${key.pattern}`);
            expect(await response.text()).toBe('normal response');
        }
    });
});

test.describe('connectionReset fault', () => {
    const keys: MatchKey[] = [];
    test.afterEach(() => cleanup(keys));

    test('aborts the connection — body is empty or request throws', async ({ request }) => {
        const key = makeKey('connection-reset');
        keys.push(key);

        await bamboozleClient.addRoute({
            match: key,
            response: { status: '200', content: 'this should not appear' },
            simulation: { fault: { type: 'connectionReset', probability: 1.0 } },
        });

        let bodyText = '';
        try {
            const response = await request.get(`http://localhost:18080/${key.pattern}`);
            bodyText = await response.text();
        } catch {
            // connection was reset before or during body read — expected
        }

        expect(bodyText).toBe('');
    });
});

test.describe('delay combined with fault', () => {
    const keys: MatchKey[] = [];
    test.afterEach(() => cleanup(keys));

    test('delay applies before the fault triggers', async ({ request }) => {
        const key = makeKey('delay-and-fault');
        keys.push(key);

        await bamboozleClient.addRoute({
            match: key,
            response: { status: '200', content: 'this should not appear' },
            simulation: {
                delay: { type: 'fixed', ms: 150 },
                fault: { type: 'emptyResponse', probability: 1.0 },
            },
        });

        const start = Date.now();
        const response = await request.get(`http://localhost:18080/${key.pattern}`);
        const elapsed = Date.now() - start;

        expect(response.status()).toBe(200);
        expect(await response.text()).toBe('');
        expect(elapsed).toBeGreaterThanOrEqual(150);
    });
});

test.describe('calls are recorded even when fault triggers', () => {
    const keys: MatchKey[] = [];
    test.afterEach(() => cleanup(keys));

    test('faulted requests appear in call history', async ({ request }) => {
        const key = makeKey('fault-recorded');
        keys.push(key);

        await bamboozleClient.addRoute({
            match: key,
            response: { status: '200', content: 'ok' },
            simulation: { fault: { type: 'emptyResponse', probability: 1.0 } },
        });

        await request.get(`http://localhost:18080/${key.pattern}`);
        await request.get(`http://localhost:18080/${key.pattern}`);

        expect(await bamboozleClient.assert(key.verb, key.pattern, { expect: 2 })).toBeTruthy();
    });
});
