import { parseHexColor } from "../colors/parse_hex_color.ts";
import { ColorHexString } from "../colors/types.ts";

export function stringifyGIMPPalette(colors: ColorHexString[]): string {
    const lines: string[] = [];
    lines.push("GIMP Palette");
    lines.push("#");
    colors.forEach((hex, index) => {
        // Format: R G B Name (3 characters wide for RGB values)
        const { r, g, b } = parseHexColor(hex);
        const sr = r.toString().padStart(3, " ");
        const sg = g.toString().padStart(3, " ");
        const sb = b.toString().padStart(3, " ");
        lines.push(`${sr} ${sg} ${sb}\tColor ${index}`);
    });
    return lines.join("\n");
}
