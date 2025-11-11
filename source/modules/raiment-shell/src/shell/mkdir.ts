import { ensureDir } from "@std/fs";

export function mkdir(path: string): Promise<void> {
    return ensureDir(path);
}
