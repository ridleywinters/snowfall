import React, { JSX } from "react";

// 8x8 grid
// can be rotated any of 4 orientations or mirrored
// stored with hash to identify unique shapes

type direction = "Y+" | "Y-" | "X+" | "X-";

function addPos(p: Pos, dir: direction): Pos {
    switch (dir) {
        case "Y+":
            return { x: p.x, y: p.y + 1 };
        case "X+":
            return { x: p.x + 1, y: p.y };
        case "Y-":
            return { x: p.x, y: p.y - 1 };
        case "X-":
            return { x: p.x - 1, y: p.y };
    }
}

type Pos = {
    x: number;
    y: number;
};

class Cell {
    map = new Array(8 * 8).fill(0);

    static from(lines: string[]): Cell {
        const cell = new Cell();
        for (let y = 0; y < 8; y++) {
            for (let x = 0; x < 8; x++) {
                cell.set(x, y, lines[y][x] === "#" ? 1 : 0);
            }
        }
        return cell;
    }

    clone(): Cell {
        const cell = new Cell();
        cell.map = [...this.map];
        return cell;
    }

    hash(): string {
        return this.map.join("");
    }

    set(x: number, y: number, value: number) {
        this.map[y * 8 + x] = value;
    }

    get(x: number, y: number): number {
        return this.map[y * 8 + x];
    }

    data(): number[][] {
        const data: number[][] = [];
        for (let y = 0; y < 8; y++) {
            const row: number[] = [];
            for (let x = 0; x < 8; x++) {
                row.push(this.map[y * 8 + x]);
            }
            data.push(row);
        }
        return data;
    }

    exits(): { x: number; y: number }[] {
        const exits: { x: number; y: number }[] = [];
        for (let y = 0; y < 8; y++) {
            for (let x = 0; x < 8; x++) {
                if (this.get(x, y) === 0) {
                    if (x === 0 || y === 0 || x === 7 || y === 7) {
                        exits.push({ x, y });
                    }
                }
            }
        }
        return exits;
    }

    alignsToExits(that: Cell, dir: direction): boolean {
        const exits = that.exits().filter((exit) => {
            return (dir === "Y+" && exit.y === 7) ||
                (dir === "Y-" && exit.y === 0) ||
                (dir === "X+" && exit.x === 7) ||
                (dir === "X-" && exit.x === 0);
        });
        for (const exit of exits) {
            const entrance = dir == "Y+"
                ? { x: exit.x, y: 0 }
                : dir == "Y-"
                ? { x: exit.x, y: 7 }
                : dir == "X+"
                ? { x: 0, y: exit.y }
                : { x: 7, y: exit.y };

            if (this.get(entrance.x, entrance.y) !== 0) {
                return false;
            }
        }
        return true;
    }

    draw(fn: (x: number, y: number) => number) {
        for (let y = 0; y < 8; y++) {
            for (let x = 0; x < 8; x++) {
                this.map[y * 8 + x] = fn(x, y);
            }
        }
    }

    mirrorX(): Cell {
        const map = new Array(8 * 8).fill(0);
        for (let y = 0; y < 8; y++) {
            for (let x = 0; x < 8; x++) {
                map[y * 8 + x] = this.map[y * 8 + (7 - x)];
            }
        }
        this.map = map;
        return this;
    }

    mirrorY(): Cell {
        const map = new Array(8 * 8).fill(0);
        for (let y = 0; y < 8; y++) {
            for (let x = 0; x < 8; x++) {
                map[y * 8 + x] = this.map[(7 - y) * 8 + x];
            }
        }
        this.map = map;
        return this;
    }
    rotate90(n: number): Cell {
        n = n % 4;
        let map = this.map;
        for (let i = 0; i < n; i++) {
            const newMap = new Array(8 * 8).fill(0);
            for (let y = 0; y < 8; y++) {
                for (let x = 0; x < 8; x++) {
                    newMap[x * 8 + (7 - y)] = map[y * 8 + x];
                }
            }
            map = newMap;
        }
        this.map = map;
        return this;
    }

    display() {
        let text = "";
        for (let y = 0; y < 8; y++) {
            let row = "";
            for (let x = 0; x < 8; x++) {
                row += this.map[y * 8 + x] ? "#" : ".";
            }
            text += row + "\n";
        }
        return text;
    }
}

function selectRnd<T>(arr: T[]): T {
    for (let i = 0; i < arr.length; i++) {
        Math.random();
    }
    const index = Math.floor(Math.random() * arr.length);
    return arr[index];
}

class Database {
    cells: Record<string, Cell> = {};

    add(cell: Cell) {
        this.cells[cell.hash()] = cell;
    }

    addPermutations(cell: Cell) {
        for (let f = 0; f < 2; f++) {
            if (f === 1) {
                cell = cell.clone().mirrorX();
            }
            for (let r = 0; r < 4; r++) {
                cell = cell.clone().rotate90(r);
                this.add(cell);
            }
        }
    }

