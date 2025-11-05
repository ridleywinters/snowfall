import { ColorHexString, EventEmitter } from "@raiment-core";
import { hexToHSL, hslToHex } from "../../util/util.ts";

export class Palette {
    events = new EventEmitter<{ "update": [] }>();

    base: ColorHexString[];
    colors: {
        primary: ColorHexString;
        shade: ColorHexString;
        highlight: ColorHexString;
    }[];

    constructor() {
        this.base = [];
        this.colors = [];
    }

    static fromGIMPPalette(colors: ColorHexString[]): Palette {
        return convertGPLToPalette(colors);
    }

    getBase(index: number): ColorHexString {
        return this.base[index];
    }
    setBase(index: number, color: ColorHexString): void {
        this.base[index] = color;
        this.events.emit("update");
    }

    get(row: number, flavor: "primary" | "shade" | "highlight"): ColorHexString {
        const colorSet = this.colors[row];
        if (!colorSet) {
            throw new Error(`No color set for row ${row}`);
        }
        return colorSet[flavor];
    }
    set(row: number, flavor: "primary" | "shade" | "highlight", color: ColorHexString): void {
        const colorSet = this.colors[row];
        if (!colorSet) {
            throw new Error(`No color set for row ${row}`);
        }
        colorSet[flavor] = color;
        this.events.emit("update");
    }

    moveRow(index: number, direction: "up" | "down"): void {
        if (direction === "up" && index > 0) {
            const temp = this.colors[index - 1];
            this.colors[index - 1] = this.colors[index];
            this.colors[index] = temp;
        } else if (direction === "down" && index < this.colors.length - 1) {
            const temp = this.colors[index + 1];
            this.colors[index + 1] = this.colors[index];
            this.colors[index] = temp;
        }
        this.events.emit("update");
    }

    computeAll(): ColorHexString[] {
        const all: ColorHexString[] = [];
        all.push(...this.base);
        for (let i = 0; i < this.colors.length; i++) {
            all.push(...this.computeRow(i));
        }
        return all;
    }

    computeRow(i: number): ColorHexString[] {
        const primary = this.colors[i].primary;
        const shade = this.colors[i].shade;
        const highlight = this.colors[i].highlight;
        const hslPrimary = hexToHSL(primary as string);
        const hslShade = hexToHSL(shade as string);
        const hslHighlight = hexToHSL(highlight as string);

        const colors: ColorHexString[] = [];

        // Helper function to interpolate hue in the shortest direction
        const interpolateHue = (h1: number, h2: number, t: number): number => {
            let diff = h2 - h1;
            // Normalize difference to [-180, 180]
            if (diff > 180) {
                diff -= 360;
            } else if (diff < -180) {
                diff += 360;
            }
            // Interpolate and wrap around
            let result = h1 + diff * t;
            if (result < 0) {
                result += 360;
            } else if (result >= 360) {
                result -= 360;
            }
            return Math.round(result);
        };

        // Element 0: shade
        colors.push(shade);

        // Elements 1-2: blends between shade and primary
        for (let j = 1; j <= 2; j++) {
            const a = j / 3;
            const h = interpolateHue(hslShade.h, hslPrimary.h, a);
            const s = Math.round(hslShade.s * (1 - a) + hslPrimary.s * a);
            const l = Math.round(hslShade.l * (1 - a) + hslPrimary.l * a);
            colors.push(hslToHex(h, s, l) as ColorHexString);
        }

        // Element 3: primary
        colors.push(primary);

        // Elements 4-5: blends between primary and highlight
        for (let j = 1; j <= 2; j++) {
            const a = j / 3;
            const h = interpolateHue(hslPrimary.h, hslHighlight.h, a);
            const s = Math.round(hslPrimary.s * (1 - a) + hslHighlight.s * a);
            const l = Math.round(hslPrimary.l * (1 - a) + hslHighlight.l * a);
            colors.push(hslToHex(h, s, l) as ColorHexString);
        }

        // Element 6: highlight
        colors.push(highlight);

        return colors;
    }
}

function convertGPLToPalette(gplColors: ColorHexString[]): Palette {
    const palette = new Palette();

    for (let i = 0; i < 7 && i < gplColors.length; i++) {
        palette.base.push(gplColors[i]);
    }
    let rows = 0;
    for (let i = 7; i < gplColors.length && rows < 15; i += 7, rows += 1) {
        const chunk = gplColors.slice(i, i + 7);
        if (chunk.length < 7) {
            chunk.push(...Array(7 - chunk.length).fill("#000000"));
        }
        palette.colors.push({
            primary: chunk[3],
            shade: chunk[0],
            highlight: chunk[6],
        });
    }
    return palette;
}
