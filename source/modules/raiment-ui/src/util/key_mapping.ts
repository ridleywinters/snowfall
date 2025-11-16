export type KeyMappingTable<T> = Record<string, (evt: T) => void>;

export function handleKeyMapping<
    T extends {
        key: string;
        ctrlKey: boolean;
        altKey: boolean;
        metaKey: boolean;
        shiftKey: boolean;
        preventDefault(): void;
        stopPropagation(): void;
    },
>(
    evt: T,
    table: KeyMappingTable<T>,
): void {
    let key = evt.key;
    if (evt.shiftKey) {
        key = `Shift+${key}`;
    }
    if (evt.altKey || evt.metaKey) {
        key = `Alt+${key}`;
    }
    if (evt.ctrlKey) {
        key = `Ctrl+${key}`;
    }

    const tableKeys = Object.keys(table);
    let handler = undefined;
    let stopEvent = true;
    for (let i = 0; i < tableKeys.length; i++) {
        const originalKey = tableKeys[i];
        let k = originalKey;
        if (k.endsWith("?")) {
            k = k.slice(0, -1);
            stopEvent = false;
        }
        if (k !== key) {
            continue;
        }
        handler = table[originalKey];
        break;
    }
    if (handler) {
        if (stopEvent) {
            evt.preventDefault();
            evt.stopPropagation();
        }
        handler(evt);
    } else {
        // console.log("No handler for key:", key);
    }
}
