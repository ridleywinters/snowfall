import * as coreImports from "./src/index.ts";


// Note: this likely breaks "tree-shaking" (optimization of unused exports), but is convenient
// and this library is optimzied for prototyping, not production
export const core = coreImports;
export * from "./src/index.ts";
