import { HexColor, HSLF32, RGBU8 } from "./types.ts";

export function rgbau8ArrayToHex7(
    rgba: Uint8Array | [number, number, number, number],
): HexColor {
    if (rgba.length !== 4) {
        throw new Error(
            `Expected RGBA array of length 4 but got length ${rgba.length}`,
        );
    }
    const rHex = rgba[0].toString(16).padStart(2, "0");
    const gHex = rgba[1].toString(16).padStart(2, "0");
    const bHex = rgba[2].toString(16).padStart(2, "0");
    return `#${rHex}${gHex}${bHex}`;
}

export function rgbu8ToHSLF32(rgb: RGBU8): HSLF32 {
    const r = rgb.r / 255;
    const g = rgb.g / 255;
    const b = rgb.b / 255;

    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    let h = 0;
    let s = 0;
    const l = (max + min) / 2;

    if (max !== min) {
        const d = max - min;
        s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
        switch (max) {
            case r:
                h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
                break;
            case g:
                h = ((b - r) / d + 2) / 6;
                break;
            case b:
                h = ((r - g) / d + 4) / 6;
                break;
        }
    }

    if (h < 0 || h > 1) {
        throw new Error(`Calculated hue out of range: ${h}`);
    }
    if (s < 0 || s > 1) {
        throw new Error(`Calculated saturation out of range: ${s}`);
    }
    if (l < 0 || l > 1) {
        throw new Error(`Calculated lightness out of range: ${l}`);
    }

    return { h, s, l };
}
