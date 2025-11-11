import { expandEnvVars } from "./expand_env_vars.ts";

const RE =
    /\{\{([A-Za-z_][A-Za-z0-9_]*(?:\.(?:[A-Za-z_][A-Za-z0-9_]*|\d+))*)\}\}/g;

export function template(
    tmpl: string,
    vars: Record<string, unknown> = {},
): string {
    return expandEnvVars(tmpl) //
        .replace(RE, (_, varPath) => {
            const parts = varPath.split(".");
            const value = resolvePath(vars, parts);
            if (value === undefined || value === null) {
                return "";
            }
            return String(value);
        });
}

function resolvePath(vars: Record<string, unknown>, parts: string[]): unknown {
    let current: unknown = vars;
    for (const p of parts) {
        if (!current || typeof current !== "object") {
            return undefined;
        }

        // support array index access when current is an array and the segment is numeric
        if (Array.isArray(current) && /^\d+$/.test(p)) {
            const index = Number(p);
            if (index < 0 || index >= current.length) {
                return undefined;
            }
            current = current[index];
            continue;
        }

        // ordinary object property
        const obj = current as Record<string, unknown>;
        if (!(p in obj)) {
            return undefined;
        }
        current = obj[p];
    }
    return current;
}
