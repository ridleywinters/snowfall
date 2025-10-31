import { RNG } from "@raiment-core";
import React, { JSX } from "react";

type Direction = "Y+" | "Y-" | "X+" | "X-";

function addPos(p: Pos, dir: Direction): Pos {
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

const CELL_SIZE = 16;

class Cell {
    map = new Array(CELL_SIZE * CELL_SIZE).fill(0);

    clone(): Cell {
        const cell = new Cell();
        cell.map = [...this.map];
        return cell;
    }

    hash(): string {
        return this.map.join("");
    }

    set(x: number, y: number, value: number) {
        this.map[y * CELL_SIZE + x] = value;
    }

    get(x: number, y: number): number {
        return this.map[y * CELL_SIZE + x];
    }

    data(): number[][] {
        const data: number[][] = [];
        for (let y = 0; y < CELL_SIZE; y++) {
            const row: number[] = [];
            for (let x = 0; x < CELL_SIZE; x++) {
                row.push(this.map[y * CELL_SIZE + x]);
            }
            data.push(row);
        }
        return data;
    }

    exits(): { x: number; y: number }[] {
        const exits: { x: number; y: number }[] = [];
        for (let y = 0; y < CELL_SIZE; y++) {
            for (let x = 0; x < CELL_SIZE; x++) {
                if (this.get(x, y) === 0) {
                    if (x === 0 || y === 0 || x === CELL_SIZE - 1 || y === CELL_SIZE - 1) {
                        exits.push({ x, y });
                    }
                }
            }
        }
        return exits;
    }

    alignsToExits(that: Cell, dir: Direction): boolean {
        const exits = that.exits().filter((exit) => {
            return (dir === "Y+" && exit.y === CELL_SIZE - 1) ||
                (dir === "Y-" && exit.y === 0) ||
                (dir === "X+" && exit.x === CELL_SIZE - 1) ||
                (dir === "X-" && exit.x === 0);
        });
        for (const exit of exits) {
            const entrance = dir == "Y+"
                ? { x: exit.x, y: 0 }
                : dir == "Y-"
                ? { x: exit.x, y: CELL_SIZE - 1 }
                : dir == "X+"
                ? { x: 0, y: exit.y }
                : { x: CELL_SIZE - 1, y: exit.y };

            if (this.get(entrance.x, entrance.y) !== 0) {
                return false;
            }
        }
        return true;
    }

    draw(fn: (x: number, y: number) => number) {
        for (let y = 0; y < CELL_SIZE; y++) {
            for (let x = 0; x < CELL_SIZE; x++) {
                this.map[y * CELL_SIZE + x] = fn(x, y);
            }
        }
    }

    mirrorX(): Cell {
        const map = new Array(CELL_SIZE * CELL_SIZE).fill(0);
        for (let y = 0; y < CELL_SIZE; y++) {
            for (let x = 0; x < CELL_SIZE; x++) {
                map[y * CELL_SIZE + x] = this.map[y * CELL_SIZE + (CELL_SIZE - 1 - x)];
            }
        }
        this.map = map;
        return this;
    }

    mirrorY(): Cell {
        const map = new Array(CELL_SIZE * CELL_SIZE).fill(0);
        for (let y = 0; y < CELL_SIZE; y++) {
            for (let x = 0; x < CELL_SIZE; x++) {
                map[y * CELL_SIZE + x] = this.map[(CELL_SIZE - 1 - y) * CELL_SIZE + x];
            }
        }
        this.map = map;
        return this;
    }
    rotate90(n: number): Cell {
        n = n % 4;
        let map = this.map;
        for (let i = 0; i < n; i++) {
            const newMap = new Array(CELL_SIZE * CELL_SIZE).fill(0);
            for (let y = 0; y < CELL_SIZE; y++) {
                for (let x = 0; x < CELL_SIZE; x++) {
                    newMap[x * CELL_SIZE + (CELL_SIZE - 1 - y)] = map[y * CELL_SIZE + x];
                }
            }
            map = newMap;
        }
        this.map = map;
        return this;
    }

    display() {
        let text = "";
        for (let y = 0; y < CELL_SIZE; y++) {
            let row = "";
            for (let x = 0; x < CELL_SIZE; x++) {
                row += this.map[y * CELL_SIZE + x] ? "#" : ".";
            }
            text += row + "\n";
        }
        return text;
    }
}

class Database {
    cells: Record<string, Cell> = {};
    rng = RNG.makeRandom();

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
        const key = this.rng.select(keys);
        return this.cells[key];
    }

    select(fn: (cell: Cell) => boolean): Cell | null {
        const orderedKeys = Object.keys(this.cells);
        const keys = [];
        while (orderedKeys.length > 0) {
            const index = this.rng.select(orderedKeys.map((_, i) => i));
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

async function loadImage(src: string): Promise<HTMLImageElement> {
    return new Promise((resolve, reject) => {
        const img = new Image();
        img.onload = () => resolve(img);
        img.onerror = (err) => reject(err);
        img.src = src;
    });
}

async function extractCellData(image: HTMLImageElement): Promise<Cell[]> {
    const canvas = document.createElement("canvas");
    canvas.width = image.width;
    canvas.height = image.height;
    const ctx = canvas.getContext("2d");
    if (!ctx) {
        throw new Error("Failed to get canvas context");
    }
    ctx.drawImage(image, 0, 0);
    const cellData: number[][] = [];
    const cellsX = Math.floor(image.width / CELL_SIZE);
    const cellsY = Math.floor(image.height / CELL_SIZE);

    const cells = new Array<Cell>();
    for (let cy = 0; cy < cellsY; cy++) {
        for (let cx = 0; cx < cellsX; cx++) {
            const cell = new Cell();
            let transparent = true;
            for (let y = 0; y < CELL_SIZE; y++) {
                for (let x = 0; x < CELL_SIZE; x++) {
                    const pixelData = ctx.getImageData(
                        cx * CELL_SIZE + x,
                        cy * CELL_SIZE + y,
                        1,
                        1,
                    ).data;
                    if (pixelData[3] !== 0) {
                        transparent = false;
                    }
                    // Assuming black pixels are walls
                    const isWall = pixelData[0] === 0 && pixelData[1] === 0 && pixelData[2] === 0;
                    cell.set(x, y, isWall ? 1 : 0);
                }
            }
            if (!transparent) {
                cells.push(cell);
            }
        }
    }

    return cells;
}

async function generateCellsFromPNG(database: Database): Promise<void> {
    console.log("Generating cells from PNG...");

    // Load the PNG image and extract the cell data
    const image = await loadImage("/tiles-map-16x16.png");
    const cells = await extractCellData(image);

    // Create cells from the extracted data
    for (const cell of cells) {
        database.addPermutations(cell);
    }
}

async function generateCells() {
    console.log("Generating cells...");
    const database = new Database();
    await generateCellsFromPNG(database);
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
            if (x < minX) {
                minX = x;
            }
            if (x > maxX) {
                maxX = x;
            }
            if (y < minY) {
                minY = y;
            }
            if (y > maxY) {
                maxY = y;
            }
        }
        return { minX, maxX, minY, maxY };
    }

    keyPos(key: string): Pos {
        const [xStr, yStr] = key.split(",");
        return { x: parseInt(xStr, 10), y: parseInt(yStr, 10) };
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

    openDirections(p: Pos): Direction[] {
        const cell = this.get(p.x, p.y);
        if (!cell) {
            return [];
        }
        const exits = cell.exits();
        const directions: Direction[] = [];
        for (const exit of exits) {
            if (exit.x === 0 && !this.get(p.x - 1, p.y)) {
                directions.push("X-");
            } else if (exit.x === CELL_SIZE - 1 && !this.get(p.x + 1, p.y)) {
                directions.push("X+");
            } else if (exit.y === 0 && !this.get(p.x, p.y - 1)) {
                directions.push("Y-");
            } else if (exit.y === CELL_SIZE - 1 && !this.get(p.x, p.y + 1)) {
                directions.push("Y+");
            }
        }
        return directions;
    }

    /**
     * Returns true if the given cell will align to the exits of all of the
     * existing cells.
     *
     * It automatically fits with any neighboring cells that are not yet
     * set.
     */
    fits(cell: Cell, p: Pos, options: {
        newExits: "never" | "always";
    } = {
        newExits: "always",
    }): boolean {
        const exits = cell.exits();

        let newExitCount = 0;
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
                //
                // Unless we have the noNewExits option set, in which case
                // we have to make sure there are no exits on that side.
                if (!n) {
                    let sideExits = [];
                    if (nx === -1) {
                        sideExits = exits.filter((exit) => exit.x === 0);
                    } else if (nx === 1) {
                        sideExits = exits.filter((exit) => exit.x === CELL_SIZE - 1);
                    } else if (ny === -1) {
                        sideExits = exits.filter((exit) => exit.y === 0);
                    } else if (ny === 1) {
                        sideExits = exits.filter((exit) => exit.y === CELL_SIZE - 1);
                    }
                    newExitCount += sideExits.length;
                    if (options.newExits === "never" && newExitCount > 0) {
                        return false;
                    }
                    continue;
                }

                if (nx === -1) {
                    const entrances = n.exits().filter((exit) => exit.x === CELL_SIZE - 1);
                    for (const door of entrances) {
                        if (cell.get(0, door.y) !== 0) {
                            return false;
                        }
                    }
                    for (const exit of exits) {
                        if (exit.x === 0) {
                            if (n.get(CELL_SIZE - 1, exit.y) !== 0) {
                                return false;
                            }
                        }
                    }
                } else if (nx === 1) {
                    const entrances = n.exits().filter((exit) => exit.x === 0);
                    for (const door of entrances) {
                        if (cell.get(CELL_SIZE - 1, door.y) !== 0) {
                            return false;
                        }
                    }
                    for (const exit of exits) {
                        if (exit.x === CELL_SIZE - 1) {
                            if (n.get(0, exit.y) !== 0) {
                                return false;
                            }
                        }
                    }
                } else if (ny === -1) {
                    const entrances = n.exits().filter((exit) => exit.y === CELL_SIZE - 1);
                    for (const door of entrances) {
                        if (cell.get(door.x, 0) !== 0) {
                            return false;
                        }
                    }
                    for (const exit of exits) {
                        if (exit.y === 0) {
                            if (n.get(exit.x, CELL_SIZE - 1) !== 0) {
                                return false;
                            }
                        }
                    }
                } else if (ny === 1) {
                    const entrances = n.exits().filter((exit) => exit.y === 0);
                    for (const door of entrances) {
                        if (cell.get(door.x, CELL_SIZE - 1) !== 0) {
                            return false;
                        }
                    }
                    for (const exit of exits) {
                        if (exit.y === CELL_SIZE - 1) {
                            if (n.get(exit.x, 0) !== 0) {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        if (options.newExits === "always" && newExitCount < 1) {
            return false;
        }
        return true;
    }

    buildFittingCell(p: Pos): Cell {
        const cell = new Cell();
        cell.draw((x, y) => 1);
        return cell;
    }
}

async function build() {
    const rng = RNG.makeRandom();
    const database = await generateCells();

    const map = new DungeonMap();

    let loc = { x: 0, y: 0 };
    const startCell = database.selectRandom().clone();
    map.set(loc.x, loc.y, startCell);

    for (let i = 0; i < 20; i++) {
        const directions = map.openDirections(loc);
        if (directions.length === 0) {
            console.log("No open directions, stopping", i);
            break;
        }
        const dir = rng.select(directions);
        const nextCell = database.select((next) => {
            return map.fits(next, addPos(loc, dir), { newExits: "always" });
        })?.clone();
        if (!nextCell) {
            console.log(`No fitting cell found, stopping (${dir}), ${i}`);
            break;
        }

        const nextLoc = addPos(loc, dir);
        map.set(nextLoc.x, nextLoc.y, nextCell);
        loc = nextLoc;
    }

    type FillSpots = {
        pos: Pos;
        direction: Direction;
    };

    for (let step = 0; step < 10; step++) {
        const openSpots: FillSpots[] = [];
        for (const key of map.grid.keys()) {
            const pos = map.keyPos(key);
            for (const dir of map.openDirections(pos)) {
                openSpots.push({ pos, direction: dir });
            }
        }
        console.log("Open cells:", openSpots.length);
        if (openSpots.length === 0) {
            break;
        }

        for (let i = 0; i < 1000 && openSpots.length > 0; i++) {
            const index = rng.selectIndex(openSpots);
            const { pos, direction } = openSpots[index];
            const nextPos = addPos(pos, direction);
            const nextCell = database.select((next) => {
                return map.fits(next, nextPos, { newExits: "never" });
            })?.clone();
            if (!nextCell) {
                continue;
            }
            if (!map.get(nextPos.x, nextPos.y)) {
                map.set(nextPos.x, nextPos.y, nextCell);
            }
            openSpots.splice(index, 1);
            i--;
        }
    }

    return map;
}

export function AppView(): JSX.Element {
    const [map, setMap] = React.useState<DungeonMap | null>(null);
    React.useEffect(() => {
        const buildMap = async () => {
            const newMap = await build();
            setMap(newMap);
        };
        buildMap();
    }, []);

    if (!map) {
        return <div>Generating...</div>;
    }

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
                                width: 6,
                                height: 6,
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
