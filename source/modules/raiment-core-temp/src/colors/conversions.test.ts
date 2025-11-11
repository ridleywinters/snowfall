import { expect } from "https://deno.land/std@0.208.0/expect/mod.ts";
import { describe, it } from "https://deno.land/std@0.208.0/testing/bdd.ts";

import { rgbau8ArrayToHex7, rgbu8ToHSLF32 } from "./conversions.ts";
import { HSLF32, RGBU8 } from "./types.ts";

type RGBAArrayCase = {
    name: string;
    input: Uint8Array | [number, number, number, number];
    want: string;
};

type RGBToHSLCase = {
    name: string;
    input: RGBU8;
    want: HSLF32;
};

const rgbau8ArrayToHex7Cases: RGBAArrayCase[] = [
    {
        name: "white",
        input: new Uint8Array([255, 255, 255, 255]),
        want: "#ffffff",
    },
    {
        name: "black",
        input: new Uint8Array([0, 0, 0, 255]),
        want: "#000000",
    },
    {
        name: "red",
        input: [255, 0, 0, 255],
        want: "#ff0000",
    },
    {
        name: "green",
        input: [0, 255, 0, 255],
        want: "#00ff00",
    },
    {
        name: "blue",
        input: [0, 0, 255, 255],
        want: "#0000ff",
    },
    {
        name: "arbitrary color",
        input: [170, 187, 204, 128],
        want: "#aabbcc",
    },
];

const rgbu8ToHSLF32Cases: RGBToHSLCase[] = [
    {
        name: "white",
        input: { r: 255, g: 255, b: 255 },
        want: { h: 0, s: 0, l: 1 },
    },
    {
        name: "black",
        input: { r: 0, g: 0, b: 0 },
        want: { h: 0, s: 0, l: 0 },
    },
    {
        name: "red",
        input: { r: 255, g: 0, b: 0 },
        want: { h: 0, s: 1, l: 0.5 },
    },
    {
        name: "green",
        input: { r: 0, g: 255, b: 0 },
        want: { h: 1 / 3, s: 1, l: 0.5 },
    },
    {
        name: "blue",
        input: { r: 0, g: 0, b: 255 },
        want: { h: 2 / 3, s: 1, l: 0.5 },
    },
    {
        name: "gray mid",
        input: { r: 128, g: 128, b: 128 },
        want: { h: 0, s: 0, l: 128 / 255 },
    },
];

describe("rgbau8ArrayToHex7", () => {
    rgbau8ArrayToHex7Cases.forEach((c) => {
        it(c.name, () => {
            const got = rgbau8ArrayToHex7(c.input);
            expect(got).toBe(c.want);
        });
    });

    it("throws on wrong array length", () => {
        expect(() => rgbau8ArrayToHex7(new Uint8Array([1, 2, 3]))).toThrow();
    });
});

describe("rgbu8ToHSLF32", () => {
    rgbu8ToHSLF32Cases.forEach((c) => {
        it(c.name, () => {
            const got = rgbu8ToHSLF32(c.input);
            expect(got.h).toBeCloseTo(c.want.h, 5);
            expect(got.s).toBeCloseTo(c.want.s, 5);
            expect(got.l).toBeCloseTo(c.want.l, 5);
        });
    });
});
