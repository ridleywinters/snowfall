import { normalize } from "@std/path";

export async function copy(source: string, destination: string): Promise<void> {
    source = normalize(source);
    destination = normalize(destination);

    // Provide a clear error if the source file does not exist
    try {
        await Deno.stat(source);
    } catch (error) {
        if (error instanceof Deno.errors.NotFound) {
            throw new Error(`Source file does not exist: ${source}`);
        }
        throw error;
    }

    return await Deno.copyFile(source, destination);
}
