import { basename as pathBasename } from "https://deno.land/std@0.208.0/path/mod.ts";

export function basename(path: string, suffix?: string): string {
    return pathBasename(path, suffix);
}
