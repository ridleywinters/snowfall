import * as core from "@raiment-core";
import { type ColorHexString } from "@raiment-core";
import { Div, invokeDownload, useEventListener } from "@raiment-ui";
import React, { JSX } from "react";
import { Palette } from "./palette.ts";
import { serverAPI } from "@/util/server_api.tsx";
import { ToolAppFrame } from "@/components/tool_app_frame.tsx";

// Color conversion utilities
function hexToRgb(hex: ColorHexString): { r: number; g: number; b: number } {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    if (!result) {
        throw new Error(`Invalid hex color: ${hex}`);
    }
    return {
        r: parseInt(result[1], 16),
        g: parseInt(result[2], 16),
        b: parseInt(result[3], 16),
    };
}

/**
 * Converts RGB color values to HSL color space.
 * @returns Object with h, s, and l values all in the range [0, 1]
 */
function rgbToHsl(r: number, g: number, b: number): { h: number; s: number; l: number } {
    r /= 255;
    g /= 255;
    b /= 255;

    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    let h = 0;
    let s = 0;
    const l = (max + min) / 2;

    if (max !== min) {
        const d = max - min;
        s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

        switch (max) {
            case r:
                h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
                break;
            case g:
                h = ((b - r) / d + 2) / 6;
                break;
            case b:
                h = ((r - g) / d + 4) / 6;
                break;
        }
    }

    return { h, s, l };
}

function hslDistance(
    hsl1: { h: number; s: number; l: number },
    hsl2: { h: number; s: number; l: number },
): number {
    // Calculate hue distance (circular)
    const hueDiff = Math.min(Math.abs(hsl1.h - hsl2.h), 1 - Math.abs(hsl1.h - hsl2.h));
    const satDiff = Math.abs(hsl1.s - hsl2.s);
    const lightDiff = Math.abs(hsl1.l - hsl2.l);

    // Weight the components (hue is most important, then lightness, then saturation)
    return Math.sqrt(hueDiff * hueDiff * 2 + lightDiff * lightDiff + satDiff * satDiff * 0.5);
}

function findClosestColor(
    targetHex: ColorHexString,
    paletteColors: ColorHexString[],
): ColorHexString {
    const targetRgb = hexToRgb(targetHex);
    const targetHsl = rgbToHsl(targetRgb.r, targetRgb.g, targetRgb.b);

    let closestColor = paletteColors[0];
    let minDistance = Infinity;

    for (const paletteColor of paletteColors) {
        const paletteRgb = hexToRgb(paletteColor);
        const paletteHsl = rgbToHsl(paletteRgb.r, paletteRgb.g, paletteRgb.b);
        const distance = hslDistance(targetHsl, paletteHsl);

        if (distance < minDistance) {
            minDistance = distance;
            closestColor = paletteColor;
        }
    }

    return closestColor;
}

export function PaletteEditorApp(): JSX.Element {
    const [palette, setPalette] = React.useState<Palette | null>(null);

    React.useEffect(() => {
        const go = async () => {
            const gplContent = await serverAPI.readFile("palette/palette.gpl", "text") as string;
            const colors = core.parseGIMPPalette(gplContent);
            const pal = Palette.fromGIMPPalette(colors ?? []);
            setPalette(pal);
        };
        go();
    }, []);

    return (
        <ToolAppFrame>
            {!palette ? <Div sl="m32">Loading palette...</Div> : <AppView2 palette={palette} />}
        </ToolAppFrame>
    );
}

