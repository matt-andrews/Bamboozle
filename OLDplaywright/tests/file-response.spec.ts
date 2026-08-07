import { test, expect } from '@playwright/test';
import { BamboozleClient, BamboozleError, MatchKey } from '@bamboozle/sdk';

const bamboozleClient = new BamboozleClient({ baseUrl: 'http://localhost:19090' });

test.describe('contentFile serves Liquid-rendered file content', () => {
    let deleteState: MatchKey[] = [];
    test.afterEach(async () => {
        for (const key of deleteState) {
            try {
                await bamboozleClient.clearCalls(key.verb, key.pattern);
                await bamboozleClient.deleteRoute(key.verb, key.pattern);
            } catch { }
        }
        deleteState = [];
    });

    test('renders Liquid template from file with route values', async ({ request }) => {
        const key: MatchKey = { verb: 'GET', pattern: 'playwright/file/greeting/{name}' };
        deleteState.push(key);
        await bamboozleClient.addRoute({
            match: key,
            response: {
                status: '200',
                contentFile: '/test_configs/assets/greeting.txt',
            },
        });

        const response = await request.get('http://localhost:18080/playwright/file/greeting/World');
        expect(response.status()).toBe(200);
        expect(await response.text()).toBe('Hello World!');
        expect(await bamboozleClient.assert(key.verb, key.pattern, { calledExactly: 1 })).toBeTruthy();
        expect(response.headers()["content-type"]).toBe("text/plain");
    });

    test('renders Liquid template from file with different route values', async ({ request }) => {
        const key: MatchKey = { verb: 'GET', pattern: 'playwright/file/greeting/alt/{name}' };
        deleteState.push(key);
        await bamboozleClient.addRoute({
            match: key,
            response: {
                status: '200',
                contentFile: '/test_configs/assets/greeting.txt',
            },
        });

        const response = await request.get('http://localhost:18080/playwright/file/greeting/alt/Alice');
        expect(response.status()).toBe(200);
        expect(await response.text()).toBe('Hello Alice!');
        expect(response.headers()["content-type"]).toBe("text/plain");
    });
});

test.describe('binaryFile serves raw bytes', () => {
    let deleteState: MatchKey[] = [];
    test.afterEach(async () => {
        for (const key of deleteState) {
            try {
                await bamboozleClient.clearCalls(key.verb, key.pattern);
                await bamboozleClient.deleteRoute(key.verb, key.pattern);
            } catch { }
        }
        deleteState = [];
    });

    test('returns raw binary content unchanged', async ({ request }) => {
        const key: MatchKey = { verb: 'GET', pattern: 'playwright/file/binary/sample' };
        deleteState.push(key);
        await bamboozleClient.addRoute({
            match: key,
            response: {
                status: '200',
                binaryFile: '/test_configs/assets/sample.bin',
            },
        });

        const response = await request.get('http://localhost:18080/playwright/file/binary/sample');
        expect(response.status()).toBe(200);
        const body = await response.body();
        // sample.bin contains PNG magic bytes: 89 50 4E 47
        expect(body.length).toBe(4);
        expect(body[0]).toBe(0x89);
        expect(body[1]).toBe(0x50);
        expect(body[2]).toBe(0x4e);
        expect(body[3]).toBe(0x47);
        expect(await bamboozleClient.assert(key.verb, key.pattern, { calledExactly: 1 })).toBeTruthy();
        expect(response.headers()["content-type"]).toBe("application/octet-stream");
    });

    test('binary content is not processed as Liquid template', async ({ request }) => {
        const key: MatchKey = { verb: 'GET', pattern: 'playwright/file/binary/no-template' };
        deleteState.push(key);
        await bamboozleClient.addRoute({
            match: key,
            response: {
                status: '200',
                binaryFile: '/test_configs/assets/sample.bin',
            },
        });

        const response = await request.get('http://localhost:18080/playwright/file/binary/no-template');
        expect(response.status()).toBe(200);
        const body = await response.body();
        // Raw bytes must be returned without modification
        expect(Buffer.from([0x89, 0x50, 0x4e, 0x47]).equals(body)).toBeTruthy();
        expect(response.headers()["content-type"]).toBe("application/octet-stream");
    });
});

test.describe('validation rejects multiple content options', () => {
    test('content and contentFile together returns 400', async () => {
        let caught: BamboozleError | undefined;
        try {
            await bamboozleClient.addRoute({
                match: { verb: 'GET', pattern: 'playwright/file/validation/content-and-file' },
                response: {
                    content: 'inline text',
                    contentFile: '/test_configs/assets/greeting.txt',
                },
            });
        } catch (e) {
            if (e instanceof BamboozleError) caught = e;
            else throw e;
        }
        expect(caught).toBeDefined();
        expect(caught!.status).toBe(400);
    });

    test('content and binaryFile together returns 400', async () => {
        let caught: BamboozleError | undefined;
        try {
            await bamboozleClient.addRoute({
                match: { verb: 'GET', pattern: 'playwright/file/validation/content-and-binary' },
                response: {
                    content: 'inline text',
                    binaryFile: '/test_configs/assets/sample.bin',
                },
            });
        } catch (e) {
            if (e instanceof BamboozleError) caught = e;
            else throw e;
        }
        expect(caught).toBeDefined();
        expect(caught!.status).toBe(400);
    });

    test('contentFile and binaryFile together returns 400', async () => {
        let caught: BamboozleError | undefined;
        try {
            await bamboozleClient.addRoute({
                match: { verb: 'GET', pattern: 'playwright/file/validation/both-files' },
                response: {
                    contentFile: '/test_configs/assets/greeting.txt',
                    binaryFile: '/test_configs/assets/sample.bin',
                },
            });
        } catch (e) {
            if (e instanceof BamboozleError) caught = e;
            else throw e;
        }
        expect(caught).toBeDefined();
        expect(caught!.status).toBe(400);
    });

    test('loopback and content together returns 400', async () => {
        let caught: BamboozleError | undefined;
        try {
            await bamboozleClient.addRoute({
                match: { verb: 'POST', pattern: 'playwright/file/validation/loopback-and-content' },
                response: {
                    loopback: true,
                    content: 'inline text',
                },
            });
        } catch (e) {
            if (e instanceof BamboozleError) caught = e;
            else throw e;
        }
        expect(caught).toBeDefined();
        expect(caught!.status).toBe(400);
    });

    test('loopback and contentFile together returns 400', async () => {
        let caught: BamboozleError | undefined;
        try {
            await bamboozleClient.addRoute({
                match: { verb: 'POST', pattern: 'playwright/file/validation/loopback-and-file' },
                response: {
                    loopback: true,
                    contentFile: '/test_configs/assets/greeting.txt',
                },
            });
        } catch (e) {
            if (e instanceof BamboozleError) caught = e;
            else throw e;
        }
        expect(caught).toBeDefined();
        expect(caught!.status).toBe(400);
    });
});
