#!/usr/bin/env -S deno run --allow-read --allow-write

import { parseArgs } from "@std/cli/parse-args";
import { basename, dirname, join } from "@std/path";
import { ensureDir } from "@std/fs";
import { crc32 } from "@raiment-core";

/**
 * Aseprite file format parser and PNG converter
 * Based on Aseprite File Format specification (.ase/.aseprite)
 */

interface AsepriteHeader {
    fileSize: number;
    magicNumber: number;
    frames: number;
    width: number;
    height: number;
    colorDepth: number;
    flags: number;
    speed: number;
    transparentIndex: number;
    numColors: number;
    pixelWidth: number;
    pixelHeight: number;
    gridX: number;
    gridY: number;
    gridWidth: number;
    gridHeight: number;
}

interface Frame {
    bytesInFrame: number;
    magicNumber: number;
    oldChunks: number;
    frameDuration: number;
    newChunks: number;
    chunks: Chunk[];
}

interface Chunk {
    size: number;
    type: number;
    data: Uint8Array;
}

interface Layer {
    flags: number;
    type: number;
    childLevel: number;
    width: number;
    height: number;
    blendMode: number;
    opacity: number;
    name: string;
    visible: boolean;
}

interface Cel {
    layerIndex: number;
    x: number;
    y: number;
    opacity: number;
    celType: number;
    width: number;
    height: number;
    pixels: Uint8Array;
}

const CHUNK_TYPES = {
    OLD_PALETTE_1: 0x0004,
    OLD_PALETTE_2: 0x0011,
    LAYER: 0x2004,
    CEL: 0x2005,
    CEL_EXTRA: 0x2006,
    COLOR_PROFILE: 0x2007,
    MASK: 0x2016,
    PATH: 0x2017,
    TAGS: 0x2018,
    PALETTE: 0x2019,
    USER_DATA: 0x2020,
    SLICE: 0x2022,
    TILESET: 0x2023,
};

class AsepriteReader {
    private data: Uint8Array;
    private offset: number = 0;

    constructor(data: Uint8Array) {
        this.data = data;
    }

    readByte(): number {
        return this.data[this.offset++];
    }

    readWord(): number {
        const value = this.data[this.offset] | (this.data[this.offset + 1] << 8);
        this.offset += 2;
        return value;
    }

    readShort(): number {
        const value = this.readWord();
        return value > 0x7fff ? value - 0x10000 : value;
    }

    readDword(): number {
        const value = this.data[this.offset] |
            (this.data[this.offset + 1] << 8) |
            (this.data[this.offset + 2] << 16) |
            (this.data[this.offset + 3] << 24);
        this.offset += 4;
        return value >>> 0;
    }

    readLong(): number {
        const value = this.readDword();
        return value > 0x7fffffff ? value - 0x100000000 : value;
    }

    readFixed(): number {
        return this.readDword() / 65536;
    }

    readBytes(count: number): Uint8Array {
        const bytes = this.data.slice(this.offset, this.offset + count);
        this.offset += count;
        return bytes;
    }

    readString(): string {
        const length = this.readWord();
        const bytes = this.readBytes(length);
        return new TextDecoder().decode(bytes);
    }

    skip(count: number): void {
        this.offset += count;
    }

    getOffset(): number {
        return this.offset;
    }

    setOffset(offset: number): void {
        this.offset = offset;
    }
}

function parseHeader(reader: AsepriteReader): AsepriteHeader {
    const fileSize = reader.readDword();
    const magicNumber = reader.readWord();

    if (magicNumber !== 0xa5e0) {
        throw new Error("Invalid Aseprite file: wrong magic number");
    }

    const frames = reader.readWord();
    const width = reader.readWord();
    const height = reader.readWord();
    const colorDepth = reader.readWord();
    const flags = reader.readDword();
    const speed = reader.readWord();

    reader.skip(8); // Skip next 8 bytes

    const transparentIndex = reader.readByte();
    reader.skip(3); // Skip 3 bytes
    const numColors = reader.readWord();
    const pixelWidth = reader.readByte();
    const pixelHeight = reader.readByte();
    const gridX = reader.readShort();
    const gridY = reader.readShort();
    const gridWidth = reader.readWord();
    const gridHeight = reader.readWord();

    reader.skip(84); // Skip reserved bytes

    return {
        fileSize,
        magicNumber,
        frames,
        width,
        height,
        colorDepth,
        flags,
        speed,
        transparentIndex,
        numColors,
        pixelWidth,
        pixelHeight,
        gridX,
        gridY,
        gridWidth,
        gridHeight,
    };
}

