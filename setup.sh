#!/usr/bin/env bash

#
# This is the "required" setup script for using the repo.
# A couple things to keep in mind:
#
# - Try to keep this minimal & fast
# - Try to keep this from "polluting" the user's computer outside the repo
# - Try to keep this cross-platform (at least Mac + most Linux-es)
#

#==============================================================================
# Ensure the script is being sourced, not executed
#=============================================================================

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  echo "ERROR: This script must be run via 'source' to ensure environment variables are set in the current shell."
  echo
  echo "Usage: source setup.sh"
  return 1 2>/dev/null || exit 1
fi

echo "Setting up development environment..."
echo

#==============================================================================
# Bootstrap with a local install of Rust
#==============================================================================

# Store the root of the repository as a well-known, "stable" reference
# environment variables which scripts can access other assets in the repo.
# https://stackoverflow.com/questions/59895/how-can-i-get-the-source-directory-of-a-bash-script-from-within-the-script-itsel
export REPO_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

function _prepend_to_path() {
    if [ -d "$1" ]; then
        PATH=":$PATH:"
        PATH="${PATH//:$1:/:}"
        PATH="${PATH#:}"
        PATH="${PATH%:}"
        PATH="$1${PATH:+":$PATH"}"
    fi
}

# Install Rust within the repo
export RUSTUP_HOME="$REPO_ROOT/bin/rustup"
export CARGO_HOME="$REPO_ROOT/bin/cargo"
if [ ! -f "$CARGO_HOME/bin/rustup" ]; then
    echo
    echo "Installing local tools..."
    echo "This may take a while, but is only required once."
    echo 
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -sSf | RUSTUP_INIT_SKIP_PATH_CHECK=yes sh -s -- -y
fi

_prepend_to_path "$REPO_ROOT/bin"
_prepend_to_path "$CARGO_HOME/bin"
unset -f _prepend_to_path

#==============================================================================
# Install other prerequisite tools locally
#==============================================================================

if [ ! -f "$CARGO_HOME/bin/cargo-binstall" ]; then
    cargo install cargo-binstall
    echo 
fi
if [ ! -f "$CARGO_HOME/bin/just" ]; then
    cargo binstall -y just
    echo 
fi
if [ ! -f "$CARGO_HOME/bin/deno" ]; then
    cargo binstall -y deno
    echo 
fi
if [ ! -f "$CARGO_HOME/bin/mprocs" ]; then
    cargo binstall -y mprocs
    echo 
fi
if [ ! -f "$CARGO_HOME/bin/watchexec" ]; then
    cargo binstall -y watchexec-cli
    echo 
fi
if [ ! -f "$REPO_ROOT/bin/apps/blender" ]; then
    ./source/scripts/download-blender.ts "$REPO_ROOT/bin/apps"
    ln -s "$REPO_ROOT/bin/apps/blender" "$REPO_ROOT/bin/blender"
    echo
fi

#==============================================================================
# Output for verification & debugging
#==============================================================================

# Show the command version and path. Useful for debugging if there are build 
# issues.
_show_command_info() {
    local cmd_name="$1"
    local cmd_path version rel_path

    cmd_path="$(command -v "$cmd_name" 2>/dev/null)"
    if [[ -z "$cmd_path" ]]; then
        echo "$cmd_name not found"
        return 1
    fi

    # Extract first semver from "<cmd> --version" output
    version="$("$cmd_name" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n1)"
    [[ -z "$version" ]] && version="unknown"

    # Compute path relative to repo root
    if [[ "$cmd_path" == "$REPO_ROOT"* ]]; then
        rel_path="${cmd_path#$REPO_ROOT/}"
    else
        rel_path="../${cmd_path#$HOME/}"
    fi
     
    local c_path=$'\e[38;2;128;128;128m'
    local c_version=$'\e[38;2;176;196;222m'
    local c_name=$'\e[38;2;218;165;32m'
    local reset=$'\e[0m'
    printf "    ${c_name}%-12s ${c_version}v%-12s ${c_path}./%s${reset}\n" "$cmd_name" "$version" "$rel_path"
}

echo "Local installations of:"
_show_command_info "cargo"
_show_command_info "rustc"
_show_command_info "deno"
_show_command_info "just"
_show_command_info "mprocs"
_show_command_info "watchexec"
_show_command_info "blender"
echo

unset -f _show_command_info

#==============================================================================
# git setup
#==============================================================================

# For each file in $REPO_ROOT/config/git-hooks, link to local hooks.
#
# The symbolic link allows the same file to both be active (in the hooks
# directory outside the repo) and stored in the repo as well.
for source in $REPO_ROOT/source/modules/raiment-devenv/config/git-hooks/*; do
    hook=$(basename $source)
    target=$REPO_ROOT/.git/hooks/$hook
    rm -f $target
    ln -s $source $target
    chmod +x $target
done


git remote add raiment-devenv git@github.com:ridleywinters/raiment-devenv.git 2> /dev/null || true
git remote add raiment-core git@github.com:ridleywinters/raiment-core.git 2> /dev/null || true
git remote add raiment-ui git@github.com:ridleywinters/raiment-ui.git 2> /dev/null || true
git remote add raiment-shell git@github.com:ridleywinters/raiment-shell.git 2> /dev/null || true
#git subtree add --prefix=source/modules/raiment-devenv raiment-devenv main --squash 2> /dev/null || true

#==============================================================================
# Aliases and tools
#==============================================================================

source "$REPO_ROOT/source/scripts/set_aliases.sh"

#==============================================================================
# Done.
#==============================================================================

echo "Setup complete. Use 'just' to run common tasks."
echo 
just
