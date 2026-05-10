import type { AppMode } from "./types";

type AppStateSnapshot = {
  mode: AppMode;
};

type AppStateListener = (snapshot: AppStateSnapshot) => void;

export class AppStateStore {
  #mode: AppMode = "canvas";
  #listeners = new Set<AppStateListener>();

  get snapshot(): AppStateSnapshot {
    return {
      mode: this.#mode,
    };
  }

  setMode(mode: AppMode) {
    if (mode === this.#mode) {
      return;
    }

    this.#mode = mode;
    this.#emit();
  }

  subscribe(listener: AppStateListener) {
    this.#listeners.add(listener);
    listener(this.snapshot);

    return () => {
      this.#listeners.delete(listener);
    };
  }

  #emit() {
    const snapshot = this.snapshot;

    for (const listener of this.#listeners) {
      listener(snapshot);
    }
  }
}
