import { randomIntegerBetween, randomSeeded } from "@std/random";

function simulateDiceRolls(seed: bigint, rolls: number) {
    const prng = randomSeeded(seed);
    const results: number[] = [];
    for (let i = 0; i < rolls; i++) {
        results.push(randomIntegerBetween(1, 6, { prng }));
    }
    return results;
}

type TimedBasedRNG = "day" | "hour" | "minute";

export class RNG {
    //-------------------------------------------------------------------------
    // Object
    //-------------------------------------------------------------------------

    static _seedCounter = 0;
    static _globalRNG: RNG | null = null;

    _seed: number;
    _rng: ReturnType<typeof randomSeeded>;

    constructor(seed: number) {
        this._seed = seed;
        const i = Math.floor(seed);
        const f = seed - i;
        const bi = BigInt(i + Math.floor(f * 1e8));
        this._rng = randomSeeded(bi);
    }

    clone(): RNG {
        return new RNG(this._seed);
    }

    fork(): RNG {
        const newSeed = this._rng() * 1e15;
        return new RNG(newSeed);
    }

    //-------------------------------------------------------------------------
    // Static methods
    //-------------------------------------------------------------------------

    static get global(): RNG {
        if (RNG._globalRNG === null) {
            RNG._globalRNG = RNG.makeRandom();
        }
        return RNG._globalRNG;
    }

    static makeRandom(): RNG {
        RNG._seedCounter += 17;
        const seed = Math.random() * 1e15 +
            Date.now() % 1e5 +
            globalThis.performance.now() % 1e4 +
            RNG._seedCounter;
        return new RNG(seed);
    }

    /**
     * Returns a seed based on the local time truncated to the specified granularity.
     *
     * This can be useful for returning deterministically random values that change
     * every day, every hour, or every minute.
     */
    static makeTimeBased(granularity: TimedBasedRNG): RNG {
        const now = Date.now();
        const offset = new Date().getTimezoneOffset() * 60_000;
        const localNow = now - offset;

        let seed: number;
        switch (granularity) {
            case "day":
                seed = Math.floor(localNow / 86_400_000);
                break;
            case "hour":
                seed = Math.floor(localNow / 3_600_000);
                break;
            case "minute":
                seed = Math.floor(localNow / 60_000);
                break;
        }
        return new RNG(seed);
    }

    //-------------------------------------------------------------------------
    // Properties
    //-------------------------------------------------------------------------

    get seed(): number {
        return this._seed;
    }

    //-------------------------------------------------------------------------
    // Values
    //-------------------------------------------------------------------------

    value(): number {
        return this._rng();
    }

    bool(): boolean {
        return this.value() < 0.5;
    }

    sign(): -1 | 1 {
        return this.value() < 0.5 ? 1 : -1;
    }

    /**
     * Returns a random integer in the range [min, max).
     *
     * The max is exclusive.
     */
    rangei(min: number, max: number): number {
        return Math.floor(this.value() * (max - min)) + min;
    }

    rangef(min: number, max: number): number {
        return this.value() * (max - min) + min;
    }

    //-------------------------------------------------------------------------
    // Dice
    //-------------------------------------------------------------------------

    d(count: number, sides: number): number {
        let total = 0;
        for (let i = 0; i < count; i++) {
            total += this.rangei(1, sides + 1);
        }
        return total;
    }

    d2(): number {
        return this.rangei(1, 3);
    }
    d4(): number {
        return this.rangei(1, 5);
    }
    d6(): number {
        return this.rangei(1, 7);
    }
    d8(): number {
        return this.rangei(1, 9);
    }
    d10(): number {
        return this.rangei(1, 11);
    }
    d12(): number {
        return this.rangei(1, 13);
    }
    d20(): number {
        return this.rangei(1, 21);
    }
    d100(): number {
        return this.rangei(1, 101);
    }

    //-------------------------------------------------------------------------
    // Collections
    //-------------------------------------------------------------------------

    select<T>(
        arr: T[],
        weightKey?: keyof T | ((item: T) => number),
    ): T {
        return arr[this.selectIndex(arr, weightKey)];
    }

    /**
     * Returns N unique selections from the array.
     */
    selectN<T>(
        arr: T[],
        count: number,
        weightKey?: keyof T | ((item: T) => number),
    ): T[] {
        const rest = arr.slice();
        const copy: T[] = [];
        while (rest.length > 0 && count > 0) {
            const index = this.selectIndex(rest, weightKey);
            const el = rest[index];
            rest[index] = rest.pop()!;
            copy.push(el);
            count--;
        }
        return copy;
    }

    selectIndex<T>(
        arr: T[],
        weightKey?: keyof T | ((item: T) => number),
    ): number {
        // Simple case: unweighted selection
        if (weightKey === undefined) {
            return this.rangei(0, arr.length);
        }

        const weightFn = (typeof weightKey === "string")
            ? (item: T) => item[weightKey as keyof T] as number
            : (weightKey as (item: T) => number);

        const total = arr.reduce((acc, item) => acc + weightFn(item), 0);
        let roll = this.rangei(0, total);
        let index = 0;
        for (const item of arr) {
            roll -= weightFn(item);
            if (roll < 0) {
                return index;
            }
            index++;
        }
        throw new Error("RNG.select failed to select an item");
    }

    /**
     * Returns a new array with the items shuffled randomly.
     */
    shuffle<T>(
        arr: T[],
        weightKey?: keyof T | ((item: T) => number),
    ): T[] {
        return this.selectN(arr, arr.length, weightKey);
    }

    /**
     * Similar to select but removes the selected item from the array.
     */
    pluck<T>(
        arr: T[],
        weightKey?: keyof T | ((item: T) => number),
    ): T {
        const index = this.selectIndex(arr, weightKey);
        return arr.splice(index, 1)[0];
    }

    //-------------------------------------------------------------------------
    // Strings
    //-------------------------------------------------------------------------

    uuid(): string {
        const chars = "0123456789abcdef".split("");
        const next = (): string => {
            const i = this.rangei(0, chars.length);
            return chars[i];
        };
        const tokens = [
            next(),
            next(),
            next(),
            next(),
            "-",
            next(),
            next(),
            next(),
            next(),
            "-",
            next(),
            next(),
            next(),
            next(),
            "-",
            next(),
            next(),
            next(),
            next(),
            "-",
            next(),
            next(),
            next(),
            next(),
            next(),
            next(),
            next(),
            next(),
        ];
        return tokens.join("");
    }

    /**
     * Generates a string ID that needs to generate ~1 trillion unique IDs
     * before there is a 1% chance of collision.
     *
     * Reference: https://zelark.github.io/nano-id-cc/
     */
    shortID(): string {
        // Do not include vowels in the character set to avoid incidental
        // generation of words which might be considered offensive
        const chars = "0123456789bcdfghjklmnpqrstvwxyzBCDFGHJKLMNPQRSTVWXYZ_-"
            .split("");
        const tokens = new Array(16);
        for (let i = 0; i < tokens.length; i++) {
            tokens[i] = this.select(chars);
        }
        return tokens.join("");
    }
}
