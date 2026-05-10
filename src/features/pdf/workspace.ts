import type { AppBootstrap, WorkspaceController } from "../../app/types";

export function mountPdfWorkspace(
  container: HTMLElement,
  bootstrap: AppBootstrap | null,
): WorkspaceController {
  const reliabilityList = bootstrap?.reliabilityStates.join(", ") ?? "unavailable";
  const backendName = bootstrap?.activePdfBackend.backendName ?? "unknown backend";
  const backendNotes = bootstrap?.activePdfBackend.notes ?? [];

  container.innerHTML = `
    <section class="pdf-workspace">
      <div class="pdf-card">
        <div class="pdf-kicker">PDF workspace shell</div>
        <h2>Document-focused mode boundary is in place.</h2>
        <p>
          This workspace is intentionally separate from the canvas. It is the future home of
          page rendering, annotation overlays, read-aloud controls, recoloring, and trust-state
          messaging.
        </p>
      </div>

      <div class="pdf-card">
        <div class="pdf-kicker">Backend contract</div>
        <p><strong>PDF adapter:</strong> ${backendName}</p>
        <p><strong>Reliability states:</strong> ${reliabilityList}</p>
        <ul class="pdf-note-list">
          ${backendNotes.map((note) => `<li>${escapeHtml(note)}</li>`).join("")}
        </ul>
      </div>

      <div class="pdf-card">
        <div class="pdf-kicker">Prepared IPC surfaces</div>
        <ul class="pdf-note-list">
          <li><code>get_app_bootstrap</code> for shell state and backend capabilities</li>
          <li><code>get_pdf_backend_status</code> for adapter inspection</li>
          <li><code>render_pdf_page</code> for page rendering contract</li>
          <li><code>extract_pdf_page_text</code> for text/reliability contract</li>
        </ul>
      </div>
    </section>
  `;

  return {
    destroy() {
      container.replaceChildren();
    },
  };
}

function escapeHtml(value: string) {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}