    selectRandom(): Cell {
        const keys = Object.keys(this.cells);
        const key = selectRnd(keys);
        return this.cells[key];
    }

    select(fn: (cell: Cell) => boolean): Cell | null {
        const orderedKeys = Object.keys(this.cells);
        const keys = [];
        while (orderedKeys.length > 0) {
            const index = selectRnd(orderedKeys.map((_, i) => i));
            keys.push(orderedKeys.splice(index, 1)[0]);
        }
        for (const key of keys) {
            const cell = this.cells[key];
            if (fn(cell)) {
                return cell;
            }
        }
        return null;
    }
}

function generateCells() {
    console.log("Generating cells...");
    const database = new Database();

    for (let i = 1; i < 7; i++) {
        const cell = new Cell();
        cell.draw((x, y) => {
            if (x === 0 || y === 0 || x === 7 || y === 7) {
                return 1;
            }
            return 0;
        });
        cell.set(i, 0, 0);
        database.addPermutations(cell);
    }

    for (let i = 1; i < 7; i++) {
        const cell = new Cell();
        cell.draw((x, y) => {
            if (x === 0 || y === 0 || x === 7 || y === 7) {
                return 1;
            }
            return 0;
        });
        cell.set(i, 0, 0);
        cell.set(i, 7, 0);
        database.addPermutations(cell);
    }
    for (let i = 1; i < 7; i++) {
        const cell = new Cell();
        cell.draw((x, y) => {
            if (x === 0 || y === 0 || x === 7 || y === 7) {
                return 1;
            }
            return 0;
        });
        cell.set(i, 0, 0);
        cell.set(0, i, 0);
        database.addPermutations(cell);
    }

    for (let i = 1; i < 7; i++) {
        const cell = new Cell();
        cell.draw((x, y) => {
            if (y === i) {
                return 0;
            }
            return 1;
        });
        database.addPermutations(cell);
    }

    database.addPermutations(Cell.from([
        "####.###",
        "#......#",
        "#.####.#",
        ".....#.#",
        "#....#.#",
        "#.####.#",
        "#......#",
        "########",
    ]));
    database.addPermutations(Cell.from([
        "########",
        "########",
        "#..##..#",
        "....#...",
        "#.#....#",
        "#.####.#",
        "#......#",
        "########",
    ]));
    database.addPermutations(Cell.from([
        "###.####",
        "###.####",
        "###...##",
        "#####.##",
        "#####.##",
        "#####.##",
        "......##",
        "########",
    ]));

    database.addPermutations(Cell.from([
        "###.####",
        "###.####",
        "........",
        "........",
        "#####.##",
        "#####.##",
        "#.....##",
        "########",
    ]));

    return database;
}

class DungeonMap {
    grid = new Map<string, Cell>();

    bounds(): {
        minX: number;
        maxX: number;
        minY: number;
        maxY: number;
    } {
        let minX = Infinity;
        let maxX = -Infinity;
        let minY = Infinity;
        let maxY = -Infinity;

        for (const key of this.grid.keys()) {
            const [xStr, yStr] = key.split(",");
            const x = parseInt(xStr, 10);
            const y = parseInt(yStr, 10);
            if (x < minX) minX = x;
            if (x > maxX) maxX = x;
            if (y < minY) minY = y;
            if (y > maxY) maxY = y;
        }
        return { minX, maxX, minY, maxY };
    }

    set(x: number, y: number, cell: Cell) {
        if (this.get(x, y)) {
            throw new Error(`Cell already exists at (${x}, ${y})`);
        }
        this.grid.set(`${x},${y}`, cell);
    }
    get(x: number, y: number): Cell | undefined {
        return this.grid.get(`${x},${y}`);
    }

    data(): (Cell | undefined)[][] {
        const bounds = this.bounds();
        const data: (Cell | undefined)[][] = [];
        for (let y = bounds.minY; y <= bounds.maxY; y++) {
            const row: (Cell | undefined)[] = [];
            for (let x = bounds.minX; x <= bounds.maxX; x++) {
                row.push(this.get(x, y));
            }
            data.push(row);
        }
        return data;
    }

    openDirections(p: Pos): direction[] {
        const cell = this.get(p.x, p.y);
        if (!cell) {
            return [];
        }
        const exits = cell.exits();
        const directions: direction[] = [];
        for (const exit of exits) {
            if (exit.x === 0 && !this.get(p.x - 1, p.y)) {
                directions.push("X-");
            } else if (exit.x === 7 && !this.get(p.x + 1, p.y)) {
                directions.push("X+");
            } else if (exit.y === 0 && !this.get(p.x, p.y - 1)) {
                directions.push("Y-");
            } else if (exit.y === 7 && !this.get(p.x, p.y + 1)) {
                directions.push("Y+");
            }
        }
        return directions;
    }

