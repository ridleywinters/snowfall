import { parse } from "@std/yaml";

export function parseYAML<T>(yaml: string): T {
    return parse(yaml) as T;
}
