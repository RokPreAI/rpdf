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

export function mountAppShell(root: HTMLElement) {
  const state = new AppStateStore();

  root.innerHTML = `
    <div id="app-shell">
      <aside class="mode-rail">
        <div class="brand-block">
          <div class="brand-kicker">rpdf</div>
          <div class="brand-copy">study shell</div>
        </div>

        <button class="mode-button active" data-mode="canvas" type="button">Canvas</button>
        <button class="mode-button" data-mode="pdf" type="button">PDF</button>
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
              <button id="project-save-button" class="project-action-button" type="button">Save</button>
              <button id="project-load-button" class="project-action-button secondary" type="button">Load</button>
            </div>
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
  const autosaveRestoreButton = requireElement<HTMLButtonElement>(root, "#autosave-restore-button");
  const autosaveClearButton = requireElement<HTMLButtonElement>(root, "#autosave-clear-button");
  const autosaveStatus = requireElement<HTMLElement>(root, "#autosave-status");
  const modeButtons = root.querySelectorAll<HTMLButtonElement>(".mode-button");

  let activeWorkspace: WorkspaceController | null = null;
  let bootstrap: AppBootstrap | null = null;

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
}

function requireElement<T extends Element>(root: ParentNode, selector: string) {
  const element = root.querySelector<T>(selector);

  if (!element) {
    throw new Error(`Required element "${selector}" was not found.`);
  }

  return element;
}
