import React, { JSX } from "react";
import { createRoot } from "react-dom/client";
import { useServerSideEvents } from "@raiment-ui";
import { TableExample } from "./table_example.tsx";

function AppWrapper({ children }: { children: React.ReactNode }): JSX.Element {
    useServerSideEvents("/api/events", (data: any) => {
        switch (data.type) {
            case "app.reload":
                globalThis.location.reload();
                break;
            default:
                console.warn("Unknown server-side event type:", data.type);
                break;
        }
    });
    return <>{children}</>;
}

function main() {
    const el = document.getElementById("root")!;
    createRoot(el).render(
        <AppWrapper>
            <TableExample />
        </AppWrapper>,
    );
}
main();
