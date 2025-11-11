export async function which(cmd: string): Promise<string | null> {
    try {
        const c = new Deno.Command("which", {
            args: [cmd],
            stdout: "piped",
            stderr: "null",
        });
        const { code, stdout } = await c.output();
        if (code !== 0) {
            return null;
        }
        const out = new TextDecoder().decode(stdout).trim();
        if (!out) {
            return null;
        }
        // Return first line (path)
        return out.split("\n")[0];
    } catch {
        return null;
    }
}
