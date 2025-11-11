export async function fetchExists(url: string): Promise<boolean> {
    const r = await fetch(url, { method: "HEAD" });
    return r.ok;
}
