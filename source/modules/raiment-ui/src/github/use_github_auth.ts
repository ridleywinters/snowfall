import { navigateToSignIn, navigateToSignOut } from "./internal.ts";
import { useGitHubAuthToken } from "./use_github_auth_token.ts";

type UseGitHubAuthResult = {
    accessToken: string | null;
    authState: "authenticated" | "unauthenticated" | "pending";
    gitHubSignIn: () => void;
    gitHubSignOut: () => void;
};

export function useGitHubAuth(): UseGitHubAuthResult {
    const authState = useGitHubAuthToken();
    return {
        accessToken: authState.accessToken,
        authState: authState.authState,
        gitHubSignIn: navigateToSignIn,
        gitHubSignOut: navigateToSignOut,
    };
}
