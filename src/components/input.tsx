import React, { JSX } from "react";
import { Element, TagProps } from "./elements.tsx";

type InputProps =
    & Omit<TagProps<"input">, "tag">
    & {
        onKeyMap?: KeyMappingTable<HTMLInputElement>;
    };

export function Input(
    {
        onKeyDown,
        onKeyMap,
        ...rest
    }: InputProps,
): JSX.Element {
    const handleKeyDown = (evt: React.KeyboardEvent<HTMLInputElement>) => {
        if (onKeyMap) {
            handleKeyMapping(evt, onKeyMap);
        }
        onKeyDown?.(evt);
    };
    return (
        <Element
            tag="input"
            onKeyDown={handleKeyDown}
            {...rest as any}
        />
    );
}

type KeyMappingTable<T> = Record<string, (evt: React.KeyboardEvent<T>) => void>;

function handleKeyMapping<T>(
    evt: React.KeyboardEvent<T>,
    table: KeyMappingTable<T>,
): void {
    let key = evt.key;
    if (evt.ctrlKey || evt.metaKey) {
        key = `Ctrl+${key}`;
    }
    if (evt.altKey) {
        key = `Alt+${key}`;
    }
    if (evt.shiftKey) {
        key = `Shift+${key}`;
    }

    const tableKeys = Object.keys(table);
    let handler = undefined;
    let stopEvent = true;
    for (let i = 0; i < tableKeys.length; i++) {
        const originalKey = tableKeys[i];
        let k = originalKey;
        if (k.endsWith("?")) {
            k = k.slice(0, -1);
            stopEvent = false;
        }
        if (k !== key) {
            continue;
        }
        handler = table[originalKey];
        break;
    }
    if (handler) {
        if (stopEvent) {
            evt.preventDefault();
            evt.stopPropagation();
        }
        handler(evt);
    }
}
