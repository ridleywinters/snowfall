import React from "react";
import { hashString } from "../internal/hash_string.ts";

/**
 * Injects CSS into the document and removes it when the hook goes out of scope.
 *
 * If the `scope` is "local", it returns the class name of the local CSS style which
 * encapsulates all the rules within the style definitions.
 *
 * If the `scope` is "global", it will return a unique string based on the CSS content,
 * but it does not represent a class name and should not be used as such.
 */
export function useCSS(scope: "global" | "local", cssString: string | undefined): string {
    const className = React.useMemo(() => {
        const content = cssString?.trim();
        if (!content) {
            return "";
        }

        // Use a hash, not simply something unique, in order to reuse identical
        // styles.
        const hash = hashString(content + scope);
        return `_CSS${hash.toString(32)}`;
    }, [cssString, scope]);

    React.useLayoutEffect(() => {
        // If the className is empty there's nothing to do
        if (!className || !cssString) {
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
            const content = scope === "local" ? `.${className} { ${cssString} }` : cssString;

            el = document.createElement("style");
            el.id = id;
            el.innerHTML = content;
            el.dataset.count = "1";
            document.head.appendChild(el);
        }
        return cleanup;
    }, [className]);

    return className;
}
