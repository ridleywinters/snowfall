export const GUIDEBOOK_AUTH_SERVER = "https://guidebook-auth-server.deno.dev/";
export const CLIENT_ID = "Ov23li89ZvKkoY3YqFDj";
export const DEBUG_AUTH = false;

export type AuthState = "authenticated" | "unauthenticated" | "pending";

export function navigateToSignIn(): void {
    const paramsHash = {
        client_id: CLIENT_ID,
        scope: "read:user, repo, gist",
        state: encodeURIComponent(globalThis.location.href),
        allow_signup: "false",
        prompt: "select_account",
    };
    const params = new URLSearchParams(paramsHash);
    const url = `https://github.com/login/oauth/authorize?${params}`;
    globalThis.location.assign(url);
}

export function navigateToSignOut(): void {
    localStorage.removeItem("github_auth/access_token");
    globalThis.location.reload();
}
