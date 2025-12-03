# Copilot Instructions

Keep these guidelines minimal and practical. Follow existing project patterns where present.

## General

- Make small, reviewable commits with clear commit messages.
- Run project tests and linters locally where available before opening a PR.
- If debugging an issue is taking a long time, write small unit tests to confirm your assumptions.
- Write short, general, independent, reusable helper functions when you can.
- Exported or public functions or other symbols generally should be at the top of the file.
- Place helper functions at the bottom of the file or in separate modules.
- In general, the functions should be ordered in the order of the call hierarchy (i.e., higher-level functions first, lower-level helper functions later) when possible.
- Follow the style in surrounding files.
- Avoid making changes that were not requested.
- Write short, general, independent, reusable helper functions when you can.
- Exported or public functions or other symbols generally should be at the top of the file.
- Place helper functions at the bottom of the file or in separate modules. If a helper is only used by that file, prefer placing it at the bottom (below exported APIs). If the helper is shared, move it to a separate module and import it.
- In general, the functions should be ordered in the order of the call hierarchy (i.e., higher-level functions first, lower-level helper functions later) when possible.
- Follow the style in surrounding files.
- Avoid making changes that were not requested.
- Prefer variable names that are either a full word or a single letter unless there is a strong convention otherwise (e.g., `cfg` for configuration).

Checklist before editing

- Scan nearby files in the same directory to observe local conventions (helper placement, export style, naming).
- Confirm helper placement: bottom-of-file for file-local helpers, separate module for shared helpers.
- Check for name shadowing (avoid using the same name for a function and a parameter).
- Ensure conditionals use braces (no single-line `if` without braces).

## Rust

- Use `cargo fmt` / `rustfmt` for formatting; prefer idiomatic Rust and clear ownership.
- Add small focused changes; avoid large refactors in the same PR.
- Include or update unit tests when behavior changes; run `cargo test` before submitting.
- Prefer descriptive error types (use `thiserror` or `anyhow` only where consistent with the repo).
- Don’t change public APIs unless necessary; bump semver-aware changes with a note.

## TypeScript (Deno)

- Use `deno` tooling for formatting and linting.
- Do not use `node`, `npm`, `npx`, or `yarn`.
- Prefer explicit return types for exported functions and keep helper functions small.
- Avoid helper functions defined within other functions if they do not have dependencies on outer function scope.
- Use async/await, handle errors explicitly, and avoid swallowing exceptions.
- Keep changes focused and avoid reformatting unrelated files.
- Always use curly braces for conditional statements; avoid single-line `if` without braces.
- Use `i` or `index` for indicies. Do not use `idx`.

## Bash

- Add a shebang and keep permissions executable when needed (`chmod +x`).
- Keep scripts idempotent and document required environment variables.

If you need more detailed rules, ask for an expanded style guide.
