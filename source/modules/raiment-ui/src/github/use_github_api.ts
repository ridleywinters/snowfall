import React from "react";
import { GitHubAPI } from "./github_api.ts";
import { useGitHubAuthToken } from "./use_github_auth_token.ts";

export function useGitHubAPI(): GitHubAPI {
    const state = useGitHubAuthToken();
    const api = React.useMemo(() => {
        const api = new GitHubAPI(state.accessToken, state.authState);
        return api;
    }, [state, state.accessToken, state.authState]);
    return api;
}
