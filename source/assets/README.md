# assets

Builds the normalized assets for use by the game engine.

## Design

#### Uses a "pull" based design

Rather than content folders containing the scripts to "push" their content by converting and exporting the source data into game-ready format, the assets folder contains the scripts to import that content.

This generally helps with normalization to the in-game format. Any change in the needs of the in-game format requires a change in this folder only, rather than potentially across multiple different content folders (where it may not be clear which folders are affected by the change).

This also helps keep the content colder solely composed of the source files for those assets without cluttering them with conversion scripts or intermediate representations.
