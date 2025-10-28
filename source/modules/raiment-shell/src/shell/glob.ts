import { expandGlob } from "@std/fs/expand-glob";
import { expandEnvVars } from "./expand_env_vars.ts";

export async function glob(pattern: string): Promise<string[]> {
    // Expand environment variables in the pattern
    const expandedPattern = expandEnvVars(pattern);

    const files: string[] = [];
    for await (const entry of expandGlob(expandedPattern)) {
        if (entry.isFile) {
            files.push(entry.path);
        }
    }
    return files;
}
