import { invoke } from "@tauri-apps/api/core";

import { mountCanvasWorkspace } from "../features/canvas/workspace";
import { mountPdfWorkspace } from "../features/pdf/workspace";
import { AppStateStore } from "./state";
import type { AppBootstrap, AppMode, WorkspaceController } from "./types";

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

          <div id="backend-status" class="backend-status">Loading backend status...</div>
        </header>

        <section id="workspace-root" class="workspace-root"></section>
      </main>
    </div>
  `;

  const workspaceRoot = requireElement(root, "#workspace-root");
  const modeTitle = requireElement(root, "#mode-title");
  const modeCopy = requireElement(root, "#mode-copy");
  const backendStatus = requireElement(root, "#backend-status");
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

  function renderMode(mode: AppMode) {
    for (const button of modeButtons) {
      button.classList.toggle("active", button.dataset.mode === mode);
    }

    modeTitle.textContent = modeMetadata[mode].title;
    modeCopy.textContent = modeMetadata[mode].copy;

    activeWorkspace?.destroy();
    activeWorkspace = null;
    workspaceRoot.replaceChildren();

    if (mode === "canvas") {
      activeWorkspace = mountCanvasWorkspace(workspaceRoot);
      return;
    }

    activeWorkspace = mountPdfWorkspace(workspaceRoot, bootstrap);
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

function requireElement(root: ParentNode, selector: string) {
  const element = root.querySelector<HTMLElement>(selector);

  if (!element) {
    throw new Error(`Required element "${selector}" was not found.`);
  }

  return element;
}
