export async function fetchText(url: string): Promise<string> {
    const r = await fetch(url);
    if (!r.ok) {
        throw new Error(`HTTP ${r.status} for ${url}`);
    }
    return await r.text();
}
