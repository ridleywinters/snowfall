import { ServerOptions, serverStart } from "@/server_start.ts";
import { parseArgs } from "jsr:@std/cli@^1.0.10/parse-args";

function main() {
    const cliArgs = parseArgs(Deno.args, {
        string: ["title", "port", "working-directory"],
        default: {
            title: "raiment-dev-server",
            port: "7000",
            "working-directory": "",
        },
    });

    const options: ServerOptions = {
        title: cliArgs.title,
        port: parseInt(cliArgs.port, 10),
    };

    if (cliArgs["working-directory"]) {
        Deno.chdir(cliArgs["working-directory"]);
    }
    serverStart(options);
}

main();
