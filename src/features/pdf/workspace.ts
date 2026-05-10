import { invoke } from "@tauri-apps/api/core";

import type {
  AppBootstrap,
  ExtractPdfTextRequest,
  OpenPdfDocumentRequest,
  OpenPdfDocumentResponse,
  PageTextExtraction,
  PdfPageAnnotationLayerDocument,
  PdfStudyDocument,
  PdfBackendStatus,
  ReadingReliabilityState,
  RenderPdfPageRequest,
  RenderPdfPageResponse,
  WorkspaceController,
  WorkspaceDocumentSnapshot,
} from "../../app/types";

type Point = {
  x: number;
  y: number;
};

type AnnotationStroke = {
  color: string;
  width: number;
  points: Point[];
};

type PdfWorkspaceState = {
  document: OpenPdfDocumentResponse | null;
  pageIndex: number;
  extraction: PageTextExtraction | null;
  backendStatus: PdfBackendStatus | null;
  pageRender: RenderPdfPageResponse | null;
  pageError: string | null;
};

export function mountPdfWorkspace(
  container: HTMLElement,
  bootstrap: AppBootstrap | null,
): WorkspaceController {
  container.innerHTML = `
    <section class="pdf-layout">
      <aside class="pdf-sidebar">
        <div class="pdf-card pdf-sidebar-card">
          <div class="pdf-kicker">Document</div>
          <label class="pdf-field">
            <span>PDF path</span>
            <input id="pdf-path-input" type="text" placeholder="/home/rok/Documents/paper.pdf" />
          </label>
          <div class="pdf-action-row">
            <button id="pdf-open-button" class="pdf-button" type="button">Open PDF</button>
            <button id="pdf-refresh-button" class="pdf-button ghost" type="button">Refresh</button>
          </div>
          <p id="pdf-open-error" class="pdf-inline-error" hidden></p>
        </div>

        <div class="pdf-card pdf-sidebar-card">
          <div class="pdf-kicker">Navigation</div>
          <div class="pdf-action-row">
            <button id="pdf-prev-button" class="pdf-button ghost" type="button">Previous</button>
            <button id="pdf-next-button" class="pdf-button ghost" type="button">Next</button>
          </div>
          <div id="pdf-page-label" class="pdf-page-label">No document open</div>
          <p class="pdf-helper-copy">
            The first pass keeps navigation state and backend calls explicit even before Pdfium rendering is configured.
          </p>
        </div>

        <div class="pdf-card pdf-sidebar-card">
          <div class="pdf-kicker">Reading trust state</div>
          <div id="pdf-reliability-badge" class="pdf-reliability-badge unavailable">Unavailable</div>
          <p id="pdf-reliability-copy" class="pdf-helper-copy">
            Open a document to inspect native text extraction status for the current page.
          </p>
          <ul id="pdf-backend-notes" class="pdf-note-list"></ul>
        </div>
      </aside>

      <section class="pdf-viewer-panel">
        <div class="pdf-card pdf-viewer-card">
          <header class="pdf-viewer-header">
            <div>
              <div class="pdf-kicker">Viewer</div>
              <h2 id="pdf-document-title">PDF Mode</h2>
            </div>
            <div class="pdf-viewer-meta">
              <div id="pdf-backend-name">Backend unavailable</div>
              <div id="pdf-page-count-copy" class="pdf-helper-copy">Page count unknown</div>
            </div>
          </header>

          <div id="pdf-stage-shell" class="pdf-stage-shell empty">
            <div id="pdf-stage-empty" class="pdf-stage-empty">
              Enter a PDF path and open it to create a reading-focused workspace.
            </div>
            <div id="pdf-stage" class="pdf-stage" hidden>
              <img id="pdf-page-image" class="pdf-page-image" alt="Rendered PDF page" hidden />
              <canvas id="pdf-annotation-layer" class="pdf-annotation-layer"></canvas>
              <div id="pdf-render-placeholder" class="pdf-render-placeholder">
                The PDF engine boundary is connected, but page rendering is not configured on this machine yet.
              </div>
            </div>
          </div>

          <footer class="pdf-stage-footer">
            <span>Annotation overlay</span>
            <span class="pdf-helper-copy">Draw directly on the page stage to test page-scoped markup placement.</span>
          </footer>
        </div>
      </section>
    </section>
  `;

  const state: PdfWorkspaceState = {
    document: null,
    pageIndex: 0,
    extraction: null,
    backendStatus: bootstrap?.activePdfBackend ?? null,
    pageRender: null,
    pageError: null,
  };

  const pathInput = requireElement<HTMLInputElement>(container, "#pdf-path-input");
  const openButton = requireElement<HTMLButtonElement>(container, "#pdf-open-button");
  const refreshButton = requireElement<HTMLButtonElement>(container, "#pdf-refresh-button");
  const prevButton = requireElement<HTMLButtonElement>(container, "#pdf-prev-button");
  const nextButton = requireElement<HTMLButtonElement>(container, "#pdf-next-button");
  const pageLabel = requireElement<HTMLElement>(container, "#pdf-page-label");
  const openError = requireElement<HTMLElement>(container, "#pdf-open-error");
  const reliabilityBadge = requireElement<HTMLElement>(container, "#pdf-reliability-badge");
  const reliabilityCopy = requireElement<HTMLElement>(container, "#pdf-reliability-copy");
  const backendNotesList = requireElement<HTMLElement>(container, "#pdf-backend-notes");
  const documentTitle = requireElement<HTMLElement>(container, "#pdf-document-title");
  const backendName = requireElement<HTMLElement>(container, "#pdf-backend-name");
  const pageCountCopy = requireElement<HTMLElement>(container, "#pdf-page-count-copy");
  const stageShell = requireElement<HTMLElement>(container, "#pdf-stage-shell");
  const stageEmpty = requireElement<HTMLElement>(container, "#pdf-stage-empty");
  const stage = requireElement<HTMLElement>(container, "#pdf-stage");
  const pageImage = requireElement<HTMLImageElement>(container, "#pdf-page-image");
  const annotationLayer = requireElement<HTMLCanvasElement>(container, "#pdf-annotation-layer");
  const renderPlaceholder = requireElement<HTMLElement>(container, "#pdf-render-placeholder");

  const context = annotationLayer.getContext("2d");

  if (!context) {
    throw new Error("Could not get PDF annotation canvas context.");
  }

  const annotationContext = context;
  const annotationsByPage = new Map<number, AnnotationStroke[]>();
  let currentStroke: AnnotationStroke | null = null;
  let documentId: string = crypto.randomUUID();

  function getCurrentPageStrokes() {
    const strokes = annotationsByPage.get(state.pageIndex);

    if (strokes) {
      return strokes;
    }

    const nextStrokes: AnnotationStroke[] = [];
    annotationsByPage.set(state.pageIndex, nextStrokes);
    return nextStrokes;
  }

  function setOpenError(message: string | null) {
    openError.hidden = !message;
    openError.textContent = message ?? "";
  }

  function renderBackendNotes(notes: string[]) {
    backendNotesList.innerHTML = notes.map((note) => `<li>${escapeHtml(note)}</li>`).join("");
  }

  function renderReliability(reliability: ReadingReliabilityState, warning: string | null) {
    reliabilityBadge.className = `pdf-reliability-badge ${reliability}`;
    reliabilityBadge.textContent = reliability.replace(/_/g, " ");
    reliabilityCopy.textContent = warning
      ?? "Native text extraction has not produced a warning for this page.";
  }

  function resizeAnnotationLayer() {
    const rect = stage.getBoundingClientRect();
    const scale = window.devicePixelRatio || 1;

    annotationLayer.width = Math.max(1, Math.floor(rect.width * scale));
    annotationLayer.height = Math.max(1, Math.floor(rect.height * scale));
    annotationLayer.style.width = `${rect.width}px`;
    annotationLayer.style.height = `${rect.height}px`;

    annotationContext.setTransform(scale, 0, 0, scale, 0, 0);
    redrawAnnotations();
  }

  function redrawAnnotations() {
    annotationContext.clearRect(0, 0, annotationLayer.width, annotationLayer.height);
    annotationContext.lineCap = "round";
    annotationContext.lineJoin = "round";

    for (const stroke of getCurrentPageStrokes()) {
      drawStroke(stroke);
    }
  }

  function drawStroke(stroke: AnnotationStroke) {
    if (stroke.points.length === 0) {
      return;
    }

    annotationContext.strokeStyle = stroke.color;
    annotationContext.fillStyle = stroke.color;
    annotationContext.lineWidth = stroke.width;

    if (stroke.points.length === 1) {
      const point = stroke.points[0];
      annotationContext.beginPath();
      annotationContext.arc(point.x, point.y, stroke.width / 2, 0, Math.PI * 2);
      annotationContext.fill();
      return;
    }

    annotationContext.beginPath();
    annotationContext.moveTo(stroke.points[0].x, stroke.points[0].y);

    for (let index = 1; index < stroke.points.length; index += 1) {
      annotationContext.lineTo(stroke.points[index].x, stroke.points[index].y);
    }

    annotationContext.stroke();
  }

  function eventPoint(event: PointerEvent): Point {
    const rect = annotationLayer.getBoundingClientRect();

    return {
      x: event.clientX - rect.left,
      y: event.clientY - rect.top,
    };
  }

  function renderDocumentState() {
    const document = state.document;
    const hasDocument = Boolean(document);

    stageShell.classList.toggle("empty", !hasDocument);
    stageEmpty.hidden = hasDocument;
    stage.hidden = !hasDocument;
    prevButton.disabled = !hasDocument || state.pageIndex <= 0;
    nextButton.disabled = !hasDocument;
    refreshButton.disabled = !hasDocument;

    if (!document) {
      documentTitle.textContent = "PDF Mode";
      pageCountCopy.textContent = "Page count unknown";
      pageLabel.textContent = "No document open";
      renderBackendNotes(state.backendStatus?.notes ?? bootstrap?.activePdfBackend.notes ?? []);
      renderReliability("unavailable", "Open a document to inspect native text extraction status for the current page.");
      pageImage.hidden = true;
      renderPlaceholder.hidden = false;
      renderPlaceholder.textContent = "The PDF page stage is ready, but no document is open yet.";
      resizeAnnotationLayer();
      redrawAnnotations();
      return;
    }

    documentTitle.textContent = document.documentName;
    pageCountCopy.textContent = document.pageCount === null
      ? "Page count not reported by the current backend"
      : `${document.pageCount} pages detected`;
    pageLabel.textContent = `Page ${state.pageIndex + 1}${document.pageCount ? ` of ${document.pageCount}` : ""}`;

    const reliability = state.extraction?.reliability ?? "unavailable";
    const warning = state.pageError ?? state.extraction?.warning ?? null;
    renderReliability(reliability, warning);

    const notes = document.notes.length > 0
      ? document.notes
      : ["The PDF backend did not return additional notes for this document."];

    renderBackendNotes(notes);

    if (state.pageRender) {
      pageImage.hidden = false;
      pageImage.src = `data:${state.pageRender.mimeType};base64,${state.pageRender.dataBase64}`;
      renderPlaceholder.hidden = true;
    } else {
      pageImage.hidden = true;
      renderPlaceholder.hidden = false;
      renderPlaceholder.textContent = state.pageError
        ?? "The PDF engine boundary is connected, but page rendering is not configured on this machine yet.";
    }

    resizeAnnotationLayer();
  }

  async function refreshPageState() {
    const document = state.document;

    if (!document) {
      return;
    }

    state.pageRender = null;
    state.pageError = null;
    state.extraction = null;
    renderDocumentState();

    try {
      state.backendStatus = await invoke<PdfBackendStatus>("get_pdf_backend_status");
      backendName.textContent = `${state.backendStatus.backendName}: ${state.backendStatus.configured ? "ready" : "boundary only"}`;
    } catch (error) {
      backendName.textContent = "Backend status unavailable";
      state.pageError = String(error);
    }

    try {
      state.pageRender = await invoke<RenderPdfPageResponse>("render_pdf_page", {
        request: {
          documentPath: document.documentPath,
          pageIndex: state.pageIndex,
          targetWidth: 1200,
          targetHeight: 1600,
        } satisfies RenderPdfPageRequest,
      });
    } catch (error) {
      state.pageError = String(error);
    }

    try {
      state.extraction = await invoke<PageTextExtraction>("extract_pdf_page_text", {
        request: {
          documentPath: document.documentPath,
          pageIndex: state.pageIndex,
        } satisfies ExtractPdfTextRequest,
      });
    } catch (error) {
      state.extraction = {
        pageIndex: state.pageIndex,
        reliability: "unavailable",
        warning: String(error),
        spans: [],
      };
    }

    renderDocumentState();
  }

  async function openDocument() {
    const documentPath = pathInput.value.trim();

    if (!documentPath) {
      setOpenError("Enter a PDF path first.");
      return;
    }

    setOpenError(null);

    try {
      state.document = await invoke<OpenPdfDocumentResponse>("open_pdf_document", {
        request: {
          documentPath,
        } satisfies OpenPdfDocumentRequest,
      });
      documentId = crypto.randomUUID();
      annotationsByPage.clear();
      state.pageIndex = 0;
      backendName.textContent = `${bootstrap?.activePdfBackend.backendName ?? "PDF backend"}: ${state.document.backendReady ? "ready" : "boundary only"}`;
      await refreshPageState();
    } catch (error) {
      state.document = null;
      state.pageIndex = 0;
      state.pageRender = null;
      state.pageError = null;
      state.extraction = null;
      setOpenError(String(error));
      renderDocumentState();
    }
  }

  function movePage(delta: number) {
    if (!state.document) {
      return;
    }

    const nextPageIndex = Math.max(0, state.pageIndex + delta);
    state.pageIndex = state.document.pageCount === null
      ? nextPageIndex
      : Math.min(nextPageIndex, state.document.pageCount - 1);
    void refreshPageState();
  }

  const onPointerDown = (event: PointerEvent) => {
    if (event.button !== 0 || state.document === null) {
      return;
    }

    const stroke: AnnotationStroke = {
      color: "#f7768e",
      width: 3,
      points: [eventPoint(event)],
    };

    currentStroke = stroke;
    getCurrentPageStrokes().push(stroke);
    annotationLayer.setPointerCapture(event.pointerId);
    redrawAnnotations();
  };

  const onPointerMove = (event: PointerEvent) => {
    if (!currentStroke) {
      return;
    }

    currentStroke.points.push(eventPoint(event));
    redrawAnnotations();
  };

  const onPointerUp = (event: PointerEvent) => {
    currentStroke = null;

    if (annotationLayer.hasPointerCapture(event.pointerId)) {
      annotationLayer.releasePointerCapture(event.pointerId);
    }
  };

  openButton.addEventListener("click", () => {
    void openDocument();
  });
  refreshButton.addEventListener("click", () => {
    void refreshPageState();
  });
  prevButton.addEventListener("click", () => {
    movePage(-1);
  });
  nextButton.addEventListener("click", () => {
    movePage(1);
  });
  pathInput.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      void openDocument();
    }
  });
  annotationLayer.addEventListener("pointerdown", onPointerDown);
  annotationLayer.addEventListener("pointermove", onPointerMove);
  annotationLayer.addEventListener("pointerup", onPointerUp);
  annotationLayer.addEventListener("pointercancel", onPointerUp);
  window.addEventListener("resize", resizeAnnotationLayer);

  backendName.textContent = `${bootstrap?.activePdfBackend.backendName ?? "Backend unavailable"}: ${bootstrap?.activePdfBackend.configured ? "ready" : "boundary only"}`;
  renderBackendNotes(bootstrap?.activePdfBackend.notes ?? []);
  renderDocumentState();

  return {
    exportDocument(): WorkspaceDocumentSnapshot {
      const annotations: PdfPageAnnotationLayerDocument[] = Array.from(annotationsByPage.entries())
        .map(([pageIndex, strokes]) => ({
          pageIndex,
          strokes: strokes.map((stroke) => ({
            color: stroke.color,
            width: stroke.width,
            points: stroke.points.map((point) => ({
              x: point.x,
              y: point.y,
            })),
          })),
          notes: [],
        }))
        .sort((firstLayer, secondLayer) => firstLayer.pageIndex - secondLayer.pageIndex);

      const readingCache = state.extraction ? [{
        pageIndex: state.extraction.pageIndex,
        reliability: state.extraction.reliability,
        sourceKind: (state.extraction.reliability.startsWith("ocr") ? "ocr" : "native") as "ocr" | "native",
        cacheKey: null,
      }] : [];

      const document: PdfStudyDocument = {
        version: {
          major: 1,
          minor: 0,
        },
        id: documentId,
        sourcePdfPath: state.document?.documentPath ?? "",
        pageCount: state.document?.pageCount ?? null,
        currentPageIndex: state.pageIndex,
        annotations,
        recolor: {
          enabled: false,
          foreground: "#c0caf5",
          background: "#1a1b26",
        },
        readingCache,
      };

      return {
        kind: "pdf",
        document,
      };
    },
    async importDocument(snapshot) {
      if (snapshot.kind !== "pdf") {
        throw new Error("PDF workspace cannot load a non-PDF study session.");
      }

      documentId = snapshot.document.id;
      annotationsByPage.clear();
      for (const layer of snapshot.document.annotations) {
        annotationsByPage.set(
          layer.pageIndex,
          layer.strokes.map((stroke) => ({
            color: stroke.color,
            width: stroke.width,
            points: stroke.points.map((point) => ({
              x: point.x,
              y: point.y,
            })),
          })),
        );
      }

      state.document = {
        documentPath: snapshot.document.sourcePdfPath,
        documentName: fileNameFromPath(snapshot.document.sourcePdfPath),
        pageCount: snapshot.document.pageCount,
        backendReady: bootstrap?.activePdfBackend.configured ?? false,
        notes: bootstrap?.activePdfBackend.notes ?? [],
      };
      state.pageIndex = snapshot.document.currentPageIndex;
      state.pageRender = null;
      state.pageError = null;
      state.extraction = null;
      pathInput.value = snapshot.document.sourcePdfPath;
      await refreshPageState();
      redrawAnnotations();
    },
    destroy() {
      window.removeEventListener("resize", resizeAnnotationLayer);
      annotationLayer.removeEventListener("pointerdown", onPointerDown);
      annotationLayer.removeEventListener("pointermove", onPointerMove);
      annotationLayer.removeEventListener("pointerup", onPointerUp);
      annotationLayer.removeEventListener("pointercancel", onPointerUp);
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

function fileNameFromPath(value: string) {
  const segments = value.split(/[/\\]/);
  return segments[segments.length - 1] || value;
}

function requireElement<T extends HTMLElement>(root: ParentNode, selector: string) {
  const element = root.querySelector<T>(selector);

  if (!element) {
    throw new Error(`Required element "${selector}" was not found.`);
  }

  return element;
}
