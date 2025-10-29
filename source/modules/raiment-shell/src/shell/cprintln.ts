import * as colors from "@std/fmt/colors";
import { NAMED_COLORS } from "@raiment-core";
import { template } from "./template.ts";

export function cprintln(message: string, params: Record<string, any> = {}): void {
    // Replace template variables first
    message = message.replace(
        /\{\{([A-Za-z_][A-Za-z0-9_]*)\}\}/g,
        (_, varName) => {
            if (params[varName] !== undefined) {
                return params[varName];
            }
            return "";
        },
    );

    // Replace Markdown-like [text](color) with colored text
    message = message.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_, text: string, color: string) => {
        const colorName = color.toLowerCase();
        const colorHex = NAMED_COLORS[colorName] || "#ff00ff";
        const colorNumber = parseInt(colorHex.replace("#", ""), 16);
        return colors.rgb24(text, colorNumber);
    });

    // Replace special character patterns last
    const patterns: Record<string, string> = {
        ":check:": "✓",
    };
    for (const key in patterns) {
        message = message.replace(new RegExp(key, "g"), patterns[key]);
    }

    console.log(message);
}
