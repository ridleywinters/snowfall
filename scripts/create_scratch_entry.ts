#!/usr/bin/env -S deno run --allow-read --allow-write --allow-run --allow-env

/**
 * Creates a temporary scratch file (that will not be committed to the repo) and
 * opens it in code-insiders.
 */

import { sh } from "@raiment-shell";

function formatDate(d: Date): string {
    const year = d.getFullYear();
    const month = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    return `${year}-${month}-${day}`;
}

async function main() {
    const date = new Date();
    const name = `${formatDate(date)}.md`;
    const tempDir = sh.expandEnvVars("$REPO_ROOT/temp");
    const path = `${tempDir}/${name}`;

    // Check if file exists; if not, create with a small header
    const doesExist = await sh.exists(path);
    if (doesExist) {
        sh.cprintln(`[:check:](green) Using existing [${path}](filename)`);
    } else {
        const content = `# ${formatDate(date)}\n\n`;
        await sh.write(path, content);
        sh.cprintln(`[+](dodgerblue) Created [${path}](filename)`);
    }

    // Find an editor: prefer `code-insiders`, fall back to `code` using `sh.which`.
    const editorPath = (await sh.which("code-insiders")) ?? (await sh.which("code"));
    const editor = editorPath ? editorPath : null;
    if (!editor) {
        sh.cprintln(
            "[!](yellow) Neither [code-insiders](filename) nor [code](filename) found on PATH.",
        );
        sh.cprintln("Install VS Code or run the script and open the file manually:");
        sh.cprintln(`[filename]: ${path}`);
        Deno.exit(1);
    }

    try {
        await sh.spawn(editor, [path]);
    } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        sh.cprintln(`[!](yellow) Failed to launch [${editor}](filename): ${msg}`);
        Deno.exit(1);
    }
}

if (import.meta.main) {
    main();
}
