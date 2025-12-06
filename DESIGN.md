# Design

## Miscellaneous design notes:

- The player character is common adventurer, not a "chosen one"
- There aren't classes: only Skills
- The player can choose initial Skills and skill levels
- The player can choose initial Conditions. Example: adding/removing Hunger would mean they do / do not need to regularly eat to control their fatigure level
- The player character does not have magic or know spells, at least not in the traditional D&D RPG sense
- There may be some mechanism for manipulating the Maelstrom which may act somewhat like a magic mechanism, but that is TBD
- Potions should eventually be very rare and expensive
- Physicians, herbalists, rest, and food should be the primary ways to regenerate helath
- Oils for weapons (ala Witcher 3) should exist
- There is no automap. The player knows the region they are in, they can find/buy a map of the region, but it will not automark where they are in that region (finding visual cues to pinpoint their position should be a fun way to figure out where they are)
- The game should not be "brutally hard" like many roguelikes (despite some roguelike characteristics to the game)
- The game should feel both small and massive: shadowy dungeons with light reaching only meters ahead explored with no map, misty forests with trees whose peaks are obscured as they reach into the low drifting clouds of rain, fierce snowy nights where the stretch between houses is journey enough to bring danger. Those shadowy corners, misty distances, and blinded nights should engage the imagination to make world of Galthea even larger by its very mystery. And the world should be filled with countless such places.


## Engine design

- Avoid a framework design; keep individual modules as independent as possible
- Use lightweight adapters & glue code to bind dependencies
- Prefer straightforward usage over the last 2% of performance optimization