function parseFrame(reader: AsepriteReader): Frame {
    const bytesInFrame = reader.readDword();
    const magicNumber = reader.readWord();

    if (magicNumber !== 0xf1fa) {
        throw new Error("Invalid frame: wrong magic number");
    }

    const oldChunks = reader.readWord();
    const frameDuration = reader.readWord();
    reader.skip(2); // Reserved
    const newChunks = reader.readDword();

    const chunkCount = newChunks === 0 ? oldChunks : newChunks;
    const chunks: Chunk[] = [];

    for (let i = 0; i < chunkCount; i++) {
        const size = reader.readDword();
        const type = reader.readWord();
        const dataSize = size - 6; // Size includes size and type fields
        const data = reader.readBytes(dataSize);

        chunks.push({ size, type, data });
    }

    return {
        bytesInFrame,
        magicNumber,
        oldChunks,
        frameDuration,
        newChunks,
        chunks,
    };
}

function parseLayerChunk(data: Uint8Array): Layer {
    const reader = new AsepriteReader(data);
    const flags = reader.readWord();
    const type = reader.readWord();
    const childLevel = reader.readWord();
    const width = reader.readWord();
    const height = reader.readWord();
    const blendMode = reader.readWord();
    const opacity = reader.readByte();
    reader.skip(3); // Reserved
    const name = reader.readString();

    return {
        flags,
        type,
        childLevel,
        width,
        height,
        blendMode,
        opacity,
        name,
        visible: (flags & 1) !== 0,
    };
}

async function parseCelChunk(
    data: Uint8Array,
    width: number,
    height: number,
): Promise<Cel> {
    const reader = new AsepriteReader(data);
    const layerIndex = reader.readWord();
    const x = reader.readShort();
    const y = reader.readShort();
    const opacity = reader.readByte();
    const celType = reader.readWord();
    reader.skip(7); // Reserved

    let celWidth = width;
    let celHeight = height;
    let pixels = new Uint8Array(0);

    if (celType === 0) {
        // Raw cel
        celWidth = reader.readWord();
        celHeight = reader.readWord();
        const expectedSize = celWidth * celHeight * 4;
        const remainingData = data.length - reader.getOffset();

        if (remainingData < expectedSize) {
            console.warn(
                `WARNING: Not enough data for raw cel! Expected ${expectedSize}, got ${remainingData}`,
            );
        }

        pixels = new Uint8Array(reader.readBytes(Math.min(expectedSize, remainingData)));
    } else if (celType === 2) {
        // Compressed cel
        celWidth = reader.readWord();
        celHeight = reader.readWord();
        const compressedSize = data.length - reader.getOffset();

        if (compressedSize <= 0) {
            console.warn(`WARNING: No compressed data available!`);
            pixels = new Uint8Array(celWidth * celHeight * 4);
        } else {
            const compressedData = reader.readBytes(compressedSize);

            try {
                const inflated = await inflate(compressedData);
                pixels = new Uint8Array(inflated);
            } catch (error) {
                console.error(`ERROR during decompression:`, error);
                pixels = new Uint8Array(celWidth * celHeight * 4);
            }
        }
    } else if (celType === 1) {
        // Linked cel
        const _framePosition = reader.readWord();
        console.warn(`WARNING: Linked cels (type 1) not fully supported yet`);
    } else {
        console.warn(`WARNING: Unknown cel type: ${celType}`);
    }

    return {
        layerIndex,
        x,
        y,
        opacity,
        celType,
        width: celWidth,
        height: celHeight,
        pixels,
    };
}

async function inflate(data: Uint8Array): Promise<Uint8Array> {
    // Aseprite uses zlib compression (deflate with zlib wrapper)
    // DecompressionStream "deflate-raw" expects raw deflate
    let deflateData = data;

    // Check for zlib header (0x78 0x9C or 0x78 0x01 or 0x78 0xDA)
    if (data.length > 2 && data[0] === 0x78) {
        // Strip zlib header (2 bytes) and checksum (last 4 bytes)
        deflateData = data.slice(2, data.length - 4);
    }

    // Use DecompressionStream for deflate
    const stream = new DecompressionStream("deflate-raw");
    const writer = stream.writable.getWriter();

    // Write the data and close the writer
    const writePromise = (async () => {
        await writer.write(deflateData as unknown as BufferSource);
        await writer.close();
    })();

    // Read the decompressed data
    const reader = stream.readable.getReader();
    const chunks: Uint8Array[] = [];

    const readPromise = (async () => {
        while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            chunks.push(value);
        }
    })();

    // Wait for both write and read to complete
    await Promise.all([writePromise, readPromise]);

    const totalLength = chunks.reduce((acc, chunk) => acc + chunk.length, 0);
    const result = new Uint8Array(totalLength);
    let offset = 0;
    for (const chunk of chunks) {
        result.set(chunk, offset);
        offset += chunk.length;
    }

    return result;
}

