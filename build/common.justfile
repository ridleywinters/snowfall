# DANGER: hard-coded assumption that this is where the module will be stored.
# I experimented with trying to determine this dynamically, but was unable to
# find a good solution.
RAIMENT_DEVENV_DIR := "$REPO_ROOT/source/modules/raiment-devenv"

# Copies the common vscode settings into the local project
[private]
ensure-vscode-directory:
    @echo "Linking local .vscode directory to common settings"
    @mkdir -p .vscode
    @rm -f .vscode/settings.json
    @ln -sf "{{RAIMENT_DEVENV_DIR}}/.vscode/settings.json" ".vscode/settings.json"
    @rm -f .vscode/extensions.json
    @ln -sf "{{RAIMENT_DEVENV_DIR}}/.vscode/extensions.json" ".vscode/extensions.json"    
    @{{RAIMENT_DEVENV_DIR}}/build/ensure_gitignore_lines.ts \
        "/.vscode"

# Symbolically links the common build directory
#
# This moves a host of common build tooling and scripts into the local directory
# for convenience.  This approach avoids absolute path and relative path references that
# can obfuscate the build process.
[private]
ensure-build-directory:
    @echo "Linking to common build directory"
    @rm -rf build
    @ln -sf "{{RAIMENT_DEVENV_DIR}}/build" "build"
    @./build/ensure_gitignore_lines.ts \
        "/build" \
        "/target" \
        "/node_modules" \
        "/.vscode"
    @just ensure-vscode-directory
    @./build/vscode_settings_to_workspace.ts ".vscode/settings.json" .



#==============================================================================
# sync
#==============================================================================

# Syncs all subtrees and pushes to origin
[private]
repo-sync:
    git status --short
    @git diff-index --quiet HEAD --
    git fetch
    git pull
    git lfs push --all https://github.com/ridleywinters/lfs-host.git
    -git remote add raiment-devenv git@github.com:ridleywinters/raiment-devenv.git 2> /dev/null
    -git remote add raiment-core git@github.com:ridleywinters/raiment-core.git 2> /dev/null
    -git remote add raiment-ui git@github.com:ridleywinters/raiment-ui.git 2> /dev/null
    -git remote add raiment-shell git@github.com:ridleywinters/raiment-shell.git 2> /dev/null
    @just repo-subtree-pull
    @just repo-subtree-push
    @just repo-subtree-pull
    @just repo-subtree-push
    git push

[private]
repo-subtree-pull:
    @echo "Pulling subtrees..."    
    git subtree pull --prefix=source/modules/raiment-devenv raiment-devenv main --squash --message="Merge commit"
    git subtree pull --prefix=source/modules/raiment-core raiment-core main --squash --message="Merge commit"
    git subtree pull --prefix=source/modules/raiment-ui raiment-ui main --squash --message="Merge commit"
    git subtree pull --prefix=source/modules/raiment-shell raiment-shell main --squash --message="Merge commit"

[private]
repo-subtree-push:
    @echo "Pushing subtrees..."
    git subtree push --prefix=source/modules/raiment-devenv raiment-devenv main
    git subtree push --prefix=source/modules/raiment-core raiment-core main
    git subtree push --prefix=source/modules/raiment-ui raiment-ui main
    git subtree push --prefix=source/modules/raiment-shell raiment-shell main