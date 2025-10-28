export async function exec(command: string, args: string[]): Promise<void> {
    const cmd = new Deno.Command(command, {
        args,
        stdout: "inherit",
        stderr: "inherit",
    });

    const { code } = await cmd.output();
    if (code !== 0) {
        throw new Error(`Command failed: ${command} ${args.join(" ")}`);
    }
}
