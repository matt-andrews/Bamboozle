
export interface IBamboozleAssertBuilder {
    build(): string;
}

export class BamboozleAssertBuilder implements IBamboozleAssertBuilder {
    private assertions: IAssertion[] = [];

    with(assertion: IAssertion): Conjunction {
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
                result += ` ${assert.op}(${assert.type}("${assert.key}"), "${assert.value}") `
            } else if (assert.op == Operator.Equals || assert.op == Operator.NotEquals) {
                result += ` ${assert.type}("${assert.key}") ${assert.op} "${assert.value}" `
            } else {
                result += ` ${assert.type}("${assert.key}") ${assert.op} "${assert.value}" `
            }
        }

        return result;
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
    value: string;
}

export class QueryAssertion implements IAssertion {
    public type: AssertionType = AssertionType.Query;
    public key: string;
    public op: Operator;
    public value: string;
    constructor(key: string, op: Operator, value: string) {
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

export class VerbAssertion implements IAssertion {
    public type: AssertionType = AssertionType.Verb;
    public key: string = "verb";
    public op: Operator;
    public value: string;
    constructor(op: Operator, value: string) {
        this.op = op;
        this.value = value;
    }
}

export class PatternAssertion extends VerbAssertion {
    public type: AssertionType = AssertionType.Pattern;
    public key: string = "pattern";
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
    Verb = 'verb',
    Pattern = 'pattern',
    And = '&&',
    Or = '||'
}