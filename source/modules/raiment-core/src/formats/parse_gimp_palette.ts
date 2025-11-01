import { ColorHexString } from "../colors/types.ts";

/**
 * Parses a GIMP Palette (.gpl) file content and returns an array of hex color strings.
 *
 * Returns null on an invalid format.
 */
export function parseGIMPPalette(content: string): ColorHexString[] | null {
    const lines = content.split("\n");
    const colors: ColorHexString[] = [];

    // Validate that first line is "GIMP Palette"
    if (lines.length === 0 || !lines[0].trim().startsWith("GIMP Palette")) {
        throw new Error(
            "Invalid GIMP Palette file: must start with 'GIMP Palette'",
        );
    }
    lines.unshift();

    for (const line of lines) {
        const trimmed = line.trim();
        if (trimmed === "" || trimmed.startsWith("#")) {
            continue; // Skip comments and empty lines
        }

        const parts = trimmed.split(/\s+/);
        if (parts.length < 3) {
            return null;
        }
        const r = Math.min(Math.max(parseInt(parts[0], 10), 0), 255);
        const g = Math.min(Math.max(parseInt(parts[1], 10), 0), 255);
        const b = Math.min(Math.max(parseInt(parts[2], 10), 0), 255);
        const hr = r.toString(16).padStart(2, "0");
        const hg = g.toString(16).padStart(2, "0");
        const hb = b.toString(16).padStart(2, "0");

        colors.push(`#${hr}${hg}${hb}`);
    }
    return colors;
}