function AppView2({ palette }: { palette: Palette }): JSX.Element {
    useEventListener(palette.events, "update");

    const exportGPL = React.useCallback(() => {
        const colors = palette.computeAll();
        const gplContent = core.stringifyGIMPPalette(colors);
        invokeDownload("palette.gpl", gplContent, "text/plain");
    }, [palette]);

    const savePalette = React.useCallback(async () => {
        const colors = palette.computeAll();
        const gplContent = core.stringifyGIMPPalette(colors);
        await serverAPI.writeFile("palette/palette.gpl", gplContent, "text");
        alert("Palette saved successfully.");
    }, [palette]);

    return (
        <Div sl="mt32 mx64">
            <Div sl="flex-row-center gap-16">
                <h1>Palette Editor v0.1</h1>
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

                <button
                    type="button"
                    onClick={savePalette}
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
                    Save Palette
                </button>
            </Div>

            <Div sl="flex-row-center gap-32">
                <Div>
                    <Div sl="flex-row-center gap-8 mb-16">
                        {palette.base.map((_color, idx) => (
                            <ColorPicker
                                key={idx}
                                value={palette.getBase(idx)}
                                onChange={(newColor) => {
                                    palette.setBase(idx, newColor);
                                }}
                            />
                        ))}
                    </Div>
                    <Div sl="flex-column mb-16">
                        {palette.colors.map((colorSet, idx) => (
                            <PaletteRow key={idx} palette={palette} rowIndex={idx} />
                        ))}
                    </Div>
                </Div>
                <Div sl="flex-row gap-32">
                    <div>
                        <div style={{ height: 40 }} />
                        <PaletteCanvas palette={palette} />
                        <div
                            style={{
                                color: "#555",
                                fontStyle: "italic",
                                maxWidth: "240px",
                                textAlign: "center",
                            }}
                        >
                            Right-click to save palette as an image
                        </div>
                    </div>
                    <div>
                        <div style={{ height: 40 }} />
                        <ImageDropZone palette={palette} />
                    </div>
                </Div>
            </Div>
            <div>
                Color count: {palette.computeAll().length}
            </div>
        </Div>
    );
}

function PaletteCanvas({ palette }: { palette: Palette }): JSX.Element {
    const BLOCK_SIZE = 32;
    const COLUMNS = 7;

    const ref = React.useRef<HTMLCanvasElement | null>(null);
    const gen = useEventListener(palette.events, "update");

    React.useEffect(() => {
        const canvas = ref.current;
        if (!canvas) {
            return;
        }
        const ctx = canvas.getContext("2d");
        if (!ctx) {
            return;
        }

        const colors = palette.computeAll();
        const rows = Math.ceil(colors.length / 7);
        const width = 7 * BLOCK_SIZE;
        const height = rows * BLOCK_SIZE;

        canvas.width = width;
        canvas.height = height;
        ctx.clearRect(0, 0, width, height);
        for (let y = 0; y < rows; y++) {
            for (let x = 0; x < COLUMNS; x++) {
                const index = y * COLUMNS + x;
                if (index >= colors.length) {
                    break;
                }
                const color = colors[index];
                ctx.fillStyle = color;
                ctx.fillRect(x * BLOCK_SIZE, y * BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE);
            }
        }
    }, [gen]);

    return (
        <div
            style={{
                display: "flex",
                flexDirection: "column",
            }}
        >
            <canvas ref={ref} width={256} height={256} style={{ border: "1px solid #444" }} />
        </div>
    );
}

function PaletteRow({ palette, rowIndex }: { palette: Palette; rowIndex: number }): JSX.Element {
    return (
        <div
            style={{
                display: "flex",
                flexDirection: "row",
                gap: "8px",
                marginBottom: "4px",
            }}
        >
            <ColorPicker
                value={palette.get(rowIndex, "primary")}
                onChange={(newColor) => {
                    palette.set(rowIndex, "primary", newColor);
                }}
            />
            <div style={{ width: "12px" }} />
            <ColorPicker
                value={palette.get(rowIndex, "shade")}
                onChange={(newColor) => {
                    palette.set(rowIndex, "shade", newColor);
                }}
            />
            <ColorPicker
                value={palette.get(rowIndex, "highlight")}
                onChange={(newColor) => {
                    palette.set(rowIndex, "highlight", newColor);
                }}
            />
            <div style={{ width: "24px" }} />
            {palette.computeRow(rowIndex).map((color, cidx) => (
                <Div
                    key={cidx}
                    sl="width-32 height-32 cursor-pointer"
                    style={{
                        backgroundColor: color,
                        border: "1px solid #444",
                    }}
                    onClick={async () => {
                        try {
                            await navigator.clipboard.writeText(color);
                            console.log(`Copied ${color} to clipboard`);
                        } catch (err) {
                            console.error(
                                "Failed to copy color to clipboard:",
                                err,
                            );
                        }
                    }}
                />
            ))}
            <button
                type="button"
                onClick={() => palette.moveRow(rowIndex, "up")}
                style={{
                    padding: "4px 8px",
                    cursor: "pointer",
                    border: "none",
                    background: "transparent",
                    color: "#555",
                }}
            >
                ▲
            </button>
        </div>
    );
}

