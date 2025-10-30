import { AppView } from "@/views/app_view.tsx";
import { createRoot } from "react-dom/client";

function main(): void {
    const el = document.getElementById("root")!;
    createRoot(el).render(<AppView />);
}
main();
