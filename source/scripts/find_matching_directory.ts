#!/usr/bin/env -S deno run --allow-read --allow-env
/**
 * Simple fuzzy string matcher (i.e. accoutn for smal typos) that
 * returns the nearest matching directory path from a set of candidate directories.
 *
 * This is a simple script **hard-coded** to this project's directory structure!
 * It is designed to be practical & convenient, not a general-purpose solution.
 */

import { distance as levenshtein } from "npm:fastest-levenshtein@1.0.16";

type MatchResult = {
  match: string;
  score: number;
};

//=============================================================================
// Main entry point
//=============================================================================

async function main() {
  const args = Deno.args;

  if (args.length === 0) {
    console.error("Usage: find_matching_directory.ts <search_term>");
    Deno.exit(1);
  }

  const searchTerm = args[0];

  const repoRoot = Deno.env.get("REPO_ROOT");
  if (!repoRoot || repoRoot.trim() === "") {
    console.error(
      "Error: REPO_ROOT environment variable is not set or is empty"
    );
    Deno.exit(1);
  }

  let result: MatchResult | null = null;

  // Special case for the root of the repo
  if (searchTerm === "/") {
    result = {
      match: repoRoot,
      score: 0,
    };
  } else {
    // Dynamically compute the candidates based on some hard-coded assumptions
    // about where the "important" directories are located.
    const candidates: string[] = [];
    candidates.push(...(await readDirectories(repoRoot)));
    candidates.push(...(await readDirectories(`${repoRoot}/source/assets`)));
    candidates.push(...(await readDirectories(`${repoRoot}/source/cmd`)));
    candidates.push(...(await readDirectories(`${repoRoot}/source/crates`)));
    candidates.push(...(await readDirectories(`${repoRoot}/source/modules`)));
    candidates.push(...(await readDirectories(`${repoRoot}/source/tools`)));
    candidates.push(...(await readDirectories(`${repoRoot}/source`)));
    result = findClosestMatch(searchTerm, candidates);
  }

  if (result) {
    console.log(result.match);
  } else {
    console.error(`No matches found for: ${searchTerm}`);
    console.log(".");
    Deno.exit(1);
  }
}

//=============================================================================
// Helper functions
//=============================================================================

async function readDirectories(path: string): Promise<string[]> {
  const directories: string[] = [];
  for await (const dirEntry of Deno.readDir(path)) {
    if (dirEntry.name.startsWith(".")) {
      continue; // Skip hidden directories
    }
    if (dirEntry.isDirectory) {
      directories.push(`${path}/${dirEntry.name}`);
    }
  }
  return directories;
}

/**
 * Find the closest matching string with prefix bias
 */
function findClosestMatch(
  query: string,
  candidates: string[]
): MatchResult | null {
  const MAX_SCORE = 2;

  if (candidates.length === 0) {
    return null;
  }

  const queryLower = query.toLowerCase();
  let bestMatch = candidates[0];
  let bestScore = Infinity;

  for (const candidate of candidates) {
    // Compare only the final directory name, not the full path
    const comparisonString = candidate.split("/").pop()?.toLowerCase() ?? "";

    let score = levenshtein(
      queryLower,
      comparisonString.slice(0, queryLower.length)
    );
    if (comparisonString.startsWith(queryLower)) {
      score -= 2; // Heavy bias for prefix matches
    }
    // Bonus for prefix match at word boundaries
    const words = comparisonString.split(/[-_]/);
    for (const word of words) {
      if (word.startsWith(queryLower)) {
        score -= 1;
        break;
      }
    }
    if (score < bestScore) {
      bestScore = score;
      bestMatch = candidate;
    }
  }

  if (bestScore > MAX_SCORE) {
    return null;
  }
  return {
    match: bestMatch,
    score: bestScore,
  };
}

if (import.meta.main) {
  await main();
}
