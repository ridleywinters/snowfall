// A # prefixed hexidecimal color string with 3, 6, or 8 digits characters.
export type HexColor = `#${string}`;

export type RGBU8 = {
    r: number; // [0-255] in range
    g: number; // [0-255] in range
    b: number; // [0-255] in range
};

export type RGBAU8 = {
    r: number; // [0-255] in range
    g: number; // [0-255] in range
    b: number; // [0-255] in range
    a: number; // [0-255] in range
};

export type RGBF32 = {
    r: number; // [0-1] in range
    g: number; // [0-1] in range
    b: number; // [0-1] in range
};

export type RGBAF32 = {
    r: number; // [0-1] in range
    g: number; // [0-1] in range
    b: number; // [0-1] in range
    a: number; // [0-1] in range
};

export type HSLF32 = {
    h: number; // [0-1] in range
    s: number; // [0-1] in range
    l: number; // [0-1] in range
};

export type RGBAU8Array = [number, number, number, number];
