#!/usr/bin/env -S deno run --allow-net=download.blender.org --allow-write --allow-read --allow-run

import { sh } from "@raiment-shell";

const RELEASE_ROOT = "https://download.blender.org/release";

/** Platform mapping to Blender artifact tags and preferred extensions. */
function detectPlatform(): { tag: string; extensions: string[] } {
    const os = Deno.build.os; // "linux" | "darwin" | "windows"
    const arch = Deno.build.arch; // "x86_64" | "aarch64" | others

    if (os === "linux") {
        const tag = (arch === "aarch64") ? "linux-aarch64" : "linux-x64";
        return { tag, extensions: ["tar.xz"] };
    }
    if (os === "darwin") {
        const tag = (arch === "aarch64") ? "macos-arm64" : "macos-x64";
        return { tag, extensions: ["dmg", "zip"] };
    }
    throw new Error(`Unsupported OS: ${os} ${arch}`);
}

/** Semantic version compare: returns negative/zero/positive like strcmp. */
function cmpSemver(a: string, b: string): number {
    const pa = a.split(".").map(Number);
    const pb = b.split(".").map(Number);
    for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
        const ai = pa[i] ?? 0, bi = pb[i] ?? 0;
        if (ai !== bi) return ai - bi;
    }
    return 0;
}

/** Find the latest available major.minor series (e.g., "4.5"). */
async function latestSeries(): Promise<string> {
    const html = await sh.fetchText(`${RELEASE_ROOT}/`);
    // Matches "Blender4.5/" etc.
    const series = [...html.matchAll(/Blender(\d+\.\d+)\//g)].map((m) => m[1]);
    if (series.length === 0) {
        throw new Error("No series found in release index.");
    }
    series.sort((a, b) => cmpSemver(a, b));
    return series[series.length - 1];
}

/** Find latest X.Y.Z in a given series (X.Y). */
async function latestVersionInSeries(series: string): Promise<string> {
    const html = await sh.fetchText(`${RELEASE_ROOT}/Blender${series}/`);
    // Matches file stems like "blender-4.5.2-"
    const versions = [...html.matchAll(/blender-((\d+\.\d+)\.\d+)-/g)]
        .map((m) => m[1])
        .filter((v) => v.startsWith(series + "."));
    if (versions.length === 0) {
        throw new Error(`No versions found for series ${series}.`);
    }
    versions.sort((a, b) => cmpSemver(a, b));
    return versions[versions.length - 1];
}

/** Resolve artifact URL for version and platform; try preferred extensions. */
async function resolveArtifactURL(
    version: string,
    series: string,
    tag: string,
    exts: string[],
): Promise<string> {
    const base = `${RELEASE_ROOT}/Blender${series}/`;
    for (const ext of exts) {
        const candidate = `${base}blender-${version}-${tag}.${ext}`;
        if (await sh.fetchExists(candidate)) {
            return candidate;
        }
    }
    throw new Error(
        `No artifact found for ${version} (${tag}) with extensions [${exts.join(", ")}].`,
    );
}

/** Expand a DMG file on macOS and copy Blender.app to the output directory. */
async function expandDmg(dmgPath: string, outDir: string): Promise<string> {
    console.log(`Expanding DMG: ${dmgPath}`);

    // Mount the DMG
    await sh.exec("hdiutil", ["attach", dmgPath]);

    try {
        // Copy Blender.app to the output directory
        const blenderAppPath = `${outDir}/Blender.app`;
        const copyResult = await sh.exec("cp", [
            "-R",
            "/Volumes/Blender/Blender.app",
            blenderAppPath,
        ]);

        console.log(`Blender.app copied to: ${blenderAppPath}`);

        // Create a bash wrapper script to run Blender
        const scriptPath = `${outDir}/blender`;
        const scriptContent = `#!/bin/bash
"${blenderAppPath}/Contents/MacOS/Blender" "$@"
`;
        await Deno.writeTextFile(scriptPath, scriptContent, { mode: 0o755 });
        console.log(`Created launcher script: ${scriptPath}`);

        return blenderAppPath;
    } finally {
        // Detach the DMG
        await sh.exec("hdiutil", ["detach", "/Volumes/Blender"]);
    }
}

/** Download file to outDir using streaming. */
async function downloadTo(url: string, outDir: string): Promise<string> {
    await sh.mkdir(outDir);
    const filename = url.split("/").pop()!;
    const outPath = outDir.endsWith("/") ? (outDir + filename) : `${outDir}/${filename}`;

    const r = await fetch(url);
    if (!r.ok || !r.body) {
        throw new Error(`Download failed: HTTP ${r.status} for ${url}`);
    }

    const file = await Deno.open(outPath, {
        write: true,
        create: true,
        truncate: true,
    });

    await r.body.pipeTo(file.writable);

    return outPath;
}

async function main() {
    const outputDir = Deno.args[0];
    if (!outputDir) {
        console.error("Usage: download-blender.ts <output-directory>");
        Deno.exit(1);
    }

    try {
        const { tag, extensions } = detectPlatform();
        const series = await latestSeries();
        const version = await latestVersionInSeries(series);
        const url = await resolveArtifactURL(version, series, tag, extensions);

        sh.cprintln(`Latest Blender series: [${series}](dodgerblue)`);
        sh.cprintln(`Latest Blender version: [${version}](dodgerblue)`);
        sh.cprintln(`Platform: [${tag}](indianred)`);
        sh.cprintln(`URL: [${url}](darkseagreen)`);

        const path = await downloadTo(url, outputDir);
        sh.cprintln(`Downloaded: [${path}](goldenrod)`);

        // Extract the archive (platform-dependent)
        if (Deno.build.os === "darwin" && path.endsWith(".dmg")) {
            const expandedPath = await expandDmg(path, outputDir);
            sh.cprintln(`Expanded DMG to: [${expandedPath}](goldenrod)`);
        } else if (path.endsWith(".tar.xz")) {
            sh.cprintln(`Extracting tar.xz archive...`);
            await sh.exec("tar", ["-xJf", path, "-C", outputDir]);
            const dirName = `${outputDir}/blender-${version}-${tag}`;
            const lines = [
                "#!/usr/bin/env bash",
                `exec "${dirName}/blender" "$@"`,
                "",
            ];
            const launcherPath = `${outputDir}/blender`;
            await Deno.writeTextFile(launcherPath, lines.join("\n"), {
                mode: 0o755,
            });
            sh.cprintln(`Created launcher script: [${launcherPath}](goldenrod)`);
        }

    } catch (err) {
        console.error(
            `Error: ${err instanceof Error ? err.message : String(err)}`,
        );
        Deno.exit(1);
    }
}

if (import.meta.main) {
    await main();
}
