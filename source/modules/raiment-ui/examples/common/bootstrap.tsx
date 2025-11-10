import React, { JSX } from "react";
import { createRoot } from "react-dom/client";
import { useServerSideEvents } from "@raiment-ui";

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

export function bootstrap(Component: () => JSX.Element): void {
    const el = document.getElementById("root")!;
    createRoot(el).render(
        <AppWrapper>
            <Component />
        </AppWrapper>,
    );
}
