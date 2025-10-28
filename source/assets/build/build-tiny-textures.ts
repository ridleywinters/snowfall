#!/usr/bin/env -S deno run --allow-read --allow-write --allow-run --allow-env
import { relative } from "@std/path";
import { sh } from "@raiment-shell";

const files: Record<string, string> = {
    "stone_1": "tiny_textures/pack-3/512x512/Stone/Stone_16-512x512.png",
    "stone_2": "tiny_textures/pack-3/512x512/Stone/Stone_06-512x512.png",
};

const sourceDir = sh.template("$REPO_ROOT/extern/expanded");

for (const [target, source] of Object.entries(files)) {
    const script = sh.template("$REPO_ROOT/source/assets/build/extract_texture.ts");
    const outfile = `./base/textures/${target}.png`;
    await sh.exec(script, [
        `${sourceDir}/${source}`,
        `${sourceDir}/tiny_textures/attribution.meta.md`,
        "64x64",
        outfile,
    ]);

    sh.cprintln(
        `[:check:](green) built texture [${relative(".", outfile)}](goldenrod)`,
    );
}
