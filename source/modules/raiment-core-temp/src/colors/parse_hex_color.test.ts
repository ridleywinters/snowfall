import { expect } from "@std/expect";
import { describe, it } from "@std/testing/bdd";

import { parseHexColor } from "./parse_hex_color.ts";
import { RGBAU8, RGBU8 } from "./types.ts";

type TestCases = Record<string, [string, RGBU8 | RGBAU8][]>;

const testCases: TestCases = {
    "parse 3-digit hex colors": [
        ["#fff", { r: 255, g: 255, b: 255 }],
        ["#000", { r: 0, g: 0, b: 0 }],
        ["#f00", { r: 255, g: 0, b: 0 }],
        ["#0f0", { r: 0, g: 255, b: 0 }],
        ["#00f", { r: 0, g: 0, b: 255 }],
        ["#abc", { r: 170, g: 187, b: 204 }],
    ],
    "parse 3-digit hex colors without hash": [
        ["fff", { r: 255, g: 255, b: 255 }],
        ["000", { r: 0, g: 0, b: 0 }],
        ["f00", { r: 255, g: 0, b: 0 }],
    ],
    "parse 6-digit hex colors": [
        ["#ffffff", { r: 255, g: 255, b: 255 }],
        ["#000000", { r: 0, g: 0, b: 0 }],
        ["#ff0000", { r: 255, g: 0, b: 0 }],
        ["#00ff00", { r: 0, g: 255, b: 0 }],
        ["#0000ff", { r: 0, g: 0, b: 255 }],
        ["#aabbcc", { r: 170, g: 187, b: 204 }],
        ["#123456", { r: 18, g: 52, b: 86 }],
    ],
    "parse 6-digit hex colors without hash": [
        ["ffffff", { r: 255, g: 255, b: 255 }],
        ["000000", { r: 0, g: 0, b: 0 }],
        ["ff0000", { r: 255, g: 0, b: 0 }],
    ],
    "parse 8-digit hex colors with alpha": [
        ["#ffffffff", { r: 255, g: 255, b: 255, a: 255 }],
        ["#00000000", { r: 0, g: 0, b: 0, a: 0 }],
        ["#ff0000ff", { r: 255, g: 0, b: 0, a: 255 }],
        ["#00ff0080", { r: 0, g: 255, b: 0, a: 128 }],
        ["#0000ff7f", { r: 0, g: 0, b: 255, a: 127 }],
        ["#aabbcc80", { r: 170, g: 187, b: 204, a: 128 }],
    ],
    "parse 8-digit hex colors with alpha without hash": [
        ["ffffffff", { r: 255, g: 255, b: 255, a: 255 }],
        ["00000000", { r: 0, g: 0, b: 0, a: 0 }],
        ["ff0000ff", { r: 255, g: 0, b: 0, a: 255 }],
    ],
};

describe("parseHexColor", () => {
    for (const [testName, tests] of Object.entries(testCases)) {
        it(`should ${testName}`, () => {
            for (const [input, expected] of tests) {
                const result = parseHexColor(input);
                expect(result).toEqual(expected);
            }
        });
    }

    it("should not include alpha value for 3-digit hex colors", () => {
        const result = parseHexColor("#fff");
        expect(result).toEqual({ r: 255, g: 255, b: 255 });
        expect("a" in result).toBe(false);
    });

    it("should not include alpha value for 6-digit hex colors", () => {
        const result = parseHexColor("#ffffff");
        expect(result).toEqual({ r: 255, g: 255, b: 255 });
        expect("a" in result).toBe(false);
    });

    it("should include alpha value only for 8-digit hex colors", () => {
        const result = parseHexColor("#ffffff80");
        expect(result).toEqual({ r: 255, g: 255, b: 255, a: 128 });
        expect("a" in result).toBe(true);
    });

    it("should throw error for invalid hex color formats", () => {
        const invalidFormats = [
            "#ff",
            "#ffff",
            "#fffff",
            "#fffffff",
            "#fffffffff",
            "ff",
            "ffff",
            "fffff",
            "fffffff",
            "fffffffff",
            "#gg0000",
            "#xyz",
            "",
            "#",
            "##123",
            "#123#",
        ];

        for (const invalid of invalidFormats) {
            expect(() => parseHexColor(invalid)).toThrow();
        }
    });

    it("should throw error for invalid non-hexadecimal characters", () => {
        const invalidColors = [
            "#gg0000",
            "#xyz",
            "#fffffg",
            "#12345g",
            "#zzzzzz",
            "gggggg",
            "xyz",
        ];

        for (const invalid of invalidColors) {
            expect(
                () => parseHexColor(invalid),
                `'${invalid}'`,
            ).toThrow();
        }
    });

    it("should throw error for named colors", () => {
        const namedColors = [
            "red",
            "blue",
            "green",
            "white",
            "black",
            "yellow",
            "cyan",
            "magenta",
            "transparent",
        ];

        for (const named of namedColors) {
            expect(() => parseHexColor(named)).toThrow();
        }
    });

    it("should throw error for strings of wrong length", () => {
        const wrongLengths = [
            "#f", // 1 digit
            "#ff", // 2 digits
            "#ffff", // 4 digits
            "#fffff", // 5 digits
            "#fffffff", // 7 digits
            "#fffffffff", // 9 digits
            "f", // 1 digit without hash
            "ff", // 2 digits without hash
            "ffff", // 4 digits without hash
            "fffff", // 5 digits without hash
            "fffffff", // 7 digits without hash
            "fffffffff", // 9 digits without hash
        ];

        for (const invalid of wrongLengths) {
            expect(() => parseHexColor(invalid)).toThrow();
        }
    });
});
