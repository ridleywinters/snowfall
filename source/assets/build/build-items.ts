#!/usr/bin/env -S deno run --allow-read --allow-write --allow-run --allow-env

/**
 * Merges all the individual item YAML files into a single items.yaml file.
 *
 * The individual files are easier to work with from an editing perspective
 * whereas a consolidated file is easier for the system to consume.
 */

import { sh } from "@raiment-shell";
import * as core from "@raiment-core";
import { normalize, relative } from "@std/path";

const SRC_DIR = sh.template("$REPO_ROOT/source/content/items");
const ATTRIBUTION_FILE = `${SRC_DIR}/attribution.meta.md`;
const DST_DIR = sh.template("$REPO_ROOT/source/assets/base/items");

const merged: any = {
    items: {},
};
for (const file of await sh.glob(`${SRC_DIR}/*.yaml`)) {
    const data: any = core.parseYAML(await sh.read(file));
    const item = { ...data.item };
    merged.items[item.id] = { ...item };
}

const filename = normalize(`${DST_DIR}/items.yaml`);

await sh.mkdir(DST_DIR);
await sh.write(filename, core.stringifyYAML(merged));
await sh.copy(ATTRIBUTION_FILE, `${DST_DIR}/attribution.meta.md`);
sh.cprintln(
    `[:check:](green) Merged {{count}} items to [{{filename}}](goldenrod)`,
    {
        count: Object.keys(merged.items).length,
        filename: relative(".", filename),
    },
);