    fits(cell: Cell, p: Pos): boolean {
        const exits = cell.exits();

        for (let ny = -1; ny <= 1; ny++) {
            for (let nx = -1; nx <= 1; nx++) {
                if (nx === 0 && ny === 0) {
                    continue;
                }
                if (nx !== 0 && ny !== 0) {
                    continue;
                }
                const n = this.get(p.x + nx, p.y + ny);

                // If there's no neighbor in that direction, we
                // don't have to worry about exits aligning.
                if (!n) {
                    continue;
                }

                if (nx === -1) {
                    const entrances = n.exits().filter((exit) => exit.x === 7);
                    for (const door of entrances) {
                        if (cell.get(0, door.y) !== 0) {
                            return false;
                        }
                    }
                    for (const exit of exits) {
                        if (exit.x === 0) {
                            if (n.get(7, exit.y) !== 0) {
                                return false;
                            }
                        }
                    }
                } else if (nx === 1) {
                    const entrances = n.exits().filter((exit) => exit.x === 0);
                    for (const door of entrances) {
                        if (cell.get(7, door.y) !== 0) {
                            return false;
                        }
                    }
                    for (const exit of exits) {
                        if (exit.x === 7) {
                            if (n.get(0, exit.y) !== 0) {
                                return false;
                            }
                        }
                    }
                } else if (ny === -1) {
                    const entrances = n.exits().filter((exit) => exit.y === 7);
                    for (const door of entrances) {
                        if (cell.get(door.x, 0) !== 0) {
                            return false;
                        }
                    }
                    for (const exit of exits) {
                        if (exit.y === 0) {
                            if (n.get(exit.x, 7) !== 0) {
                                return false;
                            }
                        }
                    }
                } else if (ny === 1) {
                    const entrances = n.exits().filter((exit) => exit.y === 0);
                    for (const door of entrances) {
                        if (cell.get(door.x, 7) !== 0) {
                            return false;
                        }
                    }
                    for (const exit of exits) {
                        if (exit.y === 7) {
                            if (n.get(exit.x, 0) !== 0) {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        return true;
    }

    buildFittingCell(p: Pos): Cell {
        const cell = new Cell();
        cell.draw((x, y) => 1);
        return cell;
    }
}

function selectRandom<T>(array: T[]): T {
    const index = Math.floor(Math.random() * array.length);
    return array[index];
}

function build() {
    const database = generateCells();

    const map = new DungeonMap();

    const startLoc = { x: 0, y: 0 };
    const startCell = database.selectRandom().clone();
    map.set(startLoc.x, startLoc.y, startCell);

    let loc = startLoc;
    for (let i = 0; i < 64; i++) {
        console.log("STEP", i);
        const directions = map.openDirections(loc);
        if (directions.length === 0) {
            console.log("No open directions, stopping", i);
            break;
        }
        const dir = selectRandom(directions);
        let nextCell = database.select((next) => {
            if (next.exits().length <= 1) {
                return false;
            }
            return map.fits(next, addPos(loc, dir));
        })?.clone();
        if (!nextCell) {
            nextCell = map.buildFittingCell(addPos(loc, dir));
        }
        if (!nextCell) {
            console.log(`No fitting cell found, stopping (${dir}), ${i}`);
            break;
        }
        console.log(`Selected cell (${dir}):\n` + nextCell.display());

        const nextLoc = addPos(loc, dir);
        map.set(nextLoc.x, nextLoc.y, nextCell);
        loc = nextLoc;
    }

    for (const key of map.grid.keys()) {
        const pos = key.split(",").map((v) => parseInt(v, 10));
        const directions = map.openDirections({ x: pos[0], y: pos[1] });
        if (directions.length === 0) {
            continue;
        }
        for (const dir of directions) {
            const nextLoc = addPos({ x: pos[0], y: pos[1] }, dir);
            const cell = map.buildFittingCell(nextLoc);
            map.set(nextLoc.x, nextLoc.y, cell);
        }
    }

    return map;
}

export function AppView(): JSX.Element {
    const map = React.useMemo(() => {
        return build();
    }, []);

    const data = map.data();

    return (
        <div style={{ margin: 16 }}>
            {data.map((row, i) => (
                <div key={i} style={{ display: "flex", flexDirection: "row" }}>
                    {row.map((cell, j) => (
                        <div key={j}>
                            <CellView cell={cell} />
                        </div>
                    ))}
                </div>
            ))}
        </div>
    );
}

function CellView({ cell }: { cell: Cell | undefined }): JSX.Element {
    const bg = cell ? "#eee" : "#555";
    cell = cell || new Cell();
    const data = cell.data();

    return (
        <div>
            {data.map((row, rowIndex) => (
                <div key={rowIndex} style={{ display: "flex", flexDirection: "row" }}>
                    {row.map((value, colIndex) => (
                        <div
                            key={colIndex}
                            style={{
                                width: 8,
                                height: 8,
                                backgroundColor: value ? "#000" : bg,
                                border: "1px solid #555",
                            }}
                        />
                    ))}
                </div>
            ))}
        </div>
    );
}
