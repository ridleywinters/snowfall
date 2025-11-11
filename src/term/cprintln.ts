import * as core from "@raiment-core";
import * as colors from "@std/fmt/colors";

type Params = Record<string, any>;

export function cprintln(
    messageOrColor?: string,
    paramsOrMessage: string | Params = {},
    paramsArg?: Params,
): void {
    if (messageOrColor === undefined) {
        console.log();
        return;
    }

    let message: string;
    let defaultColor: string | undefined;
    let params: Params;
    if (typeof paramsOrMessage === "object") {
        params = paramsOrMessage;
        message = messageOrColor;
    } else {
        defaultColor = messageOrColor;
        message = paramsOrMessage;
        params = paramsArg ?? {};
    }

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
        const colorHex = calculateColor(colorName);
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

    if (defaultColor) {
        const colorHex = calculateColor(defaultColor.toLowerCase());
        const colorNumber = parseInt(colorHex.replace("#", ""), 16);
        message = colors.rgb24(message, colorNumber);
    }
    console.log(message);
}

const COLOR_TABLE = {
    ...core.NAMED_COLORS,
    ...core.SEMANTIC_COLORS,
};

function calculateColor(text: string): core.ColorHexString {
    if (text.startsWith("#")) {
        // Expand shorthand
        if (text.length === 4) {
            const r = text[1];
            const g = text[2];
            const b = text[3];
            text = `#${r}${r}${g}${g}${b}${b}`;
        }
        return text as core.ColorHexString;
    }
    return COLOR_TABLE[text] || "#ff00ff";
}
