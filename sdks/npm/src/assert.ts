
export interface IBamboozleAssertBuilder {
    build(): string;
}

export interface AssertionBuilder {
    equals(value: string | number | boolean): IAssertion;
    notEquals(value: string | number | boolean): IAssertion;
    greaterThan(value: number): IAssertion;
    greaterThanOrEqual(value: number): IAssertion;
    lessThan(value: number): IAssertion;
    lessThanOrEqual(value: number): IAssertion;
    contains(value: string): IAssertion;
    startsWith(value: string): IAssertion;
    endsWith(value: string): IAssertion;
}

export type AssertionProxy = Record<string, AssertionBuilder>;

export interface AssertContext {
    route: AssertionProxy;
    query: AssertionProxy;
    header: AssertionProxy;
    context: AssertionProxy;
    body: AssertionProxy;
}

export class BamboozleAssertBuilder implements IBamboozleAssertBuilder {
    private assertions: IAssertion[] = [];

    with(assertion: IAssertion): Conjunction;
    with(fn: (ctx: AssertContext) => IAssertion): Conjunction;
    with(assertionOrFn: IAssertion | ((ctx: AssertContext) => IAssertion)): Conjunction {
        const assertion = typeof assertionOrFn === 'function'
            ? assertionOrFn(makeAssertContext())
            : assertionOrFn;
        this.assertions.push(assertion);
        return new Conjunction(this);
    }

    withConjunction(conjunction: IAssertion): BamboozleAssertBuilder {
        this.assertions.push(conjunction);
        return this;
    }

    build(): string {
        let result: string = "";

        for (let assert of this.assertions) {
            if (assert.type == AssertionType.And || assert.type == AssertionType.Or) {
                result += ' ' + assert.type + ' ';
                continue;
            }

            if (assert.op == Operator.Contains || assert.op == Operator.StartsWith || assert.op == Operator.EndsWith) {
                result += ` ${assert.op}(${this.getKeyFromType(assert)}, "${assert.value}") `
            } else if (typeof assert.value === 'number' || typeof assert.value === 'boolean') {
                result += ` ${this.getKeyFromType(assert)} ${assert.op} ${assert.value} `
            } else {
                result += ` ${this.getKeyFromType(assert)} ${assert.op} "${assert.value}" `
            }
        }

        return result;
    }

    getKeyFromType(assert: IAssertion): string {
        if (assert.type == AssertionType.Context) {
            return `${assert.key}`;
        } else {
            return `${assert.type}("${assert.key}")`;
        }
    }
}

export class Conjunction implements IBamboozleAssertBuilder {
    private parent: BamboozleAssertBuilder;
    constructor(parent: BamboozleAssertBuilder) {
        this.parent = parent;
    }
    and(): BamboozleAssertBuilder {
        return this.parent.withConjunction(new AndAssertion());
    }
    or(): BamboozleAssertBuilder {
        return this.parent.withConjunction(new OrAssertion());
    }
    build(): string {
        return this.parent.build();
    }
}

export interface IAssertion {
    type: AssertionType;
    key: string;
    op: Operator;
    value: string | number | boolean;
}

export class QueryAssertion implements IAssertion {
    public type: AssertionType = AssertionType.Query;
    public key: string;
    public op: Operator;
    public value: string | number | boolean;
    constructor(key: string, op: Operator, value: string | number | boolean) {
        this.key = key;
        this.op = op;
        this.value = value;
    }
}

export class RouteAssertion extends QueryAssertion {
    public type: AssertionType = AssertionType.Route;
}

export class HeaderAssertion extends QueryAssertion {
    public type: AssertionType = AssertionType.Header;
}

export class ContextAssertion implements IAssertion {
    public type: AssertionType = AssertionType.Context;
    public key: string;
    public op: Operator;
    public value: string | number | boolean;
    constructor(key: string, op: Operator, value: string | number | boolean) {
        this.key = key;
        this.op = op;
        this.value = value;
    }
}

export class BodyAssertion implements IAssertion {
    public type: AssertionType = AssertionType.Body;
    public key: string;
    public op: Operator;
    public value: string | number | boolean;
    constructor(key: string, op: Operator, value: string | number | boolean) {
        this.key = key;
        this.op = op;
        this.value = value;
    }
}

export class AndAssertion implements IAssertion {
    public type: AssertionType = AssertionType.And;
    public key: string = "and";
    public op: Operator = Operator.Equals;
    public value: string = "&&";
}

export class OrAssertion implements IAssertion {
    public type: AssertionType = AssertionType.Or;
    public key: string = "or";
    public op: Operator = Operator.Equals;
    public value: string = "||";
}

export enum Operator {
    'Equals' = '==',
    'NotEquals' = '!=',
    'GreaterThan' = '>',
    'GreaterThanOrEqual' = '>=',
    'LessThan' = '<',
    'LessThanOrEqual' = '<=',
    'Contains' = 'contains',
    'StartsWith' = 'starts_with',
    'EndsWith' = 'ends_with'
}

export enum AssertionType {
    Query = 'query',
    Route = 'route',
    Header = 'header',
    Context = 'context',
    Body = 'body',
    And = '&&',
    Or = '||'
}

function makeProxy(ctor: new (key: string, op: Operator, value: string | number) => IAssertion): AssertionProxy {
    return new Proxy({} as AssertionProxy, {
        get(_, key: string | symbol): AssertionBuilder {
            const k = String(key);
            return {
                equals: (v: string | number | boolean) => new ctor(k, Operator.Equals, v),
                notEquals: (v: string | number | boolean) => new ctor(k, Operator.NotEquals, v),
                greaterThan: (v) => new ctor(k, Operator.GreaterThan, v),
                greaterThanOrEqual: (v) => new ctor(k, Operator.GreaterThanOrEqual, v),
                lessThan: (v) => new ctor(k, Operator.LessThan, v),
                lessThanOrEqual: (v) => new ctor(k, Operator.LessThanOrEqual, v),
                contains: (v) => new ctor(k, Operator.Contains, v),
                startsWith: (v) => new ctor(k, Operator.StartsWith, v),
                endsWith: (v) => new ctor(k, Operator.EndsWith, v),
            };
        }
    });
}


function makeAssertContext(): AssertContext {
    return {
        route: makeProxy(RouteAssertion),
        query: makeProxy(QueryAssertion),
        header: makeProxy(HeaderAssertion),
        context: makeProxy(ContextAssertion),
        body: makeProxy(BodyAssertion)
    };
}