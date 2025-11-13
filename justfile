import x"source/modules/raiment-devenv/build/common.justfile"

#==============================================================================
# default
#==============================================================================

[private]
default:
    @just --list --unsorted

#==============================================================================
# ensure
#==============================================================================

ensure:
    @just ensure-vscode-directory

#==============================================================================
# demo
#==============================================================================

# Runs the demo project
demo:
    echo "TODO"

#==============================================================================
# build
#==============================================================================

# Builds all projects
build: ensure
    cd source/assets && just build
    @just run-foreach "source/modules" build
    @just run-foreach "source/cmd" build    
    @just run-foreach "source/tools" build

#==============================================================================
# dev
#==============================================================================

dev-test:
    watchexec \
        --watch source \
        --exts rs,js,jsx,ts,tsx,html,css,png,yaml,json \
        "just test"

#==============================================================================
# test
#==============================================================================

# Tests all projects
test: build
    @just run-foreach "source/cmd" test
    @just run-foreach "source/modules" test

#==============================================================================
# sync
#==============================================================================

# Syncs all subtrees and pushes to origin
sync: repo-sync

#==============================================================================
# publish
#==============================================================================

# Publishes all projects 
publish:
    echo "TODO"

#==============================================================================
# clean
#==============================================================================

# Restores the repository to a clean state
clean:
    git clean -fdx
    find . -type d -empty -delete

[private]
clean-bin:
    cd bin && git clean -fdx

[private]
clean-temp:
    cd temp && git clean -fdx

#==============================================================================
# Internal utilities
#==============================================================================

[private]
run-foreach root_dir command:
    #!/usr/bin/env bash
    set -euo pipefail
    for dir in "{{root_dir}}"/*/; do
        if [ -d "$dir" ]; then
            if [ ! -f "$dir/justfile" ]; then
                echo "Error: No justfile found in $(basename "$dir")"
                exit 1
            fi
            printf '\033[38;5;214m{{command}} %s\033[0m\n' "$dir"
            (cd "$dir" && just "{{command}}")
        fi
    done
