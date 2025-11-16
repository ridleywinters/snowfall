import React from "react";
import { handleKeyMapping, KeyMappingTable } from "../util/key_mapping.ts";

export function useGlobalKeyMapping(
    keyMap: KeyMappingTable<KeyboardEvent>,
): void {
    React.useEffect(() => {
        const handler = (evt: KeyboardEvent) => {
            handleKeyMapping(evt, keyMap);
        };
        globalThis.addEventListener("keydown", handler);
        return () => {
            globalThis.removeEventListener("keydown", handler);
        };
    }, [keyMap]);
}
