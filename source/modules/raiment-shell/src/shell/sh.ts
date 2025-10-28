import { cprintln } from "./cprintln.ts";
import { exec } from "./exec.ts";
import { spawn } from "./spawn.ts";
import { expandEnvVars } from "./expand_env_vars.ts";
import { glob } from "./glob.ts";
import { mkdir } from "./mkdir.ts";
import { template } from "./template.ts";

async function copy(source: string, destination: string): Promise<void> {
    return await Deno.copyFile(source, destination);
}

export const sh = {
    // String utilities
    expandEnvVars,

    template,

    // Console
    cprintln,

    // Commands
    exec, // pipes to stdout/stderr (retaining streaming, colors, etc.)
    spawn, // returns stdout

    // Filesystem
    mkdir,
    glob,
    copy,
};
