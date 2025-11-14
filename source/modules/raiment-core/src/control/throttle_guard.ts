export class ThrottleGuard {
    delay: number;
    lastTime: number = 0;
    timeout: ReturnType<typeof setTimeout> | null = null;

    constructor(delay: number) {
        this.delay = delay;
    }

    run(fn: () => void): void {
        const now = Date.now();
        const delta = now - this.lastTime;

        if (this.timeout) {
            clearTimeout(this.timeout);
            this.timeout = null;
        }

        if (delta >= this.delay) {
            this.lastTime = now;
            fn();
            return;
        }
        this.timeout = setTimeout(() => {
            this.timeout = null;
            this.lastTime = Date.now();
            fn();
        }, this.delay - delta);
    }
}
