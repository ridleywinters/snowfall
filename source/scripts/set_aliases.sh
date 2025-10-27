#!/usr/bin/env bash

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  echo "ERROR: This script must be run via 'source' to ensure aliases are set in the current shell."
  echo
  echo "Usage: source set_aliases.sh"
  return 1 2>/dev/null || exit 1
fi

#==============================================================================
# scd ("simple" change directory)
#==============================================================================

function scd() {
  cd "$($REPO_ROOT/source/scripts/find_matching_directory.ts "$1")"
}

#==============================================================================
# gs ("git status") 
#==============================================================================

function gs() {
    git status
}

#==============================================================================
# gcap (git commit and push)
#==============================================================================
#
# HEADS UP! This shortcut is a convenience for early development to make 
# committing new code faster. It is NOT a good practice for long-term use if
# and when there are multiple contributors and the repository is larger.
# 
# This should be removed if and when this project grows in complexity.
#
function gcap() {
    pushd $REPO_ROOT
    git add .
    git commit -m "$*"
    git push
    popd
}

