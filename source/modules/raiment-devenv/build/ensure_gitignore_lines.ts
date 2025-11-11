#!/usr/bin/env -S deno run --allow-read --allow-write

/**
 * Ensures that specified lines exist in .gitignore
 * Usage: deno run --allow-read --allow-write ensure_gitignore_line.ts "line1" "line2" ...
 */

import { sh } from "@raiment-shell";

async function main() {
    const args = Deno.args;
    if (args.length === 0) {
        console.error("Usage: ensure_gitignore_line.ts <line1> [line2] ...");
        Deno.exit(1);
    }
    const gitignorePath = ".gitignore";

    let content: string;
    try {
        content = await Deno.readTextFile(gitignorePath);
    } catch (error) {
        if (error instanceof Deno.errors.NotFound) {
            content = "";
        } else {
            throw error;
        }
    }

    const existingLines = content.split("\n").map((line) => line.trimEnd());

    let modified = false;
    const linesToAdd: string[] = [];
    for (const arg of args) {
        const trimmedArg = arg.trim();
        const exists = existingLines.some((line) =>
            line.trimEnd() === trimmedArg
        );
        if (!exists) {
            linesToAdd.push(trimmedArg);
            sh.cprintln(
                `[+](dodgerblue) Added [${trimmedArg}](filename) to [.gitignore](filename)`,
            );
            modified = true;
        }
    }

    // If modifications needed, append and save
    if (modified) {
        let newContent = content;
        if (newContent.length > 0 && !newContent.endsWith("\n")) {
            newContent += "\n";
        }
        for (const line of linesToAdd) {
            newContent += line + "\n";
        }
        await Deno.writeTextFile(gitignorePath, newContent);
    } else {
        sh.cprintln("[:check:](green) [.gitignore](filename) up to date");
    }
}

main();
