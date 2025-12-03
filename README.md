# ❄️ Snowfall

Snowfall is a long-term, hobby project to create an open source, retro-style, single-player fantasy life simulator role playing game. It is set in the fictional fantasy world of Galthea. It draws inspiration from games like Daggerfall, the Bard's Tale (the original!), and the Witcher series. It is intended to (eventually) be very well-documented and trivial to contribute to.

![](./media/screenshot-2025-10-29-114358.png)

## Status

Very early prototyping phase! Not much to see yet.

## Design goals

Snowfall aims to provide a massive open world, both in game and in lore. While every game should be fun to play, Snowfall also aims to be easy and enjoyable to contribute to. In-game editors and content management should make contributions trivial. Creating mods and even full new "distributions" can be a core part of the experience of Snowfall.

**Gameplay**

The game is intended to be easy to play but very difficult to "win." You play as an average person in Galthea seeking to make a life of your own. You can do this by farming, trading, engaging in small quests in the town you choose to call home. Or you can join the hapless throngs of self-titled adventurers across the continent pursuing the grand quest to find a way to stop the Maelstrom, a cosmic force growing in power every day that is unraveling the world and reality -- but you're not the chosen one! Don't expect that fate to pay you any particular attention.

**Content**

The game assembles hand-crafted content together with procedural variations to make every playthrough of the gameboth familiar and unique. And, using a sort of magic within the world of Galthea, the player can sometimes cause specific elements of their world -- friendships, locations, knowledge, boons -- to leak into future realities. Thus, will the impending destruction of the Maelstrom is seemingly inevitable, maybe there is a way to make life a little easier in some future incarnation?

**Engine**

The engine is designed to have a retro-feel reminscient of games from mid to late 1990s. Axis-aligned geometry, 2.5D heightmaps, and 2D sprites are intended to be common elements to promote this feel, keep the code simple, and allow modern computers to handle a great deal of concurrent elements.

The engine eventually will support in-game editing of most content. It may also have a voxel-like system to make dynamic construction and destruction of geometry possible.

## Tech stack

Rust core using Bevy and TypeScript (Deno) for scripting and tools.

Architecturally, it is by designed to prefer simplicity, modularity, and code maintainability over raw runtime efficiency or unique functionality. Ease of contribution is a priority for the codebase as well as the game content.

## Feature map

- [ ] Native `.blend` asset loading
- [ ] Deno/TypeScript scripting
- [ ] Extrude faces dynamically

## Roadmap

The major phases of development:

#### Base engine

Using Barony as a template, build a block-based, retro-style RPG engine utilizing Bevy.

- Get the basic building blocks for the engine and user interactions in place
- Use placeholder art and assets; don't worry about art style yet

#### Open world basics

Capture some of the feel of Daggerfall's massive world.

- Get city-states, villages, and wilderness in the game
- Continue to use placeholder art and assets

#### Simulation mechanics

With support of a large world, cities, and dungeons, shift focus to game mechanics to make this feel not just like a roguelike RPG but rather like a fantasy life simulator.

- Add Stardew Valley-esque mechanics to the town
- Use placeholder art and assets

#### Playtesting

All the major gameplay and game mechanics should be in place. Begin playtesting to make sure the game is actually fun!

- Iterate on core game mechanics
- Gradually add detail necessary to flesh out systems that feel incomplete

#### Content

Improve the in-game UI / UX and expand it to allow more direct, intuitive editing of the game world.

- Full in-game mod support
- Modding API
- Expanded content and gameplay mechanics

#### Lore and content

Build out the lore and worldbuilding to ensure design coherency and consistency.

- Begin work on higher quality assets for finalized gameplay systems

#### Version 2.0 and beyond...

A long, long-term goal is to rewrite the engine to use a custom, voxel-based rendering and simulation system.

## Contributing

Contributions are very welcome!

## Directory structure

```
bin/                - locally installed binaries
config/             - <placeholder>
resources/          - non-game assets
extern/             - external assets (not created by contributors)
source/             - all source code
    assets/         - <placeholder>
    cmd/            - all binaries
        fallgray/   - the main game
    common/         - source related files used by multiple projects
    crates/         - shared Rust libraries
    modules/        - shared TypeScript libraries
    scripts/        - build-related single-file scripts
    tools/          - larger utilities
```

## FAQ

#### Why Rust?

I wanted a strongly-typed language as it seems best for refactoring effectively as well as coordinating with a large number of engineers. In my own experience, Rust's tooling is straightforward and helps keep more time spent on the code itself than those tools.

#### Why Bevy?

It is has an active community and is well-documented. Ideally less time will be spent reinventing common game engine subsystems by utilizing Bevy. An eventual project goal is to write a custom voxel-based engine (partly, if not mostly, because that would be enjoyable), the higher priority is to ensure the gameplay goals are met. This means custom code should be deprioritized until then.
