#!/usr/bin/env -S deno run --allow-read --allow-write --allow-env --allow-ffi

/**
 * PNG texture extraction and resizing tool with metadata handling
 *
 * Usage: extract_texture.ts <input_png> <meta_md> <new_size> <output_png>
 *
 * Arguments:
 *   input_png  - Path to the input PNG file
 *   meta_md    - Path to the markdown metadata file
 *   new_size   - New size for the PNG in WIDTHxHEIGHT format (e.g., "64x64", "32x32")
 *   output_png - Path to the output PNG file
 *
 * Example:
 *   ./extract_texture.ts input.png input.meta.md 64x64 output.png
 *
 */

import sharp from "sharp";
import { dirname, basename, join } from "@std/path";
import { ensureDir } from "@std/fs";

/**
 * Parse and validate command line arguments
 */
function parseArgs(): {
  inputPng: string;
  metaMd: string;
  width: number;
  height: number;
  outputPng: string;
} {
  if (Deno.args.length !== 4) {
    console.error("Error: Expected 4 arguments");
    console.error(
      "Usage: extract_texture.ts <input_png> <meta_md> <new_size> <output_png>"
    );
    console.error("");
    console.error("Arguments:");
    console.error("  input_png  - Path to the input PNG file");
    console.error("  meta_md    - Path to the markdown metadata file");
    console.error(
      "  new_size   - New size for the PNG in WIDTHxHEIGHT format (e.g., '64x64', '32x32')"
    );
    console.error("  output_png - Path to the output PNG file");
    Deno.exit(1);
  }

  const [inputPng, metaMd, sizeStr, outputPng] = Deno.args;

  // Parse and validate size format (WIDTHxHEIGHT)
  const sizeMatch = sizeStr.match(/^(\d+)x(\d+)$/i);
  if (!sizeMatch) {
    console.error(
      `Error: Invalid size format "${sizeStr}". Expected format: WIDTHxHEIGHT (e.g., "64x64", "32x32")`
    );
    Deno.exit(1);
  }

  const width = parseInt(sizeMatch[1], 10);
  const height = parseInt(sizeMatch[2], 10);

  if (width <= 0 || height <= 0) {
    console.error(
      `Error: Invalid dimensions "${sizeStr}". Width and height must be positive numbers.`
    );
    Deno.exit(1);
  }

  return { inputPng, metaMd, width, height, outputPng };
}

/**
 * Main execution function
 */
async function main() {
  const { inputPng, metaMd, width, height, outputPng } = parseArgs();

  try {
    // Ensure output directory exists
    const outputDir = dirname(outputPng);
    await ensureDir(outputDir);

    // Load and resize the PNG to exact dimensions
    await sharp(inputPng)
      .resize(width, height, {
        fit: "fill",
      })
      .toFile(outputPng);

    // Copy metadata file to output directory with matching basename
    const outputBasename = basename(outputPng, ".png");
    const outputMetaPath = join(outputDir, `${outputBasename}.meta.md`);

    const metaContent = await Deno.readFile(metaMd);
    await Deno.writeFile(outputMetaPath, metaContent);

    console.log(`✓ Created ${outputPng} (${basename(outputMetaPath)})`);
  } catch (error) {
    if (error instanceof Error) {
      console.error(`Error: ${error.message}`);
    } else {
      console.error(`Error: ${error}`);
    }
    Deno.exit(1);
  }
}

// Run main function if this is the main module
if (import.meta.main) {
  await main();
}
