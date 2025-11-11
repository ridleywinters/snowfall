export class EventEmitter<T extends Record<string, any>> {
    handlers = new Map<keyof T, Set<(...args: any[]) => void>>();

    on<K extends keyof T>(event: K, fn: (...args: T[K]) => void): this {
        const set = this.handlers.get(event) ?? new Set();
        set.add(fn as (...args: any[]) => void);
        this.handlers.set(event, set);
        return this;
    }

    off<K extends keyof T>(event: K, fn: (...args: T[K]) => void): this {
        this.handlers.get(event)?.delete(fn as (...args: any[]) => void);
        return this;
    }

    toggle<K extends keyof T>(enable: boolean, event: K, fn: (...args: T[K]) => void): this {
        if (enable) {
            return this.on(event, fn);
        } else {
            return this.off(event, fn);
        }
    }

    once<K extends keyof T>(event: K, fn: (...args: T[K]) => void): this {
        const wrapped = (...args: T[K]) => {
            this.off(event, wrapped);
            fn(...args);
        };
        return this.on(event, wrapped);
    }

    emit<K extends keyof T>(event: K, ...args: T[K]): boolean {
        const set = this.handlers.get(event);
        if (!set || set.size === 0) {
            return false;
        }
        for (const fn of Array.from(set)) {
            (fn as (...a: T[K]) => void)(...args);
        }
        return true;
    }
}
