import { invoke } from "@tauri-apps/api/core";

import { mountCanvasWorkspace } from "../features/canvas/workspace";
import { mountPdfWorkspace } from "../features/pdf/workspace";
import { AppStateStore } from "./state";
import type { AppBootstrap, AppMode, CanvasDocument, PdfStudyDocument, WorkspaceController } from "./types";

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
      return;
    }

    activeWorkspace = mountPdfWorkspace(workspaceRoot, bootstrap);
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

  state.subscribe(({ mode }) => {
    renderMode(mode);
  });

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
