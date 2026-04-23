import { expect, test } from 'vitest';
import { BamboozleAssertBuilder, Operator } from '../src/assert.js';

test('BamboozleAssertBuilder constructs correct expressions', () => {
    const builder = new BamboozleAssertBuilder();
    
    builder.with(ctx => ctx.query.id.equals(42))
           .and()
           .with(ctx => ctx.header.authorization.startsWith('Bearer'));

    const expression = builder.build();

    expect(expression).toContain('query("id") == 42');
    expect(expression).toContain('&&');
    expect(expression).toContain('starts_with(header("authorization"), "Bearer")');
});
