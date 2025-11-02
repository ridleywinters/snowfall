import React from "react";

/**
 * Triggers a callback on receiving a server-side event.
 *
 * Assumes all server-side events will be sent as valid, parseable JSON.
 *
 * Uses a reconnect strategy with a backoff and eventual failure after too many
 * retries.
 */
export function useServerSideEvents(url: string, cb: (data: any) => void): void {
    React.useEffect(() => {
        let failures = 0;
        let connected = false;
        let reconnect = () => {};
        let eventSource: EventSource | null = null;

        const connect = () => {
            // Connect to the SSE endpoint
            eventSource = new EventSource(url);
            eventSource.onopen = () => {
                failures = 0;
                connected = true;
            };
            eventSource.onmessage = (event) => {
                try {
                    const json = JSON.parse(event.data);
                    cb(json);
                } catch {
                    console.warn(
                        "Received non-JSON SSE event:",
                        event.data,
                    );
                }
            };
            eventSource.onerror = () => {
                console.error("SSE connection error");
                eventSource?.close();

                // We do expect errors, for example if the server is restarting, but
                // we give up if we're failed too many times to connect.
                if (!connected && failures >= 10) {
                    return;
                }
                reconnect();
            };
        };
        reconnect = () => {
            failures++;
            setTimeout(() => {
                connect();
            }, 200 + failures * (100 + Math.random() * 100));
        };
        connect();

        return () => {
            eventSource?.close();
        };
    }, [url]);
}
