import { expect } from "@std/expect";
import { describe, it } from "@std/testing/bdd";

import { typeOfValue } from "./type_of_value.ts";

type TestCases = Record<string, [any, string][]>;

const testCases: TestCases = {
    "identify undefined": [
        [undefined, "undefined"],
    ],
    "identify null": [
        [null, "null"],
    ],
    "identify boolean": [
        [true, "boolean"],
        [false, "boolean"],
    ],
    "identify number": [
        [42, "number"],
        [-3.14, "number"],
        [0, "number"],
        [NaN, "number"],
        [Infinity, "number"],
    ],
    "identify bigint": [
        [123n, "bigint"],
        [-456n, "bigint"],
    ],
    "identify string": [
        ["hello", "string"],
        ["", "string"],
    ],
    "identify array": [
        [[], "array"],
        [[1, 2, 3], "array"],
        [["a", "b", "c"], "array"],
        [[null, 7, true, "string"], "array"],
    ],
    "identify object": [
        [{}, "object"],
        [{ a: 1 }, "object"],
        [{ a: 1, b: 2 }, "object"],
    ],
    "identify symbol": [
        [Symbol("test"), "symbol"],
        [Symbol.iterator, "symbol"],
    ],
    "identify date": [
        [new Date(), "date"],
        [new Date("2024-01-01T00:00:00Z"), "date"],
    ],
    "identify function": [
        [function () {}, "function"],
        [() => {}, "function"],
        [async function () {}, "function"],
    ],
    "identify promise": [
        [Promise.resolve(), "promise"],
        [Promise.reject().catch(() => {}), "promise"],
    ],
    "identify error": [
        [new Error("test error"), "error"],
        [new TypeError("type error"), "error"],
    ],
    "identify set": [
        [new Set(), "set"],
        [new Set([1, 2, 3]), "set"],
    ],
    "identify map": [
        [new Map(), "map"],
        [new Map([["a", 1], ["b", 2]]), "map"],
    ],
    "identify weakmap": [
        [new WeakMap(), "weakmap"],
        [new WeakMap([[{}, 1], [{}, 2]]), "weakmap"],
    ],
};

describe("typeOfValue", () => {
    for (const [testName, tests] of Object.entries(testCases)) {
        it(`should ${testName}`, () => {
            for (const [input, expected] of tests) {
                const result = typeOfValue(input);
                expect(result).toBe(expected);
            }
        });
    }
});
