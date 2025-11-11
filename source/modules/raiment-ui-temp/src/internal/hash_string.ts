/**
 * Simple hash function from
 * https://stackoverflow.com/questions/7616461/generate-a-hash-from-string-in-javascript
 */
export function hashString(input: string): number {
    let hash = 0;
    for (let i = 0; i < input.length; i++) {
        const charCode = input.charCodeAt(i);
        hash = (hash << 5) - hash + charCode;
        hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash);
}
