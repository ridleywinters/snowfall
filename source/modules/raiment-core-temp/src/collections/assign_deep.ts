import { typeOfValue } from "../base/type_of_value.ts";

export function assignDeep<T>(target: T, ...partials: Partial<T>[]): T {
    for (const partial of partials) {
        target = assignDeep1(target, partial);
    }
    return target;
}

function assignDeep1<T>(target: T, partial: Partial<T>): T {
    const typeT = typeOfValue(target);

    switch (typeT) {
        case "undefined":
        case "null":
        case "number":
        case "bigint":
        case "boolean":
        case "string":
        case "symbol":
        case "date":
        case "function":
        case "promise":
        case "error":
            return partial as T;
    }

    const typeP = typeOfValue(partial);

    // This highlights why "deep merging" of values is not a standard operation
    // as there are a number of ways to interpret what correct behavior should
    // be.
    //
    // Design decisions made here:
    //
    // - Concatenate arrays or add unique elements? Unique.
    // - Uniqueness based on deep equality or reference equality? Reference.
    // - Merge object elements? No.
    //
    if (typeT === "array" && typeP === "array") {
        const setT = new Set(target as Array<unknown>);
        const setP = new Set(partial as unknown as Array<unknown>);
        const merged = new Set([...setT, ...setP]);
        return Array.from(merged) as T;
    }

    if (typeT === "object" && typeP === "object") {
        const objT = target as Record<string, unknown>;
        const objP = partial as Record<string, unknown>;
        for (const key of Object.keys(objP)) {
            objT[key] = assignDeep1(objT[key], objP[key] as object);
        }
        return objT as T;
    }

    return target;
}
