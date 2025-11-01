import { ColorRGB } from "./types.ts";

export function parseHexColor(hex: string): ColorRGB {
    const digits = hex.startsWith("#") ? hex.slice(1) : hex;
    let r: number, g: number, b: number;
    if (digits.length === 3) {
        r = parseInt(digits[0] + digits[0], 16);
        g = parseInt(digits[1] + digits[1], 16);
        b = parseInt(digits[2] + digits[2], 16);
    } else {
        // 6-character hex
        r = parseInt(digits.substring(0, 2), 16);
        g = parseInt(digits.substring(2, 4), 16);
        b = parseInt(digits.substring(4, 6), 16);
    }
    return { r, g, b };
}
