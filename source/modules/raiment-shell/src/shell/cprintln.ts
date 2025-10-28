import * as colors from "@std/fmt/colors";
import { NAMED_COLORS } from "../../../raiment-core/src/colors/named_colors.ts";

export function cprintln(message: string): void {
    // Replace Markdown-like [text](color) with colored text
    message = message.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_, text: string, color: string) => {
        const colorName = color.toLowerCase();
        const colorHex = NAMED_COLORS[colorName] || "#ff00ff";
        const colorNumber = parseInt(colorHex.replace("#", ""), 16);
        return colors.rgb24(text, colorNumber);
    });
    const patterns: Record<string, string> = {
        ":check:": "✓",
    };
    for (const key in patterns) {
        message = message.replace(new RegExp(key, "g"), patterns[key]);
    }

    console.log(message);
}
