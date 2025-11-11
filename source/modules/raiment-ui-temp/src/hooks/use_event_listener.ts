import { EventEmitter } from "@raiment-core";
import React from "react";

/**
 * useEventListener hook
 *
 * A hook specifically for the Raiment EventEmitter class.  It will subscribe to an
 * event or events on the emitter and both (1) trigger a rerender of the component
 * when that event is emitted and (2) optionally call a callback function as well.
 *
 * The emitter also returns a "generation" number that will on each event which
 * can be used, for example, in a dependency array to trigger other effects.
 */
export function useEventListener<S extends Record<string, any>>(
    emitter: EventEmitter<S> | undefined,
    eventNamesParam: keyof S | (keyof S)[],
    callback?: () => void,
): number {
    const [generation, setGeneration] = React.useState<number>(0);
    React.useEffect(() => {
        if (!emitter) {
            return;
        }
        const listener = () => {
            setGeneration((gen: number) => gen + 1);
            callback?.();
        };
        const eventNames = Array.isArray(eventNamesParam) ? eventNamesParam : [eventNamesParam];
        for (const event of eventNames) {
            emitter.on(event, listener);
        }
        return () => {
            for (const event of eventNames) {
                emitter.off(event, listener);
            }
        };
    }, [emitter, eventNamesParam]);
    return generation;
}
