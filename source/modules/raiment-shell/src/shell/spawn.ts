export async function spawn(command: string, args: string[]): Promise<string> {
    const cmd = new Deno.Command(command, {
        args,
        stdout: "piped",
        stderr: "inherit",
    });

    const { code, stdout } = await cmd.output();
    if (code !== 0) {
        throw new Error(`Command failed: ${command} ${args.join(" ")}`);
    }
    return new TextDecoder().decode(stdout);
}
