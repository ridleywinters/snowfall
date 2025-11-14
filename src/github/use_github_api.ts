import React from "react";
import { GitHubAPI } from "./github_api.ts";
import { useGitHubAuthToken } from "./use_github_auth_token.ts";

export function useGitHubAPI(): GitHubAPI {
    const authState = useGitHubAuthToken();
    const api = React.useMemo(() => {
        const api = new GitHubAPI(authState.accessToken, authState.authState);
        return api;
    }, [authState, authState.accessToken, authState.authState]);
    return api;
}
