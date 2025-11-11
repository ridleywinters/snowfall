export type ValueType =
    | "undefined"
    | "null"
    | "boolean"
    | "number"
    | "bigint"
    | "string"
    | "array"
    | "object"
    | "set"
    | "map"
    | "weakmap"
    | "symbol"
    | "date"
    | "promise"
    | "error"
    | "function";

/**
 * Extended version of the typeof operator.
 */
export function typeOfValue(value: unknown): ValueType {
    if (value === null) {
        return "null";
    }
    if (Array.isArray(value)) {
        return "array";
    }
    if (value instanceof Set) {
        return "set";
    }
    if (value instanceof Map) {
        return "map";
    }
    if (value instanceof WeakMap) {
        return "weakmap";
    }
    if (value instanceof Date) {
        return "date";
    }
    if (value instanceof Promise) {
        return "promise";
    }
    if (value instanceof Error) {
        return "error";
    }

    const t = typeof value;
    switch (t) {
        case "object":
        case "string":
        case "number":
        case "boolean":
        case "undefined":
        case "function":
        case "symbol":
        case "bigint":
            return t;
        default:
            throw new Error(`Unknown type: ${t}`);
    }
}
