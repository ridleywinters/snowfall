import { HexColor, RGBAU8, RGBU8 } from "./types.ts";

/**
 * Parse hex colors of the following formats:
 *
 * - #RGB
 * - #RRGGBB
 * - #RRGGBBAA
 * - RGB
 * - RRGGBB
 * - RRGGBBAA
 *
 * Throws an `Error` on invalid input.
 */
export function parseHexColor(hex: HexColor | string): RGBU8 | RGBAU8 {
    // Do the actual parsing work, then check for validity
    const result = parseWorker(hex);
    if (
        !(result.r >= 0 && result.r <= 255) ||
        !(result.g >= 0 && result.g <= 255) ||
        !(result.b >= 0 && result.b <= 255)
    ) {
        throw new Error(`Parsed color components out of range: ${hex}`);
    }
    const rgba = result as RGBAU8;
    if (rgba.a !== undefined) {
        if (!(rgba.a >= 0 && rgba.a <= 255)) {
            throw new Error(`Parsed alpha component out of range: ${hex}`);
        }
    }
    return result;
}

export function parseHexRGBU8(hex: HexColor | string): RGBU8 {
    const result = parseHexColor(hex);
    if ("a" in result) {
        throw new Error(`Expected RGB color but got RGBA: ${hex}`);
    }
    return result as RGBU8;
}

function parseWorker(hex: HexColor | string): RGBU8 | RGBAU8 {
    const digits = hex.startsWith("#") ? hex.slice(1) : hex;
    let r: number, g: number, b: number, a: number;

    switch (digits.length) {
        case 3:
            r = parseHex2(digits[0] + digits[0]);
            g = parseHex2(digits[1] + digits[1]);
            b = parseHex2(digits[2] + digits[2]);
            return { r, g, b };
        case 6:
            r = parseHex2(digits.substring(0, 2));
            g = parseHex2(digits.substring(2, 4));
            b = parseHex2(digits.substring(4, 6));
            return { r, g, b };
        case 8:
            r = parseHex2(digits.substring(0, 2));
            g = parseHex2(digits.substring(2, 4));
            b = parseHex2(digits.substring(4, 6));
            a = parseHex2(digits.substring(6, 8));
            return { r, g, b, a };
        default:
            throw new Error(`Invalid hex color format: ${hex}`);
    }
}

const hexTable: Record<string, number> = {
    "0": 0,
    "1": 1,
    "2": 2,
    "3": 3,
    "4": 4,
    "5": 5,
    "6": 6,
    "7": 7,
    "8": 8,
    "9": 9,
    a: 10,
    b: 11,
    c: 12,
    d: 13,
    e: 14,
    f: 15,
    A: 10,
    B: 11,
    C: 12,
    D: 13,
    E: 14,
    F: 15,
};

// Why not use parseInt? It has some surprising behavior for input that'd be
// considered invalid for this use case.
//
// For example, parseInt("fg", 16) returns 16 as it will see 'f' as a valid hex
// digit but then stop parsing at 'g'.
//
// We do not want to allow partial parsing like that.
//
function parseHex2(hex: string): number {
    const hi = hexTable[hex[0]];
    const lo = hexTable[hex[1]];
    if (hi === undefined || lo === undefined) {
        throw new Error(`Invalid hex digit: ${hex}`);
    }
    return (hi << 4) | lo;
}
