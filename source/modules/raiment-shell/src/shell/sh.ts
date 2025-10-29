import { copy } from "./copy.ts";
import { cprintln } from "./cprintln.ts";
import { exec } from "./exec.ts";
import { expandEnvVars } from "./expand_env_vars.ts";
import { glob } from "./glob.ts";
import { mkdir } from "./mkdir.ts";
import { read } from "./read.ts";
import { write } from "./write.ts";
import { spawn } from "./spawn.ts";
import { template } from "./template.ts";

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
    write,
    glob,
    copy,
    read,
};