function _parsePalette(data: Uint8Array): number[][] {
    const reader = new AsepriteReader(data);
    const _newPaletteSize = reader.readDword();
    const firstColor = reader.readDword();
    const lastColor = reader.readDword();
    reader.skip(8); // Reserved

    const palette: number[][] = [];

    for (let i = firstColor; i <= lastColor; i++) {
        const flags = reader.readWord();
        const r = reader.readByte();
        const g = reader.readByte();
        const b = reader.readByte();
        const a = reader.readByte();

        palette[i] = [r, g, b, a];

        if (flags & 1) {
            // Has name
            reader.readString();
        }
    }

    return palette;
}

function createRGBAImage(
    width: number,
    height: number,
    layers: Layer[],
    cels: Cel[],
    colorDepth: number,
): Uint8Array {
    const image = new Uint8Array(width * height * 4);

    // Initialize with transparent pixels
    for (let i = 0; i < image.length; i += 4) {
        image[i] = 0; // R
        image[i + 1] = 0; // G
        image[i + 2] = 0; // B
        image[i + 3] = 0; // A (transparent)
    }

    // Draw cels in order (bottom to top)
    for (const cel of cels) {
        const layer = layers[cel.layerIndex];
        if (!layer || !layer.visible) continue;

        const layerOpacity = layer.opacity / 255;
        const celOpacity = cel.opacity / 255;
        const totalOpacity = layerOpacity * celOpacity;

        if (colorDepth === 32) {
            // RGBA mode
            for (let py = 0; py < cel.height; py++) {
                for (let px = 0; px < cel.width; px++) {
                    const sx = cel.x + px;
                    const sy = cel.y + py;

                    if (sx < 0 || sx >= width || sy < 0 || sy >= height) continue;

                    const srcIdx = (py * cel.width + px) * 4;
                    const dstIdx = (sy * width + sx) * 4;

                    const srcR = cel.pixels[srcIdx];
                    const srcG = cel.pixels[srcIdx + 1];
                    const srcB = cel.pixels[srcIdx + 2];
                    const srcA = (cel.pixels[srcIdx + 3] / 255) * totalOpacity;

                    const dstR = image[dstIdx];
                    const dstG = image[dstIdx + 1];
                    const dstB = image[dstIdx + 2];
                    const dstA = image[dstIdx + 3] / 255;

                    // Alpha blending
                    const outA = srcA + dstA * (1 - srcA);
                    if (outA > 0) {
                        image[dstIdx] = Math.round(
                            (srcR * srcA + dstR * dstA * (1 - srcA)) / outA,
                        );
                        image[dstIdx + 1] = Math.round(
                            (srcG * srcA + dstG * dstA * (1 - srcA)) / outA,
                        );
                        image[dstIdx + 2] = Math.round(
                            (srcB * srcA + dstB * dstA * (1 - srcA)) / outA,
                        );
                        image[dstIdx + 3] = Math.round(outA * 255);
                    }
                }
            }
        }
    }

    return image;
}

async function encodePNG(
    width: number,
    height: number,
    data: Uint8Array,
): Promise<Uint8Array> {
    // Simple PNG encoder
    const png: number[] = [];

    // PNG signature
    png.push(137, 80, 78, 71, 13, 10, 26, 10);

    // IHDR chunk
    const ihdr: number[] = [];
    writeUint32(ihdr, width);
    writeUint32(ihdr, height);
    ihdr.push(8); // Bit depth
    ihdr.push(6); // Color type (RGBA)
    ihdr.push(0); // Compression
    ihdr.push(0); // Filter
    ihdr.push(0); // Interlace
    writeChunk(png, "IHDR", new Uint8Array(ihdr));

    // IDAT chunk
    const imageData: number[] = [];
    for (let y = 0; y < height; y++) {
        imageData.push(0); // Filter type (none)
        for (let x = 0; x < width; x++) {
            const idx = (y * width + x) * 4;
            imageData.push(data[idx], data[idx + 1], data[idx + 2], data[idx + 3]);
        }
    }

    const compressed = await deflateSync(new Uint8Array(imageData));
    writeChunk(png, "IDAT", compressed);

    // IEND chunk
    writeChunk(png, "IEND", new Uint8Array(0));

    return new Uint8Array(png);
}

