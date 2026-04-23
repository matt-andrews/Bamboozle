# Bamboozle SDKs

## `@bamboozle/sdk` — TypeScript / JavaScript

Typed client for the Bamboozle control API. Works in Node.js 18+ and any runtime with the `fetch` API.

```bash
npm install @matt-andrews/bamboozle-sdk
```

```typescript
import { BamboozleClient } from '@bamboozle/sdk';

const client = new BamboozleClient({ baseUrl: 'http://localhost:9090' });

// register a route
await client.addRoute({
  match: { verb: 'GET', pattern: '/version' },
  response: { status: '200', content: '1.0.0' }
});

// assert it was called exactly once
await client.assert('GET', '/version', { calledExactly: 1 });

// reset
await client.reset();
```

The SDK exports `BamboozleClient`, `BamboozleAssertBuilder` (fluent expression builder), and all control API types as TypeScript interfaces.
