import { expect } from "https://deno.land/std@0.208.0/expect/mod.ts";
import { describe, it } from "https://deno.land/std@0.208.0/testing/bdd.ts";
import { template } from "./template.ts";

type Case = {
    name: string;
    tpl: string;
    vars?: Record<string, unknown>;
    want: string;
};

const cases: Case[] = [
    {
        name: "simple replacement",
        tpl: "Hello, {{name}}!",
        vars: { name: "World" },
        want: "Hello, World!",
    },
    {
        name: "multiple replacements",
        tpl: "{{greeting}}, {{name}}!",
        vars: { greeting: "Hi", name: "Alice" },
        want: "Hi, Alice!",
    },
    {
        name: "missing variable becomes empty string",
        tpl: "Value: {{missing}}",
        vars: {},
        want: "Value: ",
    },
    {
        name: "adjacent placeholders",
        tpl: "{{a}}{{b}}{{c}}",
        vars: { a: "1", b: "2", c: "3" },
        want: "123",
    },
    {
        name: "non-identifier placeholder remains unchanged",
        tpl: "bad: {{1a}}",
        vars: { "1a": "no" },
        want: "bad: {{1a}}",
    },
    {
        name: "underscore and digit variable",
        tpl: "ok: {{_var1}}",
        vars: { _var1: "yes" },
        want: "ok: yes",
    },
    {
        name: "spaces inside braces",
        tpl: "hello {{ name }}",
        vars: { name: "Bob" },
        want: "hello {{ name }}",
    },
    {
        name: "array index placeholder resolves",
        tpl: "{{arr.0}}",
        vars: { arr: ["a", "b"] },
        want: "a",
    },
    {
        name: "object without path",
        tpl: "obj: {{obj}}",
        vars: { obj: { a: 1 } },
        want: "obj: [object Object]",
    },
    {
        name: "nested missing returns empty",
        tpl: "{{a.b.c}}",
        vars: { a: { b: {} } },
        want: "",
    },
    {
        name: "placeholder with dash remains unchanged",
        tpl: "dash: {{a-b}}",
        vars: { "a-b": "x" },
        want: "dash: {{a-b}}",
    },
    {
        name: "repeated placeholder",
        tpl: "{{x}} and {{x}}",
        vars: { x: "Y" },
        want: "Y and Y",
    },
    {
        name: "unclosed left brace remains unchanged",
        tpl: "Hello {{name",
        vars: { name: "Bob" },
        want: "Hello {{name",
    },
    {
        name: "unclosed right brace remains unchanged",
        tpl: "Hello name}}",
        vars: { name: "Bob" },
        want: "Hello name}}",
    },
    {
        name: "nested opening brace yields wrapped result",
        tpl: "{{{name}}}",
        vars: { name: "Bob" },
        want: "{Bob}",
    },
    {
        name: "extra closing brace after placeholder",
        tpl: "{{name}}}",
        vars: { name: "Bob" },
        want: "Bob}",
    },
    {
        name: "nested variable path",
        tpl: "hello {{name.first}}",
        vars: { name: { first: "Bob", last: "Engineer" } },
        want: "hello Bob",
    },
    {
        name: "empty template",
        tpl: "",
        vars: {},
        want: "",
    },
    {
        name: "variable set to empty string",
        tpl: "value: '{{a}}'",
        vars: { a: "" },
        want: "value: ''",
    },
];

describe("template", () => {
    cases.forEach((c) => {
        it(c.name, () => {
            const got = template(c.tpl, c.vars ?? {});
            expect(got).toBe(c.want);
        });
    });

    const envCases: Array<{
        name: string;
        tpl: string;
        vars?: Record<string, unknown>;
        env?: Record<string, string>;
        want: string;
    }> = [
        {
            name: "env and placeholder expansion",
            tpl: "{{greeting}}, $TEST_NAME!",
            vars: { greeting: "Hi" },
            env: { TEST_NAME: "Bob" },
            want: "Hi, Bob!",
        },
    ];

    envCases.forEach((c) => {
        it(c.name, () => {
            const prev = new Map<string, string | undefined>();
            if (c.env) {
                for (const [k, v] of Object.entries(c.env)) {
                    prev.set(k, Deno.env.get(k));
                    Deno.env.set(k, v);
                }
            }
            try {
                const got = template(c.tpl, c.vars ?? {});
                expect(got).toBe(c.want);
            } finally {
                for (const [k, v] of prev.entries()) {
                    if (v === undefined) {
                        Deno.env.delete(k);
                    } else {
                        Deno.env.set(k, v);
                    }
                }
            }
        });
    });

    it("missing env variable throws", () => {
        const key = "__MISSING_ENV_TEST__";
        Deno.env.delete(key);
        expect(() => template(`Value: $${key}`)).toThrow();
    });
});
