import { cprintln } from "../term/cprintln.ts";
import { abspath } from "./abspath.ts";
import { basename } from "./basename.ts";
import { copy } from "./copy.ts";
import { dirname } from "./dirname.ts";
import { exec } from "./exec.ts";
import { expandEnvVars } from "./expand_env_vars.ts";
import { fetchExists } from "./fetch_exists.ts";
import { which } from "./which.ts";
import { exists } from "./exists.ts";
import { fetchText } from "./fetch_text.ts";
import { glob } from "./glob.ts";
import { mkdir } from "./mkdir.ts";
import { normalize } from "./normalize.ts";
import { read } from "./read.ts";
import { relative } from "./relative.ts";
import { spawn } from "./spawn.ts";
import { template } from "./template.ts";
import { write } from "./write.ts";

export const sh = {
    // String utilities
    expandEnvVars,

    template,

    // Console
    cprintln,

    // CommandsP
    exec, // pipes to stdout/stderr (retaining streaming, colors, etc.)
    spawn, // returns stdout

    // Filesystem
    mkdir,
    write,
    glob,
    copy,
    read,
    dirname,
    basename,
    normalize,
    abspath,
    relative,
    exists,
    which,

    // Network
    fetchText,
    fetchExists,
};
