#!/usr/bin/env -S deno run --allow-read --allow-write

/**
 * Replaces the "settings" value in a VS Code workspace file with the contents of a settings.json file
 * Usage: deno run --allow-read --allow-write vscode_settings_to_workspace.ts <settings.json> <directory>
 */

import { sh } from "@raiment-shell";
import { parse as parseJsonc } from "jsr:@std/jsonc";
import { core } from "@raiment-core";

async function main() {
    const args = Deno.args;
    if (args.length !== 2) {
        console.error(
            "Usage: vscode_settings_to_workspace.ts <settings.json> <directory>",
        );
        Deno.exit(1);
    }

    const [settingsPath, directory] = args;

    // Find .code-workspace file in directory
    const workspaceFiles: string[] = [];
    try {
        for await (const entry of Deno.readDir(directory)) {
            if (entry.isFile && entry.name.endsWith(".code-workspace")) {
                workspaceFiles.push(`${directory}/${entry.name}`);
            }
        }
    } catch (error) {
        if (error instanceof Deno.errors.NotFound) {
            sh.cprintln(
                `[!](red) Directory not found: [${directory}](filename)`,
            );
            Deno.exit(1);
        } else {
            throw error;
        }
    }

    if (workspaceFiles.length === 0) {
        sh.cprintln(
            `[:info:](yellow) No .code-workspace file found in [${directory}](filename)`,
        );
        const baseName = Deno.cwd().split("/").filter(Boolean).pop() || "workspace";
        const workspaceFile = `${baseName}.code-workspace`;
        sh.write(
            workspaceFile,
            JSON.stringify({ folders: [{ path: "." }], settings: {} }, null, 4) + "\n",
        );
        workspaceFiles.push(`${directory}/${workspaceFile}`);
    }

    if (workspaceFiles.length > 1) {
        sh.cprintln(
            `[!](red) Multiple .code-workspace files found in [${directory}](filename):`,
        );
        for (const file of workspaceFiles) {
            sh.cprintln(`  - [${file}](filename)`);
        }
        Deno.exit(1);
    }

    const workspacePath = workspaceFiles[0];

    let settingsContent: string;
    try {
        settingsContent = await Deno.readTextFile(settingsPath);
    } catch (error) {
        if (error instanceof Deno.errors.NotFound) {
            sh.cprintln(
                `[!](red) Settings file not found: [${settingsPath}](filename)`,
            );
            Deno.exit(1);
        } else {
            throw error;
        }
    }

    let settingsJSON: Record<string, unknown>;
    try {
        settingsJSON = parseJsonc(settingsContent) as Record<string, unknown>;
    } catch (error) {
        sh.cprintln(
            `[!](red) Invalid JSONC in settings file: [${settingsPath}](filename)`,
        );
        throw error;
    }

    // Load workspace file
    let workspaceContent: string;
    try {
        workspaceContent = await Deno.readTextFile(workspacePath);
    } catch (error) {
        sh.cprintln(
            `[!](red) Failed to read workspace file: [${workspacePath}](filename)`,
        );
        throw error;
    }

    // Parse workspace file
    let workspaceJSON: Record<string, unknown>;
    try {
        workspaceJSON = parseJsonc(workspaceContent) as Record<string, unknown>;
    } catch (error) {
        sh.cprintln(
            `[!](red) Invalid JSONC in workspace file: [${workspacePath}](filename)`,
        );
        throw error;
    }

    // Replace settings in workspace
    workspaceJSON.settings ??= {};
    core.assignDeep(workspaceJSON.settings, settingsJSON);

    // Write updated workspace file
    const updatedContent = JSON.stringify(workspaceJSON, null, 4) + "\n";
    await Deno.writeTextFile(workspacePath, updatedContent);

    sh.cprintln(
        `[:check:](green) Updated [settings](filename) in [${workspacePath}](filename)`,
    );
}

main();
