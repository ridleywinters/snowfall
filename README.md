# Fallgray

Fallgray is a long-term, hobby project to create an open source, retro-style, single-player fantasy life simulator role playing game. It is set in the fictional fantasy world of Galthea. It draws inspiration from games like Daggerfall and Barony.

It aims to provide a massive open world, both in game and in lore, that is fun to play and easy for players to contribute to. In-game editors and content repositories make contributions trivial for both novice and expert users. Creating mods and even full new "distributions" are considered a core part of the experience of Fallgray.

The game is intended to be easy to play but very difficult to "win." As a fantasy life simulator role-playing game, you play as an average person in the world of Galthea seeking to make a life of your own. You can do this by farming, trading, and going on small quests in the local areas. Or you can pursue the grand quest of trying to uncover a way to stop the destructive cosmic force known as the Maelstrom from unraveling the world. The game uses a mix of hand-crafted content with procedural elements to ensure each playthrough is a new variation of the world with subtle influences apparent from prior playthroughs.

## Status

Currently the project is just getting started!

## Tech stack

Rust core using Bevy and TypeScript (Deno) for scripting and tools.

Architecturally, it is by designed to prefer simplicity, modularity, and code maintainability over raw runtime efficiency or unique functionality. Ease of contribution is a priority for the codebase as well as the game content.

## Roadmap

The major phases of development:

1. Base engine: using Barony as a template, build a block-based, retro-style RPG engine utilizing Bevy
2. Open world: add city-states, villages, and wilderness to ensure the engine has a Daggerfall massive-world feel
3. Simulator mechanics: add "Stardew Valley meets Daggerfall" mechanics to ensure the fantasy life simulator goal is being delivered met
4. Content management: in-game editors and modding
5. Content & lore: all the core gameplay should be in place, so add more details and variations

Version 2.0

A long, long-term goal is to rewrite the engine to use a custom, voxel-based rendering and simulation system.

## Contributing

Contributions are very welcome!

## Directory structure

```
bin/                - locally installed binaries
config/             - <placeholder>
resources/          - non-game assets like
source/             - all source code
    assets/         - <placeholder>
    cmd/            - all binaries
        fallgray/   - the main game
    common/         - source related files used by multiple projects
    crates/         - shared Rust libraries
    modules/        - shared TypeScript libraries
    scripts/        - build-related scripts
    tools/          - larger utilities
```
