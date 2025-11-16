/**
 * Style Language
 *
 * The "style language" hook provides a dynamic Tailwind-like syntax for
 * concisely specifying styles in React components.
 *
 * It is optimized for convenience of use and brevity. Runtime performance
 * may be a trade-off for the conveniences of development speed.
 *
 * Examples:
 *
 * px8      - adds padding-left and padding-right of 8px
 * my16     - adds margin-top and margin-bottom of 16px
 * mx-auto  - adds margin-left and margin-right of auto
 */

import React from "react";
import { hashString } from "../internal/hash_string.ts";

export type StyleLanguage = string | string[] | Record<string, boolean>;

/**
 * Given a "style language" descriptor, generates a unique, global CSS class
 * for that descriptor and returns its class name.
 *
 * If the style language results in zero styles, it will return an empty string.
 * This is a valid case and does not signal an error.
 *
 * It reference counts the generated class and only removes the class from the
 * global CSS when there are no more references to it.
 */
export function useStyleLanguage(sl: StyleLanguage | undefined): string {
    const styleTokens = preprocessStyleLanguage(sl);

    const className = React.useMemo(() => {
        if (styleTokens.length === 0) {
            return "";
        }
        // Use a hash, not simply something unique, in order to reuse identical
        // styles.
        const hash = hashString(styleTokens.join(" "));
        return `_SL${hash.toString(32)}`;
    }, [styleTokens]);

    React.useLayoutEffect(() => {
        // If the className is empty there's nothing to do
        if (!className) {
            return;
        }

        // ID of the style element created to store the style information.
        // This needs to map deterministically to the className to ensure
        // the ref-counting works.
        const id = `id-${className}`;

        const cleanup = () => {
            const el = document.getElementById(id);
            if (!el) {
                return;
            }
            const count = parseInt(el.dataset.count ?? "0");
            el.dataset.count = `${count - 1}`;
            if (count <= 1) {
                el.remove();
            }
        };

        let el = document.getElementById(id);
        if (el) {
            el.dataset.count = `${parseInt(el.dataset.count ?? "0") + 1}`;
        } else {
            el = document.createElement("style");
            el.id = id;
            el.innerHTML = compileStyleLanguage(className, styleTokens);
            el.dataset.count = "1";
            document.head.appendChild(el);
        }
        return cleanup;
    }, [className]);

    return className;
}

/**
 * Reduces the style specification into a flat array of style tokens.
 *
 * The input style specification can be provided as a single string,
 * an array of strings, or an object mapping strings (to conditionally
 * enable/disable sets of styles).  This is for convenience of the caller.
 */
function preprocessStyleLanguage(sl: StyleLanguage | undefined): string[] {
    if (!sl) {
        return [];
    }
    const splitString = (sl: string): string[] => {
        return sl.split(" ").map((s) => s.trim()).filter((s) => s.length > 0);
    };
    if (typeof sl === "string") {
        return splitString(sl);
    }
    if (Array.isArray(sl)) {
        return sl.map((s) => splitString(s)).flat();
    }
    return Object.entries(sl)
        .filter(([_, include]) => include)
        .map(([key, _]) => splitString(key))
        .flat();
}

function compileStyleLanguage(className: string, tokens: string[]): string {
    const lines: string[] = [];

    for (const token of tokens) {
        let found = false;
        for (const [regexOrString, fn] of rulesTable()) {
            // Check for a match (exact string match or regular expression match)
            // and get the output if there is one.
            let result: string | string[] | undefined;
            if (typeof regexOrString === "string") {
                if (token === regexOrString) {
                    result = fn([""]);
                }
            } else {
                const m = token.match(regexOrString);
                if (m) {
                    result = fn(m);
                }
            }
            if (!result) {
                continue;
            }

            // If there was a match, add it to the current class definition.
            const css = Array.isArray(result) ? result.join("\n") : result;
            lines.push(css);
            found = true;
            break;
        }

        // Warn if no matching rule was found for the token -- it's easy to introduce typos
        // in the style definition during development.
        if (!found) {
            console.warn(`Unknown style language token: '${token}' in: '${tokens.join(" ")}'`);
        }
    }

    // Return a CSS class definition. Note that this could include internal selectors
    // like "&:hover", so do not assume this will be a simple list of properties.
    return [`.${className} {`, ...lines, "}"].join("\n");
}

let compiledRulesTable: StyleLanguageRule[] | null = null;

function rulesTable(): StyleLanguageRule[] {
    if (!compiledRulesTable) {
        compiledRulesTable = RULES_TABLE_SOURCE.map(([matcher, fn]) => {
            if (matcher instanceof RegExp) {
                const source = matcher.source;
                if (source.startsWith("^") || source.endsWith("$")) {
                    console.error(
                        `Style language rule regexes must not include start (^) or end ($) anchors: /${source}/. `,
                        "These are added automatically by the compiler to ensure all rules are full matches.",
                    );
                    throw new Error(`Invalid style rule: /${source}/`);
                }

                matcher = new RegExp("^" + matcher.source + "$");
            }
            return [matcher, fn] as StyleLanguageRule;
        });
    }
    return compiledRulesTable;
}

