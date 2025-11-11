import { normalize as pathNormalize } from "https://deno.land/std@0.208.0/path/mod.ts";

export function normalize(path: string): string {
    return pathNormalize(path);
}
