import x"$REPO_ROOT/source/modules/raiment-devenv/build/common.justfile"

ensure:
    @just ensure-vscode-directory

build: ensure

test: build
    deno check src/
    deno test --allow-read src/