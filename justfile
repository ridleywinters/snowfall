import "source/common/common.justfile"

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
# test
#==============================================================================

# Tests all projects
test: build
    @just run-foreach "source/modules" test

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
            echo "--- Running 'just {{command}}' in $dir ---"
            (cd "$dir" && just "{{command}}")
        fi
    done
