import { invoke } from "@tauri-apps/api/core";

import { mountCanvasWorkspace } from "../features/canvas/workspace";
import { mountPdfWorkspace } from "../features/pdf/workspace";
import { AppStateStore } from "./state";
import type {
  AppBootstrap,
  AppMode,
  CanvasDocument,
  PdfStudyDocument,
  WorkspaceController,
  WorkspaceDocumentSnapshot,
} from "./types";

type AutosaveRecord = {
  savedAt: string;
  snapshot: WorkspaceDocumentSnapshot;
};

type AppPreferences = {
  defaultStrokeWidth: number;
  defaultShapeKind: "rectangle" | "ellipse" | "line";
  defaultCanvasColor: string;
  speechRate: number;
  recolorEnabled: boolean;
  recolorForeground: string;
  recolorBackground: string;
};

const PREFERENCES_STORAGE_KEY = "rpdf.preferences.v1";

function defaultPreferences(): AppPreferences {
  return {
    defaultStrokeWidth: 3,
    defaultShapeKind: "rectangle",
    defaultCanvasColor: "#c0caf5",
    speechRate: 1,
    recolorEnabled: false,
    recolorForeground: "#c0caf5",
    recolorBackground: "#1a1b26",
  };
}

function readPreferences() {
  const rawValue = window.localStorage.getItem(PREFERENCES_STORAGE_KEY);

  if (!rawValue) {
    return defaultPreferences();
  }

  try {
    return {
      ...defaultPreferences(),
      ...(JSON.parse(rawValue) as Partial<AppPreferences>),
    };
  } catch (error) {
    console.error("Could not parse app preferences:", error);
    return defaultPreferences();
  }
}

function writePreferences(preferences: AppPreferences) {
  window.localStorage.setItem(PREFERENCES_STORAGE_KEY, JSON.stringify(preferences));
  window.dispatchEvent(new CustomEvent("rpdf:preferences-changed", {
    detail: preferences,
  }));
}

