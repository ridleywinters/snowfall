# github authentication

An interface for GitHub authentication.  There are two intended entry-points.

Both use the `guidebook-app` GitHub OAuth app and rely the guidebook auth server being up and running on the public internet.


### Simplifed low-level authentication hook

```ts
const { accesToken, gitHubSignIn, gitHubSignOut } = useGitHubAuth();
```

This returns the current authentication token if logged in, null otherwise. `gitHubSignIn` is a function that will navigate to begin the GitHub OAuth authentication. For example, the `onClick` handler on a login button would call this.  Correspondingly, a logout button would call `githubSignOut` in its click handler.

### Simplified GitHubAPI hook

```tsx
const api = useGitHubAPI();
const user = api.useUser();

return (
    <div>
        <div>
            {api.token ? (
                <button onClick={() => api.signOut()}>Sign Out</button>
            ) : (
                <button onClick={() => api.signIn()}>Sign In</button>
            )}
        </div>
        <div>Auth token: {api.token}</div>
        <div>User info:</div>
        <pre>{JSON.stringify(user, null, 2)}</pre>
    </div>
);
```

This hook returns an API object which is a simplified wrapper on a subset of GitHUB REST API calls. It _does not_ attempt to provide full coverage of the extensive GitHub API, but rather includes only the subset that has been used in the Guidebook projects.  That said it provides some basic conveniences:

- Sub-hooks for common state like the user profile
- APIs for reading and updating files directly in GitHub repos
- Caching of API calls that do not vary often (e.g. user profile)
- Wrappers for generic GitHub API calls (handling the necessary auth header automatically)

