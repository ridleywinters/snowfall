import React from "react";

/**
 * Updates the returned URL state whenever the URL changes, which causes a
 * re-render of that component. Optionally takes a callback which will be
 * called in the context of an Effect, for example if other state needs to be
 * updated.
 */
export function useURLChangeListener(callback?: (url: URL) => void): URL {
    const PATCH_ID = "data-use-url-listener-patch";

    const [url, setURL] = React.useState<URL>(new URL(globalThis.location.href));

    React.useEffect(() => {
        const handler = (...args: any[]) => {
            const url = new URL(globalThis.location.href);
            setURL(url);
            callback?.(url);
        };

        // Sigh...patching required...
        let unpatch: (() => void) | null = null;
        if (document.body.getAttribute(PATCH_ID) !== "true") {
            const originalPush = history.pushState;
            const originalReplace = history.replaceState;
            history.pushState = function (...args) {
                originalPush.apply(this, args);
                globalThis.dispatchEvent(new Event("patched-pushstate"));
            };
            history.replaceState = function (...args) {
                originalReplace.apply(this, args);
                globalThis.dispatchEvent(new Event("patched-replacestate"));
            };
            unpatch = () => {
                history.pushState = originalPush;
                history.replaceState = originalReplace;
                document.body.removeAttribute(PATCH_ID);
            };
            document.body.setAttribute(PATCH_ID, "true");
        }

        globalThis.addEventListener("popstate", handler);
        globalThis.addEventListener("hashchange", handler);
        globalThis.addEventListener("patched-pushstate", handler);
        globalThis.addEventListener("patched-replacestate", handler);

        return () => {
            globalThis.removeEventListener("popstate", handler);
            globalThis.removeEventListener("hashchange", handler);
            unpatch?.();
        };
    }, []);

    return url;
}
