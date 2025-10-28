# extern

The `extern` folder is intended for storing assets or content created outside this project.

Storing the original, unmodified data in the repo is being done to ensure there is a clear "source of truth" for the attribution as well as unambiguous tracking of any modifications to the original assets.

### Asset pipeline

The assets pipeline roughly works as follows:

- Original data for external assets is stored in `extern/source/<name of assets>`
- The `just expand` recipe will:
  - Decompress, rename, or otherwise move the raw assets into the `expanded` directory
  - This step occurs to make the assets easier to work with for other scripts

The `source/content` directory contains assets created by contributors to this project.

The `source/assets` directory contains scripts for building / converting both the external assets and the internal assets to normalize formats usable directly in game. It also ensures all final assets are accompanied by a `<asset name>.meta.md` file containing all licensing and attribution information so that is easily discoverable.
