#!/usr/bin/env -S deno run --allow-read --allow-write --allow-run --allow-env

import { basename, dirname, relative } from "@std/path";
import { sh } from "@raiment-shell";

const SCRIPTS = sh.template("$REPO_ROOT/source/assets/build");
const CONTENT_DIR = sh.template("$REPO_ROOT/source/content/sprites");
const OUTPUT_DIR = "./base/sprites";

type AssetTask = {
    sourceImage: string;
    sourceMeta: string;
    outputImage: string;
    outputMeta: string;
};

async function main(): Promise<void> {
    await sh.mkdir(OUTPUT_DIR);

    const files = await sh.glob(`${CONTENT_DIR}/*.aseprite`);
    const tasks: AssetTask[] = files.map((file) => {
        const baseName = basename(file, ".aseprite");
        return {
            sourceImage: file,
            sourceMeta: `${dirname(file)}/attribution.meta.md`,
            outputImage: `${OUTPUT_DIR}/${baseName}.png`,
            outputMeta: `${OUTPUT_DIR}/${baseName}.meta.md`,
        };
    });

    for (const task of tasks) {
        const script = `${SCRIPTS}/aseprite_to_png.ts`;
        await sh.exec(script, ["-i", task.sourceImage, "-o", task.outputImage]);
        await Deno.copyFile(task.sourceMeta, task.outputMeta);
        sh.cprintln(
            `[:check:](green) built sprite [${relative(".", task.outputImage)}](goldenrod)`,
        );
    }
}

main();
