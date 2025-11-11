export async function exists(path: string): Promise<boolean> {
    try {
        await Deno.stat(path);
        return true;
    } catch (err) {
        if (err instanceof Deno.errors.NotFound) {
            return false;
        }
        throw err;
    }
}
