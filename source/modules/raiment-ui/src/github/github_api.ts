import React from "react";
import { navigateToSignIn, navigateToSignOut, AuthState } from "./internal.ts";

export class GitHubAPI {
    _token: string | null;
    _authState: AuthState;
    _cache: { [key: string]: any } = {};

    constructor(token: string | null, authState: AuthState) {
        this._token = token;
        this._authState = authState;
    }

    //=========================================================================
    // Sign-in / Sign-out
    //=========================================================================

    get token(): string | null {
        return this._token;
    }

    get authState(): AuthState {
        return this._authState;
    }

    signIn(): void {
        navigateToSignIn();
    }

    signOut(): void {
        navigateToSignOut();
    }

    //=========================================================================
    // State hooks
    //=========================================================================

    useUser() {
        const [user, setUser] = React.useState<any>(null);
        React.useEffect(() => {
            const go = async () => {
                const user = await this.user();
                setUser(user);
            };
            go();
        }, []);
        return user;
    }

    //=========================================================================
    // API wrappers
    //=========================================================================

    async user() {
        return this.cacheFetch("user", `https://api.github.com/user`);
    }

    async repositoryExists(repositoryName: string): Promise<boolean> {
        const user = await this.user();
        const username = user.login;
        const url = `https://api.github.com/repos/${username}/${repositoryName}`;
        try {
            const resp = await this._fetchRaw("GET", url);
            return resp.status === 200 ? true : false;
        } catch (_err) {
            return false;
        }
    }

    async createRepository(repositoryName: string): Promise<void> {
        const url = `https://api.github.com/user/repos`;
        const params = {
            name: repositoryName,
            description: "Guidebook data repository",
            private: false,
            has_issues: false,
            has_projects: false,
            has_wiki: false,
            has_downloads: false,
            auto_init: true,
        };
        await this.fetch("POST", url, params);
    }

    async readFileContents(repo: string, filename: string): Promise<string | null> {
        const user = await this.user();
        const username = user.login;
        const url = `https://api.github.com/repos/${username}/${repo}/contents/${filename}`;

        const existing = await this.fetch("GET", url);
        if (!existing) {
            return null;
        }
        const encoded = existing.content;
        const content = atob(encoded);
        return content;
    }

    _updateTimers: Record<string, number | undefined> = {};

    async updateFileContents(
        repo: string,
        filename: string,
        content: string,
        delay = 200,
    ): Promise<void> {
        globalThis.clearTimeout(this._updateTimers[filename]);

        this._updateTimers[filename] = globalThis.setTimeout(async () => {
            const user = await this.user();
            const username = user.login;
            const url = `https://api.github.com/repos/${username}/${repo}/contents/${filename}`;

            let sha;
            {
                const existing = await this.fetch("GET", url);
                if (existing) {
                    sha = existing.sha;
                }
            }

            // Base64 encode the content
            const encoded = btoa(content);
            this.fetch("PUT", url, {
                message: "update via guidebook API",
                committer: {
                    name: "guidebook-app",
                    email: "support@raiment-studios.com",
                },
                content: encoded,
                sha: sha,
            });

            this._updateTimers[filename] = undefined;
        }, delay);
    }

    //=========================================================================
    // Direct API calls
    //=========================================================================

    async fetch(method: string, url: string, body: any = null): Promise<any> {
        const resp = await this._fetchRaw(method, url, body);
        if (!resp.ok) {
            return null;
        }
        const json = await resp.json();
        return json;
    }

    cacheClear(key: string): void {
        delete this._cache[key];
    }

    async cacheFetch(cacheKey: string, url: string): Promise<any> {
        const cached = this._cache[cacheKey];
        if (cached) {
            return cached;
        }
        const json = await this.fetch("GET", url);
        this._cache[cacheKey] = json;
        return json;
    }

    //=========================================================================
    // Internal helpers
    //=========================================================================

    async _fetchRaw(method: string, url: string, body: any = null): Promise<Response> {
        return await fetch(url, {
            method,
            headers: {
                Authorization: `Bearer ${this._token}`,
                Accept: "application/vnd.github+json",
                "X-GitHub-Api-Version": "2022-11-28",
                "User-Agent": "raiment-studios-guidebook",
            },
            body: body ? JSON.stringify(body) : undefined,
        });
    }
}
