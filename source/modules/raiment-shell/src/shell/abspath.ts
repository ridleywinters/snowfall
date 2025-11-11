import { resolve } from "https://deno.land/std@0.208.0/path/mod.ts";

export function abspath(path: string): string {
    return resolve(path);
}