export function mountAppShell(root: HTMLElement) {
  const state = new AppStateStore();

  root.innerHTML = `
    <div id="app-shell">
      <aside class="mode-rail">
        <div class="brand-block">
          <div class="brand-kicker">rpdf</div>
          <div class="brand-copy">study shell</div>
        </div>

        <button class="mode-button active" data-mode="canvas" type="button"><span class="button-icon" aria-hidden="true">✎</span><span>Canvas</span></button>
        <button class="mode-button" data-mode="pdf" type="button"><span class="button-icon" aria-hidden="true">📄</span><span>PDF</span></button>
      </aside>

      <main class="workspace-panel">
        <header class="workspace-header">
          <div>
            <div id="mode-title" class="mode-title">Canvas Mode</div>
            <div id="mode-copy" class="mode-copy">Spatial note-taking workspace</div>
          </div>

          <div class="header-actions">
            <label class="project-path-field">
              <span id="project-path-label">Canvas file path</span>
              <input id="project-path-input" type="text" placeholder="/tmp/canvas-project.rpdf.json" />
            </label>
            <div class="project-action-row">
              <button id="project-save-button" class="project-action-button" type="button"><span class="button-icon" aria-hidden="true">💾</span><span>Save</span></button>
              <button id="project-load-button" class="project-action-button secondary" type="button"><span class="button-icon" aria-hidden="true">📂</span><span>Load</span></button>
              <button id="settings-toggle-button" class="project-action-button secondary" type="button"><span class="button-icon" aria-hidden="true">⚙</span><span>Settings</span></button>
            </div>
            <section id="settings-panel" class="settings-panel" hidden>
              <label class="settings-field">
                <span>Default stroke width</span>
                <input id="settings-stroke-width" type="range" min="1" max="24" step="1" />
              </label>
              <label class="settings-field">
                <span>Default shape</span>
                <select id="settings-shape-kind">
                  <option value="rectangle">Rectangle</option>
                  <option value="ellipse">Ellipse</option>
                  <option value="line">Line</option>
                </select>
              </label>
              <label class="settings-field">
                <span>Default pen color</span>
                <input id="settings-canvas-color" type="color" />
              </label>
              <label class="settings-field">
                <span>Speech rate</span>
                <input id="settings-speech-rate" type="range" min="0.6" max="1.6" step="0.1" />
              </label>
              <label class="settings-field settings-checkbox">
                <input id="settings-recolor-enabled" type="checkbox" />
                <span>Enable recolor defaults for future PDF imports</span>
              </label>
              <div class="settings-color-row">
                <label class="settings-field">
                  <span>Recolor foreground</span>
                  <input id="settings-recolor-foreground" type="color" />
                </label>
                <label class="settings-field">
                  <span>Recolor background</span>
                  <input id="settings-recolor-background" type="color" />
                </label>
              </div>
            </section>
            <div class="project-action-row recovery-row">
              <button id="autosave-restore-button" class="project-action-button secondary" type="button" hidden>Restore autosave</button>
              <button id="autosave-clear-button" class="project-action-button secondary" type="button" hidden>Clear recovery</button>
            </div>
            <div id="autosave-status" class="autosave-status">Autosave idle</div>
            <div id="backend-status" class="backend-status">Loading backend status...</div>
          </div>
        </header>

        <section id="workspace-root" class="workspace-root"></section>
      </main>
    </div>
  `;

  const workspaceRoot = requireElement<HTMLElement>(root, "#workspace-root");
  const modeTitle = requireElement<HTMLElement>(root, "#mode-title");
  const modeCopy = requireElement<HTMLElement>(root, "#mode-copy");
  const backendStatus = requireElement<HTMLElement>(root, "#backend-status");
  const projectPathLabel = requireElement<HTMLElement>(root, "#project-path-label");
  const projectPathInput = requireElement<HTMLInputElement>(root, "#project-path-input");
  const saveButton = requireElement<HTMLButtonElement>(root, "#project-save-button");
  const loadButton = requireElement<HTMLButtonElement>(root, "#project-load-button");
  const settingsToggleButton = requireElement<HTMLButtonElement>(root, "#settings-toggle-button");
  const settingsPanel = requireElement<HTMLElement>(root, "#settings-panel");
  const settingsStrokeWidth = requireElement<HTMLInputElement>(root, "#settings-stroke-width");
  const settingsShapeKind = requireElement<HTMLSelectElement>(root, "#settings-shape-kind");
  const settingsCanvasColor = requireElement<HTMLInputElement>(root, "#settings-canvas-color");
  const settingsSpeechRate = requireElement<HTMLInputElement>(root, "#settings-speech-rate");
  const settingsRecolorEnabled = requireElement<HTMLInputElement>(root, "#settings-recolor-enabled");
  const settingsRecolorForeground = requireElement<HTMLInputElement>(root, "#settings-recolor-foreground");
  const settingsRecolorBackground = requireElement<HTMLInputElement>(root, "#settings-recolor-background");
  const autosaveRestoreButton = requireElement<HTMLButtonElement>(root, "#autosave-restore-button");
  const autosaveClearButton = requireElement<HTMLButtonElement>(root, "#autosave-clear-button");
  const autosaveStatus = requireElement<HTMLElement>(root, "#autosave-status");
  const modeButtons = root.querySelectorAll<HTMLButtonElement>(".mode-button");

  let activeWorkspace: WorkspaceController | null = null;
  let bootstrap: AppBootstrap | null = null;
  let preferences = readPreferences();

  const modeMetadata: Record<AppMode, { title: string; copy: string }> = {
    canvas: {
      title: "Canvas Mode",
      copy: "Spatial note-taking workspace",
    },
    pdf: {
      title: "PDF Mode",
      copy: "Document reading and annotation workspace",
    },
  };

  const modeFileHints: Record<AppMode, { label: string; placeholder: string }> = {
    canvas: {
      label: "Canvas file path",
      placeholder: "/tmp/canvas-project.rpdf.json",
    },
    pdf: {
      label: "PDF session path",
      placeholder: "/tmp/pdf-study-session.rpdf.json",
    },
  };

  const autosaveStorageKeys: Record<AppMode, string> = {
    canvas: "rpdf.autosave.canvas",
    pdf: "rpdf.autosave.pdf",
  };

  function syncSettingsPanel() {
    settingsStrokeWidth.value = String(preferences.defaultStrokeWidth);
    settingsShapeKind.value = preferences.defaultShapeKind;
    settingsCanvasColor.value = preferences.defaultCanvasColor;
    settingsSpeechRate.value = String(preferences.speechRate);
    settingsRecolorEnabled.checked = preferences.recolorEnabled;
    settingsRecolorForeground.value = preferences.recolorForeground;
    settingsRecolorBackground.value = preferences.recolorBackground;
  }

  function renderMode(mode: AppMode) {
    for (const button of modeButtons) {
      button.classList.toggle("active", button.dataset.mode === mode);
    }

    modeTitle.textContent = modeMetadata[mode].title;
    modeCopy.textContent = modeMetadata[mode].copy;
    projectPathLabel.textContent = modeFileHints[mode].label;
    projectPathInput.placeholder = modeFileHints[mode].placeholder;

    activeWorkspace?.destroy();
    activeWorkspace = null;
    workspaceRoot.replaceChildren();

    if (mode === "canvas") {
      activeWorkspace = mountCanvasWorkspace(workspaceRoot);
      renderAutosaveRecoveryState(mode);
      return;
    }

    activeWorkspace = mountPdfWorkspace(workspaceRoot, bootstrap);
    renderAutosaveRecoveryState(mode);
  }

  function autosaveKey(mode: AppMode) {
    return autosaveStorageKeys[mode];
  }

  function readAutosave(mode: AppMode): AutosaveRecord | null {
    const rawValue = window.localStorage.getItem(autosaveKey(mode));

    if (!rawValue) {
      return null;
    }

    try {
      return JSON.parse(rawValue) as AutosaveRecord;
    } catch (error) {
      console.error("Could not parse autosave record:", error);
      window.localStorage.removeItem(autosaveKey(mode));
      return null;
    }
  }

  function renderAutosaveRecoveryState(mode: AppMode) {
    const record = readAutosave(mode);
    const hasRecovery = Boolean(record);

    autosaveRestoreButton.hidden = !hasRecovery;
    autosaveClearButton.hidden = !hasRecovery;
    autosaveStatus.textContent = hasRecovery
      ? `Recovery available from ${record?.savedAt ?? "unknown time"}`
      : "Autosave idle";
  }

  function writeAutosaveSnapshot() {
    if (!activeWorkspace) {
      return;
    }

    const mode = state.snapshot.mode;
    const record: AutosaveRecord = {
      savedAt: new Date().toISOString(),
      snapshot: activeWorkspace.exportDocument(),
    };

    window.localStorage.setItem(autosaveKey(mode), JSON.stringify(record));
    renderAutosaveRecoveryState(mode);
  }

  function clearAutosave(mode: AppMode) {
    window.localStorage.removeItem(autosaveKey(mode));
    renderAutosaveRecoveryState(mode);
  }

  async function restoreAutosave() {
    if (!activeWorkspace) {
      backendStatus.textContent = "No active workspace to restore into";
      return;
    }

    const mode = state.snapshot.mode;
    const record = readAutosave(mode);

    if (!record) {
      autosaveStatus.textContent = "No recovery snapshot found";
      return;
    }

    await activeWorkspace.importDocument(record.snapshot);
    backendStatus.textContent = `Restored ${mode} autosave from ${record.savedAt}`;
    renderAutosaveRecoveryState(mode);
  }

  async function saveCurrentDocument() {
    const filePath = projectPathInput.value.trim();

    if (!activeWorkspace) {
      backendStatus.textContent = "No active workspace to save";
      return;
    }

    if (!filePath) {
      backendStatus.textContent = "Enter a file path before saving";
      return;
    }

    const snapshot = activeWorkspace.exportDocument();

    if (snapshot.kind === "canvas") {
      await invoke("save_canvas_project", {
        request: {
          filePath,
          document: snapshot.document,
        },
      });
    } else {
      await invoke("save_pdf_study_session", {
        request: {
          filePath,
          document: snapshot.document,
        },
      });
    }

    backendStatus.textContent = `Saved ${snapshot.kind} document to ${filePath}`;
    clearAutosave(state.snapshot.mode);
  }

  async function loadCurrentDocument() {
    const filePath = projectPathInput.value.trim();

    if (!activeWorkspace) {
      backendStatus.textContent = "No active workspace to load into";
      return;
    }

    if (!filePath) {
      backendStatus.textContent = "Enter a file path before loading";
      return;
    }

    if (state.snapshot.mode === "canvas") {
      const document = await invoke<CanvasDocument>("load_canvas_project", {
        request: {
          filePath,
        },
      });
      await activeWorkspace.importDocument({
        kind: "canvas",
        document,
      });
      backendStatus.textContent = `Loaded canvas document from ${filePath}`;
      renderAutosaveRecoveryState("canvas");
      return;
    }

    const document = await invoke<PdfStudyDocument>("load_pdf_study_session", {
      request: {
        filePath,
      },
    });
    await activeWorkspace.importDocument({
      kind: "pdf",
      document,
    });
    backendStatus.textContent = `Loaded PDF session from ${filePath}`;
    renderAutosaveRecoveryState("pdf");
  }

  function renderBackendStatus(currentBootstrap: AppBootstrap | null) {
    if (!currentBootstrap) {
      backendStatus.textContent = "Backend status unavailable";
      return;
    }

    const { activePdfBackend } = currentBootstrap;
    const summary = activePdfBackend.configured ? "ready" : "boundary only";

    backendStatus.textContent =
      `${activePdfBackend.backendName}: ${summary}`;
    backendStatus.title = activePdfBackend.notes.join("\n");
  }

  for (const button of modeButtons) {
    button.addEventListener("click", () => {
      const mode = button.dataset.mode;

      if (mode === "canvas" || mode === "pdf") {
        state.setMode(mode);
      }
    });
  }

  saveButton.addEventListener("click", () => {
    saveCurrentDocument().catch((error) => {
      backendStatus.textContent = `Save failed: ${String(error)}`;
    });
  });

  loadButton.addEventListener("click", () => {
    loadCurrentDocument().catch((error) => {
      backendStatus.textContent = `Load failed: ${String(error)}`;
    });
  });

  autosaveRestoreButton.addEventListener("click", () => {
    restoreAutosave().catch((error) => {
      backendStatus.textContent = `Recovery failed: ${String(error)}`;
    });
  });

  autosaveClearButton.addEventListener("click", () => {
    clearAutosave(state.snapshot.mode);
    backendStatus.textContent = `Cleared ${state.snapshot.mode} recovery snapshot`;
  });

  state.subscribe(({ mode }) => {
    renderMode(mode);
  });

  settingsToggleButton.addEventListener("click", () => {
    settingsPanel.hidden = !settingsPanel.hidden;
  });

  settingsStrokeWidth.addEventListener("input", () => {
    preferences = {
      ...preferences,
      defaultStrokeWidth: Number(settingsStrokeWidth.value),
    };
    writePreferences(preferences);
  });

  settingsShapeKind.addEventListener("change", () => {
    if (settingsShapeKind.value !== "rectangle" && settingsShapeKind.value !== "ellipse" && settingsShapeKind.value !== "line") {
      return;
    }

    preferences = {
      ...preferences,
      defaultShapeKind: settingsShapeKind.value,
    };
    writePreferences(preferences);
  });

  settingsCanvasColor.addEventListener("input", () => {
    preferences = {
      ...preferences,
      defaultCanvasColor: settingsCanvasColor.value,
    };
    writePreferences(preferences);
  });

  settingsSpeechRate.addEventListener("input", () => {
    preferences = {
      ...preferences,
      speechRate: Number(settingsSpeechRate.value),
    };
    writePreferences(preferences);
  });

  settingsRecolorEnabled.addEventListener("change", () => {
    preferences = {
      ...preferences,
      recolorEnabled: settingsRecolorEnabled.checked,
    };
    writePreferences(preferences);
  });

  settingsRecolorForeground.addEventListener("input", () => {
    preferences = {
      ...preferences,
      recolorForeground: settingsRecolorForeground.value,
    };
    writePreferences(preferences);
  });

  settingsRecolorBackground.addEventListener("input", () => {
    preferences = {
      ...preferences,
      recolorBackground: settingsRecolorBackground.value,
    };
    writePreferences(preferences);
  });

  window.setInterval(() => {
    try {
      writeAutosaveSnapshot();
    } catch (error) {
      autosaveStatus.textContent = `Autosave failed: ${String(error)}`;
    }
  }, 5000);

  window.addEventListener("beforeunload", writeAutosaveSnapshot);

  invoke<AppBootstrap>("get_app_bootstrap")
    .then((result) => {
      bootstrap = result;
      renderBackendStatus(bootstrap);

      if (!bootstrap.supportedModes.includes(state.snapshot.mode)) {
        state.setMode(bootstrap.supportedModes[0] ?? "canvas");
        return;
      }

      renderMode(state.snapshot.mode);
    })
    .catch((error) => {
      backendStatus.textContent = "Backend bootstrap failed";
      backendStatus.title = String(error);
    });

  syncSettingsPanel();
}

function requireElement<T extends Element>(root: ParentNode, selector: string) {
  const element = root.querySelector<T>(selector);

  if (!element) {
    throw new Error(`Required element "${selector}" was not found.`);
  }

  return element;
}