type StyleLanguageRule =
    | [RegExp, (m: string[]) => string | string[]]
    | [string, () => string | string[]];

const RULES_TABLE_SOURCE: StyleLanguageRule[] = [
    //-------------------------------------------------------------------------
    // Positioning
    //
    // absolute, relative, fixed, sticky
    // top-*, right-*, bottom-*, left-*
    // z-*
    // border-box, content-box
    // display-none, display-block, display-inline, display-inline-block,
    // display-flex, display-grid
    //-------------------------------------------------------------------------

    [
        /(absolute|relative|fixed|sticky)/,
        (m) => `position: ${m[0]};`,
    ],
    [
        /(top|right|bottom|left)-?([0-9]+)/,
        (m) => `${m[1]}: ${m[2]}px;`,
    ],
    [
        /z-?([0-9]+)/,
        (m) => `z-index: ${m[1]};`,
    ],
    [
        /(border|content)-box/,
        (m) => `box-sizing: ${m[1]};;`,
    ],
    [
        /display-(none|block|inline|inline-block|flex|grid)/,
        (m) => `display: ${m[1]};`,
    ],

    //-------------------------------------------------------------------------
    // Dimensions
    //
    // width-*, height-*
    // min-width-*, max-width-*, min-height-*, max-height-*
    // border-box, content-box
    //-------------------------------------------------------------------------

    [
        /(width|height|min-width|min-height|max-width|max-height)-([0-9]+)%/,
        (m) => `${m[1]}: ${m[2]}%;`,
    ],
    [
        /(width|height|min-width|min-height|max-width|max-height)-([0-9]+)/,
        (m) => `${m[1]}: ${m[2]}px;`,
    ],
    [
        /(width|height|min-width|min-height|max-width|max-height)-(.+)/,
        (m) => `${m[1]}: ${m[2]};`,
    ],
    [
        /(border-box|content-box)/,
        (m) => `box-sizing: ${m[1]};`,
    ],
    [
        /overflow-(auto|hidden|visible|scroll)/,
        (m) => `overflow: ${m[1]};`,
    ],

    //-------------------------------------------------------------------------
    // Flexbox
    //
    // flex-row, flex-col
    // flex-row-center
    // flex-*-*-*
    // gap-*
    //-------------------------------------------------------------------------

    [
        "flex",
        () => "display: flex;",
    ],
    [
        "flex-row",
        () => "display: flex; flex-direction: row;",
    ],
    [
        "flex-row-center",
        () => "display: flex; flex-direction: row; align-items: center;",
    ],
    [
        /flex-row-(start|end)/,
        (m) => `display: flex; flex-direction: row; align-items: flex-${m[1]};`,
    ],
    [
        "flex-col",
        () => "display: flex; flex-direction: column;",
    ],
    [
        "flex-wrap",
        () => "flex-wrap: wrap;",
    ],
    [
        /flex-([0-9A-Za-z%]+)-([0-9A-Za-z%]+)-([0-9A-Za-z%]+)/,
        (m) => `flex: ${m[1]} ${m[2]} ${m[3]};`,
    ],
    [
        /grow-([0-9]+)/,
        (m) => `flex-grow: ${m[1]};`,
    ],
    [
        /gap-([0-9]+)/,
        (m) => `gap: ${m[1]}px;`,
    ],
    [
        /align-(start|end)/,
        (m) => `align-items: flex-${m[1]};`,
    ],
    [
        /align-(flex-start|flex-end|center|stretch|baseline)/,
        (m) => `align-items: ${m[1]};`,
    ],
    [
        /justify-(start|end)/,
        (m) => `justify-content: flex-${m[1]};`,
    ],
    [
        /justify-(flex-start|flex-end|center|space-between|space-around|space-evenly)/,
        (m) => `justify-content: ${m[1]};`,
    ],

    //-------------------------------------------------------------------------
    // Padding & Margins
    //
    // p*, px*, py*, pr*, pl*, pt*, pb*
    // m*, mx*, my*, mr*, ml*, mt*, mb*
    // mx-auto
    //-------------------------------------------------------------------------
    // Padding
    [
        /p-?([0-9]+)/,
        (m) => `padding: ${m[1]}px;`,
    ],
    [
        /pr-?([0-9]+)/,
        (m) => `padding-right: ${m[1]}px;`,
    ],
    [
        /pl-?([0-9]+)/,
        (m) => `padding-left: ${m[1]}px;`,
    ],
    [
        /pt-?([0-9]+)/,
        (m) => `padding-top: ${m[1]}px;`,
    ],
    [
        /pb-?([0-9]+)/,
        (m) => `padding-bottom: ${m[1]}px;`,
    ],
    [
        /px-?([0-9]+)/,
        (m) => `padding-left: ${m[1]}px; padding-right: ${m[1]}px;`,
    ],
    [
        /py-?([0-9]+)/,
        (m) => `padding-top: ${m[1]}px; padding-bottom: ${m[1]}px;`,
    ],

    // Margins
    [
        /m-?([0-9]+)/,
        (m) => `margin: ${m[1]}px;`,
    ],
    [
        /mr-?([0-9]+)/,
        (m) => `margin-right: ${m[1]}px;`,
    ],
    [
        /ml-?([0-9]+)/,
        (m) => `margin-left: ${m[1]}px;`,
    ],
    [
        /mt-?([0-9]+)/,
        (m) => `margin-top: ${m[1]}px;`,
    ],
    [
        /mb-?([0-9]+)/,
        (m) => `margin-bottom: ${m[1]}px;`,
    ],
    [
        /mx-?([0-9]+)/,
        (m) => `margin-left: ${m[1]}px; margin-right: ${m[1]}px;`,
    ],
    [
        /my-?([0-9]+)/,
        (m) => `margin-top: ${m[1]}px; margin-bottom: ${m[1]}px;`,
    ],
    [
        /mx-auto/,
        () => "margin-left: auto; margin-right: auto;",
    ],

    //-------------------------------------------------------------------------
    // Typography
    //
    // bold|string
    // em|italic
    // font-size-*
    // line-height-*
    //-------------------------------------------------------------------------

    [
        /(bold|string)/,
        () => "font-weight: bold;",
    ],
    [
        /(em|italic)/,
        () => "font-style: italic;",
    ],
    [
        /font-size-([0-9]+)%/,
        (m) => `font-size: ${m[1]}%;`,
    ],
    [
        /font-size-([0-9]+)/,
        (m) => `font-size: ${m[1]}px;`,
    ],
    [
        /line-height-([0-9]+)%/,
        (m) => `line-height: ${m[1]}%;`,
    ],
    [
        /line-height-([0-9\.]+)/,
        (m) => `line-height: ${m[1]};`,
    ],
    [
        /text-(left|center|right|justify)/,
        (m) => `text-align: ${m[1]};`,
    ],
    [
        "no-wrap",
        () => "white-space: nowrap;",
    ],

    //-------------------------------------------------------------------------
    // Cursor
    //-------------------------------------------------------------------------

    [
        /cursor-(auto|default|pointer|wait|text|move|help|not-allowed)/,
        (m) => `cursor: ${m[1]};`,
    ],

    //-------------------------------------------------------------------------
    // Colors
    //-------------------------------------------------------------------------
    [
        /opacity-([0-9]+)%?/,
        (m) => `opacity: ${parseInt(m[1]) / 100};`,
    ],

    [
        /fg-(white|black|red|green|blue)/,
        (m) => `color: ${m[1]};`,
    ],
    [
        /fg-gray-([0-9]+)%?/,
        (m) => `color: hsl(0, 0%, ${m[1]}%);`,
    ],
    [
        /fg-#([0-9A-Za-z]+)/,
        (m) => `color: #${m[1]};`,
    ],
    [
        /fg-hover-#([0-9A-Za-z]+)/,
        (m) => `&:hover { color: #${m[1]}; }`,
    ],
    [
        /bg-(white|black|red|green|blue|transparent)/,
        (m) => `background-color: ${m[1]};`,
    ],
    [
        /bg-gray-([0-9]+)%?/,
        (m) => `background-color: hsl(0, 0%, ${m[1]}%);`,
    ],
    [
        /bg-#([0-9A-Za-z]+)/,
        (m) => `background-color: #${m[1]};`,
    ],
    [
        /bg-hover-#([0-9A-Za-z]+)/,
        (m) => `&:hover { background-color: #${m[1]}; }`,
    ],

    //-------------------------------------------------------------------------
    // Borders
    //-------------------------------------------------------------------------

    [
        /border-(none|solid|dashed|dotted)/,
        (m) => `border-style: ${m[1]};`,
    ],
    [
        /border-(bottom|top|left|right)-(none|solid|dashed|dotted)-#([0-9A-Za-z]+)/,
        (m) => `border-${m[1]}: 1px ${m[2]} #${m[3]};`,
    ],
    [
        /border-(bottom|top|left|right)-#([0-9A-Za-z]+)/,
        (m) => `border-${m[1]}: 1px solid #${m[2]};`,
    ],
    [
        /border-#([0-9A-Za-z]+)/,
        (m) => `border: 1px solid #${m[1]};`,
    ],
    [
        /border-(width|radius)-([0-9]+)/,
        (m) => `border-${m[1]}: ${m[2]}px;`,
    ],
    [
        "outline-none",
        () => "outline: none;",
    ],

    //-------------------------------------------------------------------------
    // User select
    //-------------------------------------------------------------------------
    [
        /select-(none|text|all|auto)/,
        (m) => `user-select: ${m[1]};`,
    ],
];
