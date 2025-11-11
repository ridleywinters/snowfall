import { relative as pathRelative } from "https://deno.land/std@0.208.0/path/mod.ts";

export function relative(from: string, to: string): string {
    return pathRelative(from, to);
}
