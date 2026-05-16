import { invoke } from "@tauri-apps/api/core";

import { mountCanvasWorkspace } from "../features/canvas/workspace";
import { mountPdfWorkspace } from "../features/pdf/workspace";
import { setActiveAppConfig } from "./config";
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

type PendingPdfPageImport = {
  sourcePdfPath: string;
  pageIndex: number;
  assetPath: string;
  width: number;
  height: number;
  recolor: {
    enabled: boolean;
    foreground: string;
    background: string;
  };
};

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
              <input id="project-path-input" type="text" placeholder="/tmp/canvas-project.rpdf.json" />
            </label>
            <div class="project-action-row">
              <button id="project-save-button" class="project-action-button" type="button"><span class="button-icon" aria-hidden="true">💾</span><span>Save</span></button>
              <button id="project-load-button" class="project-action-button secondary" type="button"><span class="button-icon" aria-hidden="true">📂</span><span>Load</span></button>
              <button id="canvas-export-svg-button" class="project-action-button secondary" type="button" hidden><span class="button-icon" aria-hidden="true">⬒</span><span>Export SVG</span></button>
            </div>
            <div class="project-action-row recovery-row">
              <button id="autosave-restore-button" class="project-action-button recovery-missing" type="button">Restore autosave</button>
              <button id="autosave-clear-button" class="project-action-button secondary" type="button" hidden>Clear recovery</button>
            </div>
          </div>
        </header>

        <section id="workspace-root" class="workspace-root">
          <div id="backend-status" class="backend-status">Loading backend status...</div>
        </section>
      </main>
    </div>
  `;

  const workspaceRoot = requireElement<HTMLElement>(root, "#workspace-root");
  const modeTitle = requireElement<HTMLElement>(root, "#mode-title");
  const modeCopy = requireElement<HTMLElement>(root, "#mode-copy");
  const backendStatus = requireElement<HTMLElement>(root, "#backend-status");
  const projectPathInput = requireElement<HTMLInputElement>(root, "#project-path-input");
  const saveButton = requireElement<HTMLButtonElement>(root, "#project-save-button");
  const loadButton = requireElement<HTMLButtonElement>(root, "#project-load-button");
  const canvasExportSvgButton = requireElement<HTMLButtonElement>(root, "#canvas-export-svg-button");
  const autosaveRestoreButton = requireElement<HTMLButtonElement>(root, "#autosave-restore-button");
  const autosaveClearButton = requireElement<HTMLButtonElement>(root, "#autosave-clear-button");
  const modeButtons = root.querySelectorAll<HTMLButtonElement>(".mode-button");

  let activeWorkspace: WorkspaceController | null = null;
  let bootstrap: AppBootstrap | null = null;
  const modeWorkspaceSnapshots: Partial<Record<AppMode, WorkspaceDocumentSnapshot>> = {};
  const modeProjectPaths: Record<AppMode, string> = {
    canvas: "",
    pdf: "",
  };

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

  const modeFileHints: Record<AppMode, { placeholder: string }> = {
    canvas: {
      placeholder: "/tmp/canvas-project.rpdf.json",
    },
    pdf: {
      placeholder: "/tmp/pdf-study-session.rpdf.json",
    },
  };

  const autosaveStorageKeys: Record<AppMode, string> = {
    canvas: "rpdf.autosave.canvas",
    pdf: "rpdf.autosave.pdf",
  };

  function renderCanvasExportButton(mode: AppMode) {
    const isCanvasMode = mode === "canvas";
    canvasExportSvgButton.hidden = !isCanvasMode;

    if (!isCanvasMode) {
      canvasExportSvgButton.disabled = true;
      canvasExportSvgButton.title = "";
    }
  }

  function persistActiveModeState() {
    if (!activeWorkspace) {
      return;
    }

    const currentSnapshot = activeWorkspace.exportDocument();
    modeWorkspaceSnapshots[currentSnapshot.kind] = currentSnapshot;
    modeProjectPaths[currentSnapshot.kind] = projectPathInput.value;
  }

  async function restoreModeState(mode: AppMode) {
    const snapshot = modeWorkspaceSnapshots[mode];

    if (!activeWorkspace || !snapshot || snapshot.kind !== mode) {
      return;
    }

    await activeWorkspace.importDocument(snapshot);
  }

  async function renderMode(mode: AppMode) {
    persistActiveModeState();

    for (const button of modeButtons) {
      button.classList.toggle("active", button.dataset.mode === mode);
    }

    modeTitle.textContent = modeMetadata[mode].title;
    modeCopy.textContent = modeMetadata[mode].copy;
    projectPathInput.placeholder = modeFileHints[mode].placeholder;
    projectPathInput.value = modeProjectPaths[mode];
    renderCanvasExportButton(mode);

    activeWorkspace?.destroy();
    activeWorkspace = null;
    workspaceRoot.replaceChildren();

    if (mode === "canvas") {
      activeWorkspace = mountCanvasWorkspace(workspaceRoot);
      renderAutosaveRecoveryState(mode);
      await restoreModeState(mode);
      return;
    }

    activeWorkspace = mountPdfWorkspace(workspaceRoot, bootstrap);
    renderAutosaveRecoveryState(mode);
    await restoreModeState(mode);
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

    autosaveClearButton.hidden = !hasRecovery;
    autosaveRestoreButton.disabled = !hasRecovery;
    autosaveRestoreButton.classList.toggle("recovery-ready", hasRecovery);
    autosaveRestoreButton.classList.toggle("recovery-missing", !hasRecovery);
    autosaveRestoreButton.title = hasRecovery
      ? `Recovery available from ${record?.savedAt ?? "unknown time"}`
      : "No autosave available";
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
      backendStatus.textContent = "No recovery snapshot found";
      return;
    }

    await activeWorkspace.importDocument(record.snapshot);
    modeWorkspaceSnapshots[mode] = record.snapshot;
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
    modeWorkspaceSnapshots[snapshot.kind] = snapshot;
    modeProjectPaths[snapshot.kind] = filePath;

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
      const snapshot: WorkspaceDocumentSnapshot = {
        kind: "canvas",
        document,
      };
      await activeWorkspace.importDocument(snapshot);
      modeWorkspaceSnapshots.canvas = snapshot;
      modeProjectPaths.canvas = filePath;
      backendStatus.textContent = `Loaded canvas document from ${filePath}`;
      renderAutosaveRecoveryState("canvas");
      return;
    }

    const document = await invoke<PdfStudyDocument>("load_pdf_study_session", {
      request: {
        filePath,
      },
    });
    const snapshot: WorkspaceDocumentSnapshot = {
      kind: "pdf",
      document,
    };
    await activeWorkspace.importDocument(snapshot);
    modeWorkspaceSnapshots.pdf = snapshot;
    modeProjectPaths.pdf = filePath;
    backendStatus.textContent = `Loaded PDF session from ${filePath}`;
    renderAutosaveRecoveryState("pdf");
  }

  function renderBackendStatus(currentBootstrap: AppBootstrap | null) {
    if (!currentBootstrap) {
      backendStatus.textContent = "Backend status unavailable";
      return;
    }

    const { activePdfBackend, appConfigPath, appConfigWarnings } = currentBootstrap;
    const summary = activePdfBackend.configured ? "ready" : "boundary only";
    const configSummary = appConfigWarnings.length > 0 ? " | config fallback" : "";

    backendStatus.textContent =
      `${activePdfBackend.backendName}: ${summary}${configSummary}`;
    backendStatus.title = [
      `Config: ${appConfigPath}`,
      ...activePdfBackend.notes,
      ...appConfigWarnings,
    ].join("\n");

    for (const warning of appConfigWarnings) {
      console.warn("[app config]", warning);
    }
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

  canvasExportSvgButton.addEventListener("click", () => {
    const explicitPath = projectPathInput.value.trim();

    if (state.snapshot.mode !== "canvas") {
      return;
    }

    if (explicitPath && !explicitPath.toLowerCase().endsWith(".svg")) {
      backendStatus.textContent = "SVG export uses the header path only for `.svg` destinations. Enter a `.svg` path or clear the field to choose a save location.";
      return;
    }

    window.dispatchEvent(new CustomEvent("rpdf:request-canvas-svg-export", {
      detail: {
        filePath: explicitPath,
      },
    }));
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

  projectPathInput.addEventListener("input", () => {
    modeProjectPaths[state.snapshot.mode] = projectPathInput.value;
  });

  window.addEventListener("rpdf:canvas-svg-export-state", ((event) => {
    const customEvent = event as CustomEvent<{
      eligible: boolean;
      message: string;
    }>;

    if (state.snapshot.mode !== "canvas") {
      return;
    }

    canvasExportSvgButton.disabled = !customEvent.detail.eligible;
    canvasExportSvgButton.title = customEvent.detail.message;
  }) as EventListener);

  window.addEventListener("rpdf:canvas-svg-export-result", ((event) => {
    const customEvent = event as CustomEvent<{
      level: "success" | "info" | "error";
      message: string;
      savedPath: string | null;
    }>;

    if (state.snapshot.mode !== "canvas") {
      return;
    }

    backendStatus.textContent = customEvent.detail.message;

    if (customEvent.detail.savedPath) {
      projectPathInput.value = customEvent.detail.savedPath;
      modeProjectPaths.canvas = customEvent.detail.savedPath;
    }
  }) as EventListener);

  state.subscribe(({ mode }) => {
    void renderMode(mode);
  });

  window.addEventListener("rpdf:request-pdf-page-import", (event) => {
    const customEvent = event as CustomEvent<PendingPdfPageImport>;
    const dispatchImport = () => {
      window.dispatchEvent(new CustomEvent("rpdf:canvas-import-pdf-page", {
        detail: customEvent.detail,
      }));
    };

    if (state.snapshot.mode !== "canvas") {
      state.setMode("canvas");
      window.setTimeout(dispatchImport, 0);
      return;
    }

    dispatchImport();
  });

  window.setInterval(() => {
    try {
      writeAutosaveSnapshot();
    } catch (error) {
      backendStatus.textContent = `Autosave failed: ${String(error)}`;
    }
  }, 5000);

  window.addEventListener("beforeunload", writeAutosaveSnapshot);

  invoke<AppBootstrap>("get_app_bootstrap")
    .then((result) => {
      bootstrap = result;
      setActiveAppConfig(result.appConfig);
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
}

function requireElement<T extends Element>(root: ParentNode, selector: string) {
  const element = root.querySelector<T>(selector);

  if (!element) {
    throw new Error(`Required element "${selector}" was not found.`);
  }

  return element;
}
