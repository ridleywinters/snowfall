import "../../common/common.justfile"

ensure:
    @just ensure-vscode-directory

build: ensure

test:
    deno test --allow-read src/
