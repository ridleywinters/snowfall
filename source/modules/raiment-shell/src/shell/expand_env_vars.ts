export function expandEnvVars(template: string): string {
    return template.replace(
        /\$([A-Za-z_][A-ZaZ0-9_]*)/g,
        (_, varName) => {
            const value = Deno.env.get(varName);
            if (!value) {
                throw new Error(`Environment variable ${varName} is not set`);
            }
            return value;
        },
    );
}
