import { dirname as pathDirname } from "https://deno.land/std@0.208.0/path/mod.ts";

export function dirname(path: string): string {
    return pathDirname(path);
}
