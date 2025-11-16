import React, { JSX } from "react";
import { Element, TagProps } from "./elements.tsx";
import { handleKeyMapping, KeyMappingTable } from "../util/key_mapping.ts";

type InputProps =
    & Omit<TagProps<"input">, "tag">
    & {
        onKeyMap?: KeyMappingTable<React.KeyboardEvent<HTMLInputElement>>;
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
