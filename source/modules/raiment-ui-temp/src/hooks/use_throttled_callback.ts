// deno-lint-ignore-file no-explicit-any

import React from "react";

/**
 * Creates a throttled version of a callback that fires at most once every `delay` ms.
 * Ensures the callback is called at least once during continuous input.
 */
export function useThrottledCallback<T extends (...args: any[]) => void>(
    callback: T,
    delay: number,
): T {
    type State = {
        timeout: number | null;
        prevCall: number;
        pendingArgs: any[] | null;
    };
    const stateRef = React.useRef<State>({
        timeout: null,
        prevCall: 0,
        pendingArgs: null,
    });

    return React.useCallback((...args: any[]) => {
        const state = stateRef.current;
        const now = Date.now();
        const timeDelta = now - state.prevCall;

        if (state.timeout) {
            clearTimeout(state.timeout);
        }

        // Enough time has passed, call immediately
        if (timeDelta >= delay) {
            state.prevCall = now;
            state.pendingArgs = null;
            callback(...args);
            return;
        }

        // Subtle, but useful special-case: since useThrottledCallback is often used
        // in React event handlers and React event objects are pooled and reused, we want
        // to extract the event from the pool if we're going to be using it later.
        //
        // Note: another approach would be to have the caller save the event
        // parameters it needs before passing them to the throttled callback (since
        // this often may just be the target value). That unfortunately makes the call
        // less wieldy (especially if chaining callbacks where some ancestor component
        // "shouldn't" even know the event is being throttled) and this library
        // optimizes for convenience in usage, so we're choosing to put in this subtle
        // special-case instead.
        //
        state.pendingArgs = args.map((arg: any) => {
            if (
                arg && typeof arg === "object" &&
                typeof arg["persist"] === "function"
            ) {
                arg.persist();
            }
            return arg;
        });

        // Note: timeDelta must be >= delay at this point per the earlier conditional
        const actualDelay = delay - timeDelta;
        state.timeout = setTimeout(() => {
            state.prevCall = Date.now();
            state.pendingArgs = null;
            callback(...state.pendingArgs!);
        }, actualDelay);
    }, [callback, delay]) as T;
}
