/**
 * Returns a value clamped between min and max.
 *
 * On invalid values such as NaN, it will return min.
 */
export function clamp(value: number, min: number, max: number): number {
    if (!(value >= min)) {
        return min;
    }
    if (!(value <= max)) {
        return max;
    }
    return value;
}

export function clampi(value: number, min: number, max: number): number {
    return Math.floor(clamp(value, min, max));
}
