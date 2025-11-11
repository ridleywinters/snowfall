import { dirname } from "jsr:@std/path";
import { ensureDir } from "@std/fs";

export async function write(filename: string, data: string): Promise<void> {
    const dir = dirname(filename);
    await ensureDir(dir);
    await Deno.writeTextFile(filename, data);
}
