import { parseHexColor } from "../colors/parse_hex_color.ts";
import { HexColor, RGBAU8 } from "../colors/types.ts";

export function stringifyGIMPPalette(colors: HexColor[]): string {
    const lines: string[] = [];
    lines.push("GIMP Palette");
    lines.push("#");
    colors.forEach((hex, index) => {
        // Format: R G B Name (3 characters wide for RGB values)
        const rgb = parseHexColor(hex);
        if ((rgb as RGBAU8).a !== undefined) {
            throw new Error("Unexpected alpha channel in color: " + hex);
        }
        const sr = rgb.r.toString().padStart(3, " ");
        const sg = rgb.g.toString().padStart(3, " ");
        const sb = rgb.b.toString().padStart(3, " ");
        lines.push(`${sr} ${sg} ${sb}\tColor ${index}`);
    });
    return lines.join("\n");
}
