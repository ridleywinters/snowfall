import React from "react";
import { AuthState, CLIENT_ID, DEBUG_AUTH, GUIDEBOOK_AUTH_SERVER } from "./internal.ts";

const dbg = DEBUG_AUTH ? (...args: any[]) => console.log("DEBUG:", ...args) : () => {};

export type GitHubAuthState = {
    accessToken : string | null;
    authState : AuthState;
};

/**
 * Returns the current GitHub auth token is the user is authenticated.
 * If not, it will be null.
 * 
 * See GitHubAPI.signIn() for initiating the sign-in process.
 * See GitHubAPI.signOut() for signing out.
 */
export function useGitHubAuthToken(): GitHubAuthState {
    const current = localStorage.getItem("github_auth/access_token") ?? null;
    const [authState, setAuthState] = React.useState<GitHubAuthState>({
        accessToken : current,
        authState : current ? "authenticated" : "unauthenticated",
});
    dbg("current access token:", authState.accessToken);

    React.useEffect(() => {
        const url = new URL(globalThis.location.href);

        // --- If there's already an access token... ---
        //
        // We're already authenticated, there's not much to do. The only
        // exception is if we just did the oauth handshake, we want to clean
        // the URL of the auth parameters.
        //
        if (authState.accessToken) {
            scrubURL();
            return;
        }

        // --- Is this a callback from GitHub after sign-in? ---
        //
        // If the oauth handshake was initiated and the browser has been
        // directed back here from GitHub, we want to complete the handshake.
        //
        //
        // Redirect to the guidebook auth server to get an access token from
        // the "code" provided by GitHub. We need the indirect auth server to
        // avoid CORS issues
        // (GitHub won't accept the callback from the browser).
        //

        const auth = url.searchParams.get("auth");
        const code = url.searchParams.get("code");
        dbg("auth callback params:", { auth, code });

        if (auth === "github" && code?.length) {
            dbg("Resolving GitHub auth callback", auth, code, CLIENT_ID);
            setAuthState({
                accessToken: null,
                authState: "pending",
            });

            const go = async () => {
                const resp = await fetch(GUIDEBOOK_AUTH_SERVER, {
                    method: "POST",
                    headers: {
                        Accept: "application/json",
                        "Content-Type": "application/json",
                        "cache-control": "no-cache",
                    },
                    body: JSON.stringify({
                        url: `https://github.com/login/oauth/access_token`,
                        method: "POST",
                        headers: {
                            Accept: "application/json",
                            "Content-Type": "application/json",
                        },
                        body: {
                            client_id: CLIENT_ID,
                            code,
                        },
                    }),
                });

                const json = await resp.json();
                if (!json.access_token) {
                    console.error("Failed to get access token", json);
                    if (json.error_uri) {
                        console.log("See:", json.error_uri);
                    }
                    return;
                }

                dbg("Received access token", json);
                localStorage.setItem("github_auth/access_token", json.access_token);
                scrubURL();
                setAuthState({
                    accessToken: json.access_token,
                    authState: "authenticated",
                });
            };
            go();
        }

        // Otherwise, we don't have an access token or a code, so there's
        // nothing for this hook to do!
    }, []);

    return authState;
}

function scrubURL(): void {
    // Clear any auth callback parameters from the URL if necessary and
    // reload.
    const scrubbed = new URL(globalThis.location.href);
    scrubbed.searchParams.delete("auth");
    scrubbed.searchParams.delete("code");
    scrubbed.searchParams.delete("state");

    const current = globalThis.location.href;
    const next = scrubbed.href
    if (current === next) {
        return;
    }
    globalThis.history.replaceState({}, "", next);
}