function writeUint32(arr: number[], value: number): void {
    arr.push((value >>> 24) & 0xff);
    arr.push((value >>> 16) & 0xff);
    arr.push((value >>> 8) & 0xff);
    arr.push(value & 0xff);
}

function writeChunk(png: number[], type: string, data: Uint8Array): void {
    const typeBytes = new TextEncoder().encode(type);

    writeUint32(png, data.length);

    const crcData: number[] = [];
    crcData.push(...typeBytes, ...data);

    png.push(...typeBytes);
    png.push(...data);

    const crc = crc32(new Uint8Array(crcData));
    writeUint32(png, crc);
}

async function deflateSync(data: Uint8Array): Promise<Uint8Array> {
    // Use CompressionStream for deflate
    const stream = new CompressionStream("deflate");
    const writer = stream.writable.getWriter();

    // Write and read in parallel to prevent deadlock
    const writePromise = (async () => {
        await writer.write(data as unknown as BufferSource);
        await writer.close();
    })();

    const reader = stream.readable.getReader();
    const chunks: Uint8Array[] = [];

    const readPromise = (async () => {
        while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            chunks.push(value);
        }
    })();

    // Wait for both operations to complete
    await Promise.all([writePromise, readPromise]);

    const totalLength = chunks.reduce((acc, chunk) => acc + chunk.length, 0);

    const result = new Uint8Array(totalLength);
    let offset = 0;
    for (const chunk of chunks) {
        result.set(chunk, offset);
        offset += chunk.length;
    }
    return result;
}

async function convertAsepriteToPNG(
    inputPath: string,
    outputPath: string,
    frameIndex: number = 0,
): Promise<void> {
    const fileData = await Deno.readFile(inputPath);
    const reader = new AsepriteReader(fileData);

    const header = parseHeader(reader);

    if (header.colorDepth !== 32) {
        throw new Error(
            `Unsupported color depth: ${header.colorDepth}. Only RGBA (32-bit) is currently supported.`,
        );
    }

    const layers: Layer[] = [];
    const cels: Cel[] = [];

    for (let i = 0; i < header.frames; i++) {
        const frame = parseFrame(reader);

        if (i === frameIndex) {
            for (const chunk of frame.chunks) {
                if (chunk.type === CHUNK_TYPES.LAYER) {
                    const layer = parseLayerChunk(chunk.data);
                    layers.push(layer);
                } else if (chunk.type === CHUNK_TYPES.CEL) {
                    const cel = await parseCelChunk(
                        chunk.data,
                        header.width,
                        header.height,
                    );
                    cels.push(cel);
                } else if (chunk.type === CHUNK_TYPES.PALETTE) {
                    // Palette chunk - not currently used
                }
            }

            break; // Only process the requested frame
        }
    }

    const rgbaData = createRGBAImage(
        header.width,
        header.height,
        layers,
        cels,
        header.colorDepth,
    );

    const pngData = await encodePNG(header.width, header.height, rgbaData);

    await ensureDir(dirname(outputPath));
    await Deno.writeFile(outputPath, pngData);
}

// CLI
if (import.meta.main) {
    const args = parseArgs(Deno.args, {
        string: ["input", "output", "frame"],
        alias: {
            i: "input",
            o: "output",
            f: "frame",
            h: "help",
        },
        boolean: ["help"],
    });

    if (args.help || !args.input) {
        console.log(`
Aseprite to PNG Converter

Usage:
  deno run --allow-read --allow-write aseprite_to_png.ts -i <input.aseprite> [-o <output.png>] [-f <frame>]

Options:
  -i, --input <file>    Input Aseprite file (required)
  -o, --output <file>   Output PNG file (default: input filename with .png extension)
  -f, --frame <num>     Frame number to export (default: 0)
  -h, --help            Show this help message

Examples:
  deno run --allow-read --allow-write aseprite_to_png.ts -i sprite.aseprite
  deno run --allow-read --allow-write aseprite_to_png.ts -i sprite.ase -o output.png -f 1
    `);
        Deno.exit(args.help ? 0 : 1);
    }

    const inputPath = args.input as string;
    const outputPath = (args.output as string) ||
        join(dirname(inputPath), basename(inputPath, ".aseprite") + ".png").replace(
            /\.ase$/,
            ".png",
        );
    const frameIndex = args.frame ? parseInt(args.frame as string, 10) : 0;

    try {
        await convertAsepriteToPNG(inputPath, outputPath, frameIndex);
    } catch (error) {
        console.error(
            "Error:",
            error instanceof Error ? error.message : String(error),
        );
        Deno.exit(1);
    }
}
