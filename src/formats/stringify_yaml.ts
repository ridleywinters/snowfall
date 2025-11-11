import { stringify } from "@std/yaml";

export type StringifyYAMLOptions = {
    lineWidth?: number;
    indent?: number;
    condenseFlow?: boolean;
};

export function stringifyYAML(
    data: unknown,
    options?: StringifyYAMLOptions,
): string {
    return stringify(data, {
        lineWidth: 120,
        indent: 2,
        condenseFlow: true,
        ...options,
    });
}
