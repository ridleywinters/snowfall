export async function read(filename: string): Promise<string> {
    const data = await Deno.readTextFile(filename);
    return data;
}
