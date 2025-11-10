/**
 * Defines a "no-op" CSS template literal tag function that simply marks the
 * string as template CSS string so that extensions like
 * "vscode-embedded-css-template-strings" can highlight, format, and
 * autocomplete the content as CSS.
 */
export function css(strings: TemplateStringsArray, ...values: any[]): string {
    return strings.reduce((result, str, i) => result + str + (values[i] || ""), "");
}
