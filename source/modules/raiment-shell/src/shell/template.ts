import { expandEnvVars } from "./expand_env_vars.ts";

// TODO: this function needs tests
export function template(template: string, vars: Record<string, string> = {}): string {
    return expandEnvVars(template).replace(
        /\{\{([A-Za-z_][A-ZaZ0-9_]*)\}\}/g,
        (_, varName) => {
            if (vars[varName] !== undefined) {
                return vars[varName];
            }
            return "";
        },
    );
}
