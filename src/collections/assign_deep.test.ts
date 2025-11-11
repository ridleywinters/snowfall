import { expect } from "@std/expect";
import { describe, it } from "@std/testing/bdd";

import { assignDeep } from "./assign_deep.ts";

type TestCase = [any[], any];
type TestCases = Record<string, TestCase[]>;

const testCases: TestCases = {
    "replace primitive values": [
        [[42, 100], 100],
        [[null, "not null"], "not null"],
        [[undefined, "defined"], "defined"],
        [[true, false], false],
    ],
    "merge arrays": [
        [[[], []], []],
        [[[1], []], [1]],
        [[[], [1]], [1]],
        [[[], [1], [2]], [1, 2]],
        [[[], [1, 3, 4], [2]], [1, 3, 4, 2]],
    ],
    "merge objects": [
        [[{}, {}], {}],
        [[{ a: 1 }, {}], { a: 1 }],
        [[{}, { a: 1 }], { a: 1 }],
        [[{ a: 1 }, { b: 2 }], { a: 1, b: 2 }],
        [[{ a: 1, c: 3 }, { b: 2 }], { a: 1, b: 2, c: 3 }],
        [[{ a: { x: 1 } }, { a: { y: 2 } }], { a: { x: 1, y: 2 } }],
        [[{ a: { x: 1, z: 3 } }, { a: { y: 2 } }], { a: { x: 1, y: 2, z: 3 } }],
    ],
    "merge nested values": [
        [
            [
                { name: "library", imports: ["fs", "path"] },
            ],
            { name: "library", imports: ["fs", "path"] },
        ],
        [
            [
                { name: "library", imports: ["fs", "path"] },
                { imports: ["fs", "directories"] },
                { version: "1.0.0" },
            ],
            {
                name: "library",
                imports: ["fs", "path", "directories"],
                version: "1.0.0",
            },
        ],
    ],
};

describe("typeOfValue", () => {
    for (const [testName, tests] of Object.entries(testCases)) {
        it(`should ${testName}`, () => {
            for (const [args, expected] of tests) {
                const result = assignDeep(args[0], ...args.slice(1));
                expect(result).toEqual(expected);
            }
        });
    }
});
