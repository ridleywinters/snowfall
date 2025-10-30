import { JSX, useCallback, useEffect, useRef, useState } from "react";
import { useLocalStorage } from "./use_local_storage.tsx";

export function AppView(): JSX.Element {
    const rowCount = 20;
    const [allRowColors, setAllRowColors] = useState<string[][]>([]);
    const [rowOrder, setRowOrder] = useState<number[]>(() =>
        Array.from({ length: rowCount }, (_, i) => i)
    );

    const updateRowColors = useCallback((rowIndex: number, colors: string[]) => {
        setAllRowColors((prev) => {
            const updated = [...prev];
            updated[rowIndex] = colors;
            return updated;
        });
    }, []);

    const moveRow = useCallback((fromIndex: number, toIndex: number) => {
        setRowOrder((prev) => {
            const updated = [...prev];
            const [movedItem] = updated.splice(fromIndex, 1);
            updated.splice(toIndex, 0, movedItem);
            return updated;
        });
    }, []);

    // Reorder colors based on rowOrder
    const orderedColors = rowOrder.map((originalIndex) => allRowColors[originalIndex]).filter(
        Boolean,
    );

    const exportGPL = useCallback(() => {
        // Flatten all colors into a single array
        const allColors = orderedColors.flat();

        // Build GIMP Palette format
        let gplContent = "GIMP Palette\n";
        gplContent += "#\n";

        allColors.forEach((hex, idx) => {
            const r = parseInt(hex.substring(1, 3), 16);
            const g = parseInt(hex.substring(3, 5), 16);
            const b = parseInt(hex.substring(5, 7), 16);
            // Format: R G B Name (3 characters wide for RGB values)
            gplContent += `${r.toString().padStart(3, " ")} ${g.toString().padStart(3, " ")} ${
                b.toString().padStart(3, " ")
            }\tColor ${idx}\n`;
        });

        const blob = new Blob([gplContent], { type: "text/plain" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = "palette.gpl";
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }, [orderedColors]);

    return (
        <div style={{ margin: 16 }}>
            <div style={{ display: "flex", alignItems: "center", gap: "16px" }}>
                <h1>Palette Editor</h1>
                <button
                    type="button"
                    onClick={exportGPL}
                    style={{
                        padding: "8px 16px",
                        fontSize: "14px",
                        cursor: "pointer",
                        backgroundColor: "#4CAF50",
                        color: "white",
                        border: "none",
                        borderRadius: "4px",
                    }}
                >
                    Export Palette
                </button>
            </div>
            <div
                style={{
                    display: "flex",
                    flexDirection: "row",
                    gap: "16px",
                }}
            >
                <div
                    style={{
                        display: "flex",
                        flexDirection: "column",
                        gap: "4px",
                        margin: "16px",
                    }}
                >
                    {rowOrder.map((originalIndex, displayIndex) => (
                        <ColorRow
                            key={originalIndex}
                            rowIndex={originalIndex}
                            displayIndex={displayIndex}
                            storageKey={`palette-row${originalIndex + 1}`}
                            onColorsChange={updateRowColors}
                            onMoveUp={displayIndex > 0
                                ? () => moveRow(displayIndex, displayIndex - 1)
                                : undefined}
                            onMoveDown={displayIndex < rowCount - 1
                                ? () => moveRow(displayIndex, displayIndex + 1)
                                : undefined}
                        />
                    ))}
                </div>
                <PaletteCanvas allColors={orderedColors} />
            </div>
        </div>
    );
}

function PaletteCanvas({ allColors }: { allColors: string[][] }): JSX.Element {
    const canvasRef = useRef<HTMLCanvasElement>(null);

    const blockSize = 32;
    const rowCount = allColors.length;
    const colCount = allColors[0]?.length || 0;
    const canvasWidth = colCount * blockSize;
    const canvasHeight = rowCount * blockSize;

    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas) return;

        const ctx = canvas.getContext("2d");
        if (!ctx) return;

        // Clear canvas
        ctx.clearRect(0, 0, canvasWidth, canvasHeight);

        allColors.forEach((row, rowIndex) => {
            row.forEach((color, colIndex) => {
                ctx.fillStyle = color;
                ctx.fillRect(
                    colIndex * blockSize,
                    rowIndex * blockSize,
                    blockSize,
                    blockSize,
                );
            });
        });
    }, [allColors, canvasWidth, canvasHeight]);

    return (
        <div style={{ margin: "16px" }}>
            <canvas
                ref={canvasRef}
                width={canvasWidth}
                height={canvasHeight}
                style={{
                    border: "1px solid #000000",
                    imageRendering: "pixelated",
                }}
            />
        </div>
    );
}

