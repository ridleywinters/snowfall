import { cprintln } from "@raiment-shell";
import { handleServerSideEvents, SSEClientSet } from "./handle_server_side_events.ts";
import { handleStaticFiles } from "./handle_static_files.ts";
import { handleStatus } from "./handle_status.ts";
import { runFilePollingLoop } from "./run_file_polling_loop.ts";

export type ServerOptions = {
    title: string;
    port: number;
};

type HandlerPattern =
    | string
    | RegExp;

async function checkPortAvailable(port: number): Promise<boolean> {
    try {
        const listener = Deno.listen({ port });
        listener.close();
        return true;
    } catch {
        return false;
    }
}

export async function serverStart(options: ServerOptions) {
    const isPortAvailable = await checkPortAvailable(options.port);
    if (!isPortAvailable) {
        cprintln(`[error](error): port [${options.port}](number) is already in use.`);
        Deno.exit(1);
    }

    const clients = new SSEClientSet();

    runFilePollingLoop("./dist/build.timestamp", ({ filename, current, previous }) => {
        cprintln(
            "#555",
            [
                `File update [${filename}](filename)`,
                `Broadcasting [app.reload](string)`,
            ].join("\n"),
        );
        clients.broadcast({ type: "app.reload" });
    });

    const handlers: [HandlerPattern, (request: Request) => Promise<Response>][] = [
        ["/status", handleStatus],
        ["/api/events", (req) => handleServerSideEvents(clients)],
        [/.*/, (req) => handleStaticFiles("./dist", req)],
    ];

    await Deno.serve({
        port: options.port,
        onListen: () => {
            cprintln();
            cprintln(`${options.title} is running on [http://localhost:${options.port}](url)`);
            cprintln("#555", `Platform: ${Deno.build.os}-${Deno.build.arch}`);
            cprintln("#555", `Working directory: ${relativeCWD()}`);
            cprintln("#555", "Press [Ctrl+C](#89C) to stop the server.");

            cprintln(
                "#555",
                [
                    "",
                    "[WARNING](warn): this is a development server. It is not intended to be used in",
                    "production environments as it has not been security tested or optimized",
                    "for performance.",
                    "",
                ].join("\n"),
            );
        },
        handler: (req: Request): Promise<Response> => {
            const url = new URL(req.url);
            const match = handlers.find(([pattern]) => {
                return typeof pattern === "string"
                    ? pattern === url.pathname
                    : pattern.test(url.pathname);
            });
            if (!match) {
                return Promise.resolve(new Response("Not Found", { status: 404 }));
            }
            const [, handler] = match;
            return handler(req);
        },
    });
}

function relativeCWD(): string {
    let cwd = Deno.cwd();
    const home = Deno.env.get("HOME") ?? "";
    if (cwd.startsWith(`${home}/`)) {
        cwd = `~/${cwd.slice(home.length)}`;
    }
    return cwd;
}