function ImageDropZone({ palette }: { palette: Palette }): JSX.Element {
    const [droppedImage, setDroppedImage] = React.useState<HTMLImageElement | null>(null);
    const [isDragging, setIsDragging] = React.useState(false);
    const originalCanvasRef = React.useRef<HTMLCanvasElement | null>(null);
    const mappedCanvasRef = React.useRef<HTMLCanvasElement | null>(null);
    const dropZoneRef = React.useRef<HTMLDivElement | null>(null);

    const gen = useEventListener(palette.events, "update");

    const handleCanvasClick = React.useCallback((evt: React.MouseEvent<HTMLCanvasElement>) => {
        const canvas = evt.currentTarget;
        const ctx = canvas.getContext("2d");
        if (!ctx) {
            return;
        }

        const rect = canvas.getBoundingClientRect();
        const x = Math.floor(evt.clientX - rect.left);
        const y = Math.floor(evt.clientY - rect.top);

        const pixelData = ctx.getImageData(x, y, 1, 1).data;
        const hexColor = `#${pixelData[0].toString(16).padStart(2, "0")}${
            pixelData[1].toString(16).padStart(2, "0")
        }${pixelData[2].toString(16).padStart(2, "0")}`;

        navigator.clipboard.writeText(hexColor).then(() => {
            console.log(`Copied ${hexColor} to clipboard`);
        }).catch((err) => {
            console.error("Failed to copy color to clipboard:", err);
        });
    }, []);

    React.useEffect(() => {
        const dropZone = dropZoneRef.current;
        if (!dropZone) {
            return;
        }

        const handleDragOver = (evt: DragEvent) => {
            evt.preventDefault();
            setIsDragging(true);
        };

        const handleDragLeave = (evt: DragEvent) => {
            evt.preventDefault();
            setIsDragging(false);
        };

        const handleDrop = (evt: DragEvent) => {
            evt.preventDefault();
            setIsDragging(false);

            const files = Array.from(evt.dataTransfer?.files ?? []);
            const imageFile = files.find((file) => file.type.startsWith("image/"));

            if (imageFile) {
                const reader = new FileReader();
                reader.onload = (e) => {
                    const img = new Image();
                    img.onload = () => {
                        setDroppedImage(img);
                    };
                    img.src = e.target?.result as string;
                };
                reader.readAsDataURL(imageFile);
            }
        };

        dropZone.addEventListener("dragover", handleDragOver);
        dropZone.addEventListener("dragleave", handleDragLeave);
        dropZone.addEventListener("drop", handleDrop);

        return () => {
            dropZone.removeEventListener("dragover", handleDragOver);
            dropZone.removeEventListener("dragleave", handleDragLeave);
            dropZone.removeEventListener("drop", handleDrop);
        };
    }, []);

    React.useEffect(() => {
        if (!droppedImage || !originalCanvasRef.current || !mappedCanvasRef.current) {
            return;
        }

        const paletteColors = palette.computeAll();
        const originalCanvas = originalCanvasRef.current;
        const mappedCanvas = mappedCanvasRef.current;
        const originalCtx = originalCanvas.getContext("2d");
        const mappedCtx = mappedCanvas.getContext("2d");

        if (!originalCtx || !mappedCtx) {
            return;
        }

        // Set canvas size to 8x the image size
        originalCanvas.width = droppedImage.width * 8;
        originalCanvas.height = droppedImage.height * 8;
        mappedCanvas.width = droppedImage.width * 8;
        mappedCanvas.height = droppedImage.height * 8;

        // Draw original image at 8x scale
        originalCtx.imageSmoothingEnabled = false;
        originalCtx.drawImage(droppedImage, 0, 0, droppedImage.width * 8, droppedImage.height * 8);

        // Create a temporary canvas at original size to read pixel data
        const tempCanvas = document.createElement("canvas");
        tempCanvas.width = droppedImage.width;
        tempCanvas.height = droppedImage.height;
        const tempCtx = tempCanvas.getContext("2d");
        if (!tempCtx) {
            return;
        }
        tempCtx.drawImage(droppedImage, 0, 0);

        const imageData = tempCtx.getImageData(0, 0, droppedImage.width, droppedImage.height);
        const pixels = imageData.data;

        // Create mapped image data
        const mappedImageData = mappedCtx.createImageData(droppedImage.width, droppedImage.height);
        const mappedPixels = mappedImageData.data;

        for (let i = 0; i < pixels.length; i += 4) {
            const r = pixels[i];
            const g = pixels[i + 1];
            const b = pixels[i + 2];
            const a = pixels[i + 3];

            // Convert to hex and find closest palette color
            const hexColor = `#${r.toString(16).padStart(2, "0")}${
                g.toString(16).padStart(2, "0")
            }${b.toString(16).padStart(2, "0")}` as ColorHexString;
            const closestColor = findClosestColor(hexColor, paletteColors);
            const closestRgb = hexToRgb(closestColor);

            mappedPixels[i] = closestRgb.r;
            mappedPixels[i + 1] = closestRgb.g;
            mappedPixels[i + 2] = closestRgb.b;
            mappedPixels[i + 3] = a;
        }

        // Draw mapped image at 2x scale
        const mappedTempCanvas = document.createElement("canvas");
        mappedTempCanvas.width = droppedImage.width;
        mappedTempCanvas.height = droppedImage.height;
        const mappedTempCtx = mappedTempCanvas.getContext("2d");
        if (!mappedTempCtx) {
            return;
        }
        mappedTempCtx.putImageData(mappedImageData, 0, 0);

        mappedCtx.imageSmoothingEnabled = false;
        mappedCtx.drawImage(
            mappedTempCanvas,
            0,
            0,
            droppedImage.width * 8,
            droppedImage.height * 8,
        );
    }, [droppedImage, gen]);

    return (
        <div
            style={{
                display: "flex",
                flexDirection: "column",
                gap: "16px",
            }}
        >
            <div
                ref={dropZoneRef}
                style={{
                    border: isDragging ? "2px dashed #4CAF50" : "2px dashed #666",
                    borderRadius: "8px",
                    padding: "16px",
                    minWidth: "300px",
                    minHeight: "40px",
                    backgroundColor: isDragging ? "rgba(76, 175, 80, 0.1)" : "transparent",
                    transition: "all 0.2s",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                }}
            >
                <div
                    style={{
                        color: "#888",
                        textAlign: "center",
                    }}
                >
                    Drop a PNG for palette mapping preview
                </div>
            </div>
            {droppedImage && (
                <Div sl="flex-column gap-16">
                    <div>
                        <div style={{ marginBottom: "8px", fontWeight: "bold", color: "#aaa" }}>
                            Original
                        </div>
                        <canvas
                            ref={originalCanvasRef}
                            onClick={handleCanvasClick}
                            style={{
                                border: "1px solid #444",
                                imageRendering: "pixelated",
                                cursor: "pointer",
                            }}
                        />
                    </div>
                    <div>
                        <div style={{ marginBottom: "8px", fontWeight: "bold", color: "#aaa" }}>
                            Palette Mapped
                        </div>
                        <canvas
                            ref={mappedCanvasRef}
                            onClick={handleCanvasClick}
                            style={{
                                border: "1px solid #444",
                                imageRendering: "pixelated",
                                cursor: "pointer",
                            }}
                        />
                    </div>
                </Div>
            )}
        </div>
    );
}

function ColorPicker({
    value,
    onChange,
}: {
    value: ColorHexString;
    onChange: (newColor: ColorHexString) => void;
}): JSX.Element {
    const timeoutRef = React.useRef<number | null>(null);
    const handleChange = React.useCallback((evt: React.ChangeEvent<HTMLInputElement>) => {
        const newColor = evt.target.value as ColorHexString;
        if (timeoutRef.current) {
            clearTimeout(timeoutRef.current);
        }
        timeoutRef.current = setTimeout(() => {
            onChange(newColor);
        }, 200);
    }, [onChange]);

    return (
        <Div>
            <input
                type="color"
                value={value}
                onChange={handleChange}
            />
        </Div>
    );
}