/**
 * Computes the color gradient for a row based on three key colors
 */
function computeRowColors(color1: string, color2: string, color3: string): string[] {
    const hsl1 = hexToHSL(color1);
    const hsl2 = hexToHSL(color2);
    const hsl3 = hexToHSL(color3);

    const colors: string[] = [];

    // Element 0: color1
    colors.push(color1);

    // Elements 1-2: blends between color1 and color2
    for (let i = 1; i <= 2; i++) {
        const a = i / 3;
        const h = Math.round(hsl1.h * (1 - a) + hsl2.h * a);
        const s = Math.round(hsl1.s * (1 - a) + hsl2.s * a);
        const l = Math.round(hsl1.l * (1 - a) + hsl2.l * a);
        colors.push(hslToHex(h, s, l));
    }

    // Element 3: color2
    colors.push(color2);

    // Elements 4-5: blends between color2 and color3
    for (let i = 1; i <= 2; i++) {
        const a = i / 3;
        const h = Math.round(hsl2.h * (1 - a) + hsl3.h * a);
        const s = Math.round(hsl2.s * (1 - a) + hsl3.s * a);
        const l = Math.round(hsl2.l * (1 - a) + hsl3.l * a);
        colors.push(hslToHex(h, s, l));
    }

    // Element 6: color3
    colors.push(color3);

    return colors;
}

function ColorRow({
    rowIndex,
    displayIndex,
    storageKey,
    onColorsChange,
    onMoveUp,
    onMoveDown,
}: {
    rowIndex: number;
    displayIndex: number;
    storageKey: string;
    onColorsChange: (rowIndex: number, colors: string[]) => void;
    onMoveUp?: () => void;
    onMoveDown?: () => void;
}): JSX.Element {
    const [color1, setColor1] = useLocalStorage(`${storageKey}-color1`, "#000000");
    const [color2, setColor2] = useLocalStorage(`${storageKey}-color2`, "#1072C0");
    const [color3, setColor3] = useLocalStorage(`${storageKey}-color3`, "#ffffff");

    // Compute colors whenever the input colors change
    const colors = computeRowColors(color1, color2, color3);

    // Notify parent of color changes
    useEffect(() => {
        onColorsChange(rowIndex, colors);
    }, [rowIndex, color1, color2, color3, onColorsChange]);

    return (
        <div
            style={{
                display: "flex",
                flexDirection: "row",
                gap: "8px",
                alignItems: "center",
            }}
        >
            <div style={{ display: "flex", flexDirection: "column", gap: "2px" }}>
                <button
                    type="button"
                    onClick={onMoveUp}
                    disabled={!onMoveUp}
                    style={{
                        width: "24px",
                        height: "18px",
                        padding: "0",
                        fontSize: "12px",
                        cursor: onMoveUp ? "pointer" : "not-allowed",
                        opacity: onMoveUp ? 1 : 0.3,
                    }}
                    title="Move up"
                >
                    ▲
                </button>
                <button
                    type="button"
                    onClick={onMoveDown}
                    disabled={!onMoveDown}
                    style={{
                        width: "24px",
                        height: "18px",
                        padding: "0",
                        fontSize: "12px",
                        cursor: onMoveDown ? "pointer" : "not-allowed",
                        opacity: onMoveDown ? 1 : 0.3,
                    }}
                    title="Move down"
                >
                    ▼
                </button>
            </div>
            <input
                type="color"
                value={color2}
                onChange={(e) => {
                    const inputHsl = hexToHSL(e.target.value);

                    // Color1: reduce lightness by 50%
                    const color1Hsl = {
                        ...inputHsl,
                        l: Math.min(Math.max(5, inputHsl.l - 50), 100),
                    };

                    const color3Hsl = {
                        ...inputHsl,
                        l: Math.min(Math.max(5, inputHsl.l + 50), 100),
                    };

                    setColor2(e.target.value);
                    setColor1(hslToHex(color1Hsl.h, color1Hsl.s, color1Hsl.l));
                    setColor3(hslToHex(color3Hsl.h, color3Hsl.s, color3Hsl.l));
                }}
            />
            <input type="color" value={color1} onChange={(e) => setColor1(e.target.value)} />
            <input type="color" value={color3} onChange={(e) => setColor3(e.target.value)} />
            {colors.map((color, idx) => (
                <div
                    key={idx}
                    style={{
                        width: "40px",
                        height: "40px",
                        backgroundColor: color,
                        border: "1px solid #000000",
                    }}
                />
            ))}
        </div>
    );
}

