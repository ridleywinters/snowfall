COMMON_DIR := "${REPO_ROOT}/source/common"

# Copies the common vscode settings into the local project
[private]
ensure-vscode-directory:
    @echo "Linking local .vscode directory to common settings"
    @mkdir -p .vscode
    @rm -f .vscode/settings.json
    @ln -sf "{{COMMON_DIR}}/.vscode/settings.json" ".vscode/settings.json"
    @rm -f .vscode/extensions.json
    @ln -sf "{{COMMON_DIR}}/.vscode/extensions.json" ".vscode/extensions.json"

# Symbolically links the common build directory
#
# This moves a host of common build tooling and scripts into the local directory
# for convenience.  This approach avoids absolute path and relative path references that
# can obfuscate the build process.
[private]
ensure-build-directory:
    @echo "Linking to common build directory"
    @rm -rf build
    @ln -sf "{{COMMON_DIR}}/build" "build"
    @./build/ensure_gitignore_lines.ts \
        "/build" \
        "/target" \
        "/.vscode"
    @just ensure-vscode-directory
    @./build/vscode_settings_to_workspace.ts ".vscode/settings.json" .