/**
 * Converts HSL values to a hex color string
 * @param h - Hue (0-360)
 * @param s - Saturation (0-100)
 * @param l - Lightness (0-100)
 * @returns Hex color string with # prefix
 */
function hslToHex(h: number, s: number, l: number): string {
    // Normalize values
    h = h / 360;
    s = s / 100;
    l = l / 100;

    const c = (1 - Math.abs(2 * l - 1)) * s;
    const x = c * (1 - Math.abs((h * 6) % 2 - 1));
    const m = l - c / 2;

    let r = 0, g = 0, b = 0;

    if (0 <= h && h < 1 / 6) {
        r = c;
        g = x;
        b = 0;
    } else if (1 / 6 <= h && h < 2 / 6) {
        r = x;
        g = c;
        b = 0;
    } else if (2 / 6 <= h && h < 3 / 6) {
        r = 0;
        g = c;
        b = x;
    } else if (3 / 6 <= h && h < 4 / 6) {
        r = 0;
        g = x;
        b = c;
    } else if (4 / 6 <= h && h < 5 / 6) {
        r = x;
        g = 0;
        b = c;
    } else if (5 / 6 <= h && h < 1) {
        r = c;
        g = 0;
        b = x;
    }

    // Convert to 0-255 range and round
    r = Math.round((r + m) * 255);
    g = Math.round((g + m) * 255);
    b = Math.round((b + m) * 255);

    // Convert to hex and pad with zeros if needed
    const toHex = (n: number) => n.toString(16).padStart(2, "0");

    return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
}

/**
 * Converts a hex color string to HSL values
 * @param hex - Hex color string (with or without #)
 * @returns Object with h (0-360), s (0-100), l (0-100) values
 */
function hexToHSL(hex: string): { h: number; s: number; l: number } {
    // Remove the hash if present
    hex = hex.replace("#", "");

    // Parse hex to RGB
    const r = parseInt(hex.substring(0, 2), 16) / 255;
    const g = parseInt(hex.substring(2, 4), 16) / 255;
    const b = parseInt(hex.substring(4, 6), 16) / 255;

    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    const diff = max - min;

    let h = 0;
    let s = 0;
    const l = (max + min) / 2;

    if (diff !== 0) {
        s = l > 0.5 ? diff / (2 - max - min) : diff / (max + min);

        switch (max) {
            case r:
                h = ((g - b) / diff + (g < b ? 6 : 0)) / 6;
                break;
            case g:
                h = ((b - r) / diff + 2) / 6;
                break;
            case b:
                h = ((r - g) / diff + 4) / 6;
                break;
        }
    }

    return {
        h: Math.round(h * 360),
        s: Math.round(s * 100),
        l: Math.round(l * 100),
    };
}
