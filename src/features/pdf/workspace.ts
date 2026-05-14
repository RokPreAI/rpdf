import { invoke } from "@tauri-apps/api/core";

import type {
  AppBootstrap,
  ExtractPdfTextRequest,
  OpenPdfDocumentRequest,
  OpenPdfDocumentResponse,
  PageTextExtraction,
  PdfBackendStatus,
  PdfPageAnnotationLayerDocument,
  PdfRecolorSettingsDocument,
  PdfStudyDocument,
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
  nativeExtraction: PageTextExtraction | null;
  ocrExtraction: PageTextExtraction | null;
  backendStatus: PdfBackendStatus | null;
  pageRender: RenderPdfPageResponse | null;
  pageImageDataUrl: string | null;
  pageError: string | null;
  readingStatus: string;
  isOcrRunning: boolean;
  isSpeaking: boolean;
  activeSpeechBackend: "native" | "browser" | null;
  recolor: PdfRecolorSettingsDocument;
};

const PREFERENCES_STORAGE_KEY = "rpdf.preferences.v1";
const RECENT_PDF_PATHS_STORAGE_KEY = "rpdf.recent-pdf-paths.v1";
const MAX_RECENT_PDF_PATHS = 5;
const DEFAULT_PAGE_FRAME_WIDTH = 1200;
const DEFAULT_PAGE_FRAME_HEIGHT = 1600;

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
            <button id="pdf-open-button" class="pdf-button" type="button">📂 Open PDF</button>
            <button id="pdf-refresh-button" class="pdf-button ghost" type="button">↻ Refresh</button>
          </div>
          <div id="pdf-recent-paths-section" class="pdf-recent-paths" hidden>
            <div class="pdf-recent-paths-header">Recent PDFs</div>
            <div id="pdf-recent-paths-list" class="pdf-recent-paths-list"></div>
          </div>
          <p id="pdf-open-error" class="pdf-inline-error" hidden></p>
        </div>

        <div class="pdf-card pdf-sidebar-card">
          <div class="pdf-kicker">Navigation</div>
          <div class="pdf-action-row">
            <button id="pdf-prev-button" class="pdf-button ghost" type="button">← Previous</button>
            <button id="pdf-next-button" class="pdf-button ghost" type="button">Next →</button>
          </div>
          <div id="pdf-page-label" class="pdf-page-label">No document open</div>
          <p class="pdf-helper-copy">
            Native text is attempted first. OCR stays manual and explicit when native extraction is weak.
          </p>
        </div>

        <div class="pdf-card pdf-sidebar-card">
          <div class="pdf-kicker">Reading trust state</div>
          <div id="pdf-reliability-badge" class="pdf-reliability-badge unavailable">Unavailable</div>
          <p id="pdf-reliability-copy" class="pdf-helper-copy">
            Open a document to inspect native text extraction status for the current page.
          </p>
          <div id="pdf-reading-source" class="pdf-helper-copy"></div>
          <ul id="pdf-backend-notes" class="pdf-note-list"></ul>
        </div>

        <div class="pdf-card pdf-sidebar-card">
          <div class="pdf-kicker">Reading controls</div>
          <div class="pdf-action-row">
            <button id="pdf-read-button" class="pdf-button" type="button">▶ Read page</button>
            <button id="pdf-stop-button" class="pdf-button ghost" type="button">■ Stop</button>
            <button id="pdf-ocr-button" class="pdf-button ghost" type="button">◌ Run OCR fallback</button>
          </div>
          <p id="pdf-reading-status" class="pdf-helper-copy">
            Reading is local and page-scoped. Weak extraction does not imply reliable follow-along.
          </p>
          <div id="pdf-text-preview" class="pdf-text-preview">
            Open a document to inspect extracted reading text for the current page.
          </div>
        </div>

        <div class="pdf-card pdf-sidebar-card">
          <div class="pdf-kicker">Recolor and import</div>
          <label class="pdf-recolor-toggle">
            <span>Enable recolor for the current page</span>
            <input id="pdf-recolor-enabled" type="checkbox" />
          </label>
          <div id="pdf-recolor-controls" class="pdf-recolor-controls">
            <label class="pdf-recolor-field">
              <span>Foreground</span>
              <div class="pdf-recolor-input-shell">
                <input id="pdf-recolor-foreground" type="color" />
                <span id="pdf-recolor-foreground-swatch" class="pdf-recolor-swatch" aria-hidden="true"></span>
                <code id="pdf-recolor-foreground-value" class="pdf-recolor-value">#c0caf5</code>
              </div>
            </label>
            <label class="pdf-recolor-field">
              <span>Background</span>
              <div class="pdf-recolor-input-shell">
                <input id="pdf-recolor-background" type="color" />
                <span id="pdf-recolor-background-swatch" class="pdf-recolor-swatch" aria-hidden="true"></span>
                <code id="pdf-recolor-background-value" class="pdf-recolor-value">#1a1b26</code>
              </div>
            </label>
          </div>
          <div class="pdf-action-row">
            <button id="pdf-import-canvas-button" class="pdf-button ghost" type="button">⇢ Import page to canvas</button>
          </div>
          <p class="pdf-helper-copy">
            Imported canvas pages preserve the page path, page index, and the recolor settings used at import time.
          </p>
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
              <div id="pdf-page-frame" class="pdf-page-frame" hidden>
                <img id="pdf-page-image" class="pdf-page-image" alt="Rendered PDF page" hidden />
                <canvas id="pdf-annotation-layer" class="pdf-annotation-layer"></canvas>
              </div>
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
    nativeExtraction: null,
    ocrExtraction: null,
    backendStatus: bootstrap?.activePdfBackend ?? null,
    pageRender: null,
    pageImageDataUrl: null,
    pageError: null,
    readingStatus: "Reading is local and page-scoped. Weak extraction does not imply reliable follow-along.",
    isOcrRunning: false,
    isSpeaking: false,
    activeSpeechBackend: null,
    recolor: readPdfPreferences().recolor,
  };

  const pathInput = requireElement<HTMLInputElement>(container, "#pdf-path-input");
  const openButton = requireElement<HTMLButtonElement>(container, "#pdf-open-button");
  const refreshButton = requireElement<HTMLButtonElement>(container, "#pdf-refresh-button");
  const recentPathsSection = requireElement<HTMLElement>(container, "#pdf-recent-paths-section");
  const recentPathsList = requireElement<HTMLElement>(container, "#pdf-recent-paths-list");
  const prevButton = requireElement<HTMLButtonElement>(container, "#pdf-prev-button");
  const nextButton = requireElement<HTMLButtonElement>(container, "#pdf-next-button");
  const readButton = requireElement<HTMLButtonElement>(container, "#pdf-read-button");
  const stopButton = requireElement<HTMLButtonElement>(container, "#pdf-stop-button");
  const ocrButton = requireElement<HTMLButtonElement>(container, "#pdf-ocr-button");
  const importCanvasButton = requireElement<HTMLButtonElement>(container, "#pdf-import-canvas-button");
  const pageLabel = requireElement<HTMLElement>(container, "#pdf-page-label");
  const openError = requireElement<HTMLElement>(container, "#pdf-open-error");
  const reliabilityBadge = requireElement<HTMLElement>(container, "#pdf-reliability-badge");
  const reliabilityCopy = requireElement<HTMLElement>(container, "#pdf-reliability-copy");
  const readingSource = requireElement<HTMLElement>(container, "#pdf-reading-source");
  const backendNotesList = requireElement<HTMLElement>(container, "#pdf-backend-notes");
  const readingStatus = requireElement<HTMLElement>(container, "#pdf-reading-status");
  const textPreview = requireElement<HTMLElement>(container, "#pdf-text-preview");
  const recolorEnabledInput = requireElement<HTMLInputElement>(container, "#pdf-recolor-enabled");
  const recolorControls = requireElement<HTMLElement>(container, "#pdf-recolor-controls");
  const recolorForegroundInput = requireElement<HTMLInputElement>(container, "#pdf-recolor-foreground");
  const recolorBackgroundInput = requireElement<HTMLInputElement>(container, "#pdf-recolor-background");
  const recolorForegroundSwatch = requireElement<HTMLElement>(container, "#pdf-recolor-foreground-swatch");
  const recolorBackgroundSwatch = requireElement<HTMLElement>(container, "#pdf-recolor-background-swatch");
  const recolorForegroundValue = requireElement<HTMLElement>(container, "#pdf-recolor-foreground-value");
  const recolorBackgroundValue = requireElement<HTMLElement>(container, "#pdf-recolor-background-value");
  const documentTitle = requireElement<HTMLElement>(container, "#pdf-document-title");
  const backendName = requireElement<HTMLElement>(container, "#pdf-backend-name");
  const pageCountCopy = requireElement<HTMLElement>(container, "#pdf-page-count-copy");
  const stageShell = requireElement<HTMLElement>(container, "#pdf-stage-shell");
  const stageEmpty = requireElement<HTMLElement>(container, "#pdf-stage-empty");
  const stage = requireElement<HTMLElement>(container, "#pdf-stage");
  const pageFrame = requireElement<HTMLElement>(container, "#pdf-page-frame");
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
  let speechRate = readSpeechRatePreference();
  let activeSpeechRequestId = 0;

  pageImage.style.zIndex = "1";
  annotationLayer.style.zIndex = "2";
  annotationLayer.style.cursor = "crosshair";

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

  function escapeText(value: string) {
    return value
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }

  function renderBackendNotes(notes: string[]) {
    backendNotesList.innerHTML = notes.map((note) => `<li>${escapeText(note)}</li>`).join("");
  }

  function readRecentPdfPaths() {
    const rawValue = window.localStorage.getItem(RECENT_PDF_PATHS_STORAGE_KEY);

    if (!rawValue) {
      return [] as string[];
    }

    try {
      const parsed = JSON.parse(rawValue);

      if (!Array.isArray(parsed)) {
        return [] as string[];
      }

      return parsed
        .filter((entry): entry is string => typeof entry === "string")
        .map((entry) => entry.trim())
        .filter((entry) => entry.length > 0)
        .slice(0, MAX_RECENT_PDF_PATHS);
    } catch (error) {
      console.error("Could not parse recent PDF paths:", error);
      window.localStorage.removeItem(RECENT_PDF_PATHS_STORAGE_KEY);
      return [] as string[];
    }
  }

  function writeRecentPdfPaths(paths: string[]) {
    window.localStorage.setItem(
      RECENT_PDF_PATHS_STORAGE_KEY,
      JSON.stringify(paths.slice(0, MAX_RECENT_PDF_PATHS)),
    );
  }

  function pushRecentPdfPath(documentPath: string) {
    const normalizedPath = documentPath.trim();

    if (!normalizedPath) {
      return;
    }

    const nextPaths = [
      normalizedPath,
      ...readRecentPdfPaths().filter((entry) => entry !== normalizedPath),
    ].slice(0, MAX_RECENT_PDF_PATHS);

    writeRecentPdfPaths(nextPaths);
    renderRecentPdfPaths();
  }

  function renderRecentPdfPaths() {
    const recentPaths = readRecentPdfPaths();
    recentPathsSection.hidden = recentPaths.length === 0;

    if (recentPaths.length === 0) {
      recentPathsList.replaceChildren();
      return;
    }

    const buttons = recentPaths.map((recentPath) => {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "pdf-button ghost pdf-recent-path-button";
      button.title = recentPath;

      const name = document.createElement("span");
      name.className = "pdf-recent-path-name";
      name.textContent = fileNameFromPath(recentPath);

      const value = document.createElement("span");
      value.className = "pdf-recent-path-value";
      value.textContent = recentPath;

      button.append(name, value);
      button.addEventListener("click", () => {
        pathInput.value = recentPath;
        void openDocument(recentPath);
      });

      return button;
    });

    recentPathsList.replaceChildren(...buttons);
  }

  function activeExtraction() {
    return state.ocrExtraction ?? state.nativeExtraction;
  }

  function currentReadableText() {
    const extraction = activeExtraction();

    if (!extraction) {
      return "";
    }

    return extraction.spans
      .map((span) => span.text.trim())
      .filter((text) => text.length > 0)
      .join("\n");
  }

  function canUseOcrFallback(reliability: ReadingReliabilityState) {
    return reliability === "native_weak" || reliability === "unavailable";
  }

  function renderReliability(reliability: ReadingReliabilityState, warning: string | null) {
    reliabilityBadge.className = `pdf-reliability-badge ${reliability}`;
    reliabilityBadge.textContent = reliability.replace(/_/g, " ");
    reliabilityCopy.textContent = warning
      ?? "Native text extraction has not produced a warning for this page.";
  }

  function renderReadingPanel() {
    const extraction = activeExtraction();
    const text = currentReadableText();

    readButton.disabled = !state.document || text.length === 0;
    stopButton.disabled = !state.isSpeaking;
    ocrButton.disabled = !state.document || state.isOcrRunning || !canUseOcrFallback(state.nativeExtraction?.reliability ?? "unavailable");
    importCanvasButton.disabled = !state.document || !state.pageImageDataUrl;
    ocrButton.textContent = state.isOcrRunning ? "Running OCR..." : "Run OCR fallback";
    readingStatus.textContent = state.readingStatus;

    if (!extraction) {
      readingSource.textContent = "Source: no extracted text available yet";
      textPreview.textContent = "Open a document to inspect extracted reading text for the current page.";
      return;
    }

    readingSource.textContent = extraction.sourceKind === "ocr"
      ? "Source: OCR fallback text"
      : "Source: native PDF text";
    textPreview.textContent = text.length > 0
      ? text.slice(0, 2400)
      : "No readable text was extracted for this page.";
  }

  function currentAnnotationBounds() {
    const stageRect = stage.getBoundingClientRect();
    const sourceWidth = pageImage.naturalWidth || state.pageRender?.width || DEFAULT_PAGE_FRAME_WIDTH;
    const sourceHeight = pageImage.naturalHeight || state.pageRender?.height || DEFAULT_PAGE_FRAME_HEIGHT;

    if (
      stageRect.width <= 0
      || stageRect.height <= 0
    ) {
      return {
        width: stageRect.width,
        height: stageRect.height,
        offsetLeft: 0,
        offsetTop: 0,
      };
    }

    const scale = Math.min(
      stageRect.width / sourceWidth,
      stageRect.height / sourceHeight,
    );
    const width = sourceWidth * scale;
    const height = sourceHeight * scale;

    return {
      width,
      height,
      offsetLeft: (stageRect.width - width) / 2,
      offsetTop: (stageRect.height - height) / 2,
    };
  }

  function resizeAnnotationLayer() {
    const bounds = currentAnnotationBounds();
    const scale = window.devicePixelRatio || 1;

    pageFrame.style.left = `${bounds.offsetLeft}px`;
    pageFrame.style.top = `${bounds.offsetTop}px`;
    pageFrame.style.width = `${bounds.width}px`;
    pageFrame.style.height = `${bounds.height}px`;
    annotationLayer.width = Math.max(1, Math.floor(bounds.width * scale));
    annotationLayer.height = Math.max(1, Math.floor(bounds.height * scale));
    annotationLayer.hidden = !state.document || !state.pageRender;
    pageFrame.hidden = !state.document || !state.pageRender;

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

  function speechAvailabilityMessage() {
    return "No local speech backend is available in this runtime for Read page.";
  }

  function stopSpeaking() {
    activeSpeechRequestId += 1;

    if (state.activeSpeechBackend === "native") {
      void invoke("stop_local_speech").catch((error) => {
        console.error("Could not stop native speech:", error);
      });
    } else if ("speechSynthesis" in window) {
      window.speechSynthesis.cancel();
    }

    state.isSpeaking = false;
    state.activeSpeechBackend = null;
    state.readingStatus = "Reading stopped.";
    renderReadingPanel();
  }

  function readPdfPreferences() {
    const rawValue = window.localStorage.getItem(PREFERENCES_STORAGE_KEY);

    if (!rawValue) {
      return {
        speechRate: 1,
        recolor: {
          enabled: false,
          foreground: "#c0caf5",
          background: "#1a1b26",
        } satisfies PdfRecolorSettingsDocument,
      };
    }

    try {
      const parsed = JSON.parse(rawValue) as Partial<{
        speechRate: number;
        recolorEnabled: boolean;
        recolorForeground: string;
        recolorBackground: string;
      }>;

      return {
        speechRate: typeof parsed.speechRate === "number" ? parsed.speechRate : 1,
        recolor: {
          enabled: parsed.recolorEnabled ?? false,
          foreground: parsed.recolorForeground ?? "#c0caf5",
          background: parsed.recolorBackground ?? "#1a1b26",
        } satisfies PdfRecolorSettingsDocument,
      };
    } catch (error) {
      console.error("Could not parse PDF preferences:", error);
      return {
        speechRate: 1,
        recolor: {
          enabled: false,
          foreground: "#c0caf5",
          background: "#1a1b26",
        } satisfies PdfRecolorSettingsDocument,
      };
    }
  }

  function readSpeechRatePreference() {
    return readPdfPreferences().speechRate;
  }

  function syncRecolorControls() {
    recolorEnabledInput.checked = state.recolor.enabled;
    recolorForegroundInput.value = state.recolor.foreground;
    recolorBackgroundInput.value = state.recolor.background;
    recolorForegroundSwatch.style.backgroundColor = state.recolor.foreground;
    recolorBackgroundSwatch.style.backgroundColor = state.recolor.background;
    recolorForegroundValue.textContent = state.recolor.foreground.toLowerCase();
    recolorBackgroundValue.textContent = state.recolor.background.toLowerCase();
    recolorControls.classList.toggle("disabled", !state.recolor.enabled);
  }

  function startBrowserSpeech(
    text: string,
    extraction: PageTextExtraction,
    requestId: number,
  ) {
    if (!("speechSynthesis" in window)) {
      state.isSpeaking = false;
      state.activeSpeechBackend = null;
      state.readingStatus = speechAvailabilityMessage();
      renderReadingPanel();
      return false;
    }

    window.speechSynthesis.cancel();
    const utterance = new SpeechSynthesisUtterance(text);
    utterance.rate = speechRate;
    utterance.onstart = () => {
      if (requestId !== activeSpeechRequestId) {
        return;
      }

      state.isSpeaking = true;
      state.activeSpeechBackend = "browser";
      state.readingStatus = extraction.reliability === "native_reliable" || extraction.reliability === "ocr_reliable"
        ? `Reading ${extraction.sourceKind} text aloud in the webview.`
        : `Reading ${extraction.sourceKind} text aloud in the webview with weak reliability.`;
      renderReadingPanel();
    };
    utterance.onend = () => {
      if (requestId !== activeSpeechRequestId) {
        return;
      }

      state.isSpeaking = false;
      state.activeSpeechBackend = null;
      state.readingStatus = "Reading finished.";
      renderReadingPanel();
    };
    utterance.onerror = () => {
      if (requestId !== activeSpeechRequestId) {
        return;
      }

      state.isSpeaking = false;
      state.activeSpeechBackend = null;
      state.readingStatus = "Speech synthesis failed for the current page.";
      renderReadingPanel();
    };
    window.speechSynthesis.speak(utterance);
    return true;
  }

  async function speakCurrentPage() {
    const text = currentReadableText();
    const extraction = activeExtraction();

    if (!text || !extraction) {
      state.readingStatus = "There is no extracted text available to read aloud for this page.";
      renderReadingPanel();
      return;
    }

    stopSpeaking();

    const requestId = ++activeSpeechRequestId;
    state.isSpeaking = true;
    state.activeSpeechBackend = "native";
    state.readingStatus = "Starting local speech for the current page.";
    renderReadingPanel();

    try {
      await invoke("speak_text_locally", {
        request: {
          text,
          rate: speechRate,
        },
      });

      if (requestId !== activeSpeechRequestId) {
        return;
      }

      state.isSpeaking = false;
      state.activeSpeechBackend = null;
      state.readingStatus = "Reading finished.";
      renderReadingPanel();
      return;
    } catch (error) {
      console.error("Native local speech failed:", error);

      if (requestId !== activeSpeechRequestId) {
        return;
      }
    }

    state.readingStatus = "Native local speech was unavailable. Falling back to webview speech if possible.";
    renderReadingPanel();

    if (!startBrowserSpeech(text, extraction, requestId)) {
      state.readingStatus = speechAvailabilityMessage();
      renderReadingPanel();
    }
  }

  function hexToRgb(hexColor: string) {
    const normalized = hexColor.replace("#", "");
    const safe = normalized.length === 3
      ? normalized.split("").map((value) => `${value}${value}`).join("")
      : normalized.padEnd(6, "0").slice(0, 6);

    return {
      r: Number.parseInt(safe.slice(0, 2), 16),
      g: Number.parseInt(safe.slice(2, 4), 16),
      b: Number.parseInt(safe.slice(4, 6), 16),
    };
  }

  async function buildRenderedPageDataUrl() {
    if (!state.pageRender) {
      return null;
    }

    const baseDataUrl = `data:${state.pageRender.mimeType};base64,${state.pageRender.dataBase64}`;

    if (!state.recolor.enabled) {
      return baseDataUrl;
    }

    return new Promise<string>((resolve, reject) => {
      const image = new Image();

      image.onload = () => {
        const canvasElement = document.createElement("canvas");
        canvasElement.width = image.naturalWidth;
        canvasElement.height = image.naturalHeight;
        const context2d = canvasElement.getContext("2d");

        if (!context2d) {
          reject(new Error("Could not get recolor canvas context."));
          return;
        }

        context2d.drawImage(image, 0, 0);
        const imageData = context2d.getImageData(0, 0, canvasElement.width, canvasElement.height);
        const foreground = hexToRgb(state.recolor.foreground);
        const background = hexToRgb(state.recolor.background);

        for (let index = 0; index < imageData.data.length; index += 4) {
          const red = imageData.data[index];
          const green = imageData.data[index + 1];
          const blue = imageData.data[index + 2];
          const alpha = imageData.data[index + 3];
          const intensity = (red + green + blue) / (255 * 3);

          imageData.data[index] = Math.round(foreground.r * (1 - intensity) + background.r * intensity);
          imageData.data[index + 1] = Math.round(foreground.g * (1 - intensity) + background.g * intensity);
          imageData.data[index + 2] = Math.round(foreground.b * (1 - intensity) + background.b * intensity);
          imageData.data[index + 3] = alpha;
        }

        context2d.putImageData(imageData, 0, 0);
        resolve(canvasElement.toDataURL("image/png"));
      };

      image.onerror = () => {
        reject(new Error("Could not load page image for recoloring."));
      };

      image.src = baseDataUrl;
    });
  }

  async function refreshRenderedPageImage() {
    if (!state.pageRender) {
      state.pageImageDataUrl = null;
      return;
    }

    try {
      state.pageImageDataUrl = await buildRenderedPageDataUrl();
    } catch (error) {
      state.pageImageDataUrl = `data:${state.pageRender.mimeType};base64,${state.pageRender.dataBase64}`;
      state.readingStatus = `Recolor failed: ${String(error)}`;
    }
  }

  function renderDocumentState() {
    const document = state.document;
    const hasDocument = Boolean(document);
    const extraction = activeExtraction();

    stageShell.classList.toggle("empty", !hasDocument);
    stageEmpty.hidden = hasDocument;
    stage.hidden = !hasDocument;
    prevButton.disabled = !hasDocument || state.pageIndex <= 0;
    nextButton.disabled = !hasDocument || (document?.pageCount !== null && document !== null && state.pageIndex >= document.pageCount - 1);
    refreshButton.disabled = !hasDocument;
    syncRecolorControls();

    if (!document) {
      documentTitle.textContent = "PDF Mode";
      pageCountCopy.textContent = "Page count unknown";
      pageLabel.textContent = "No document open";
      renderBackendNotes(state.backendStatus?.notes ?? bootstrap?.activePdfBackend.notes ?? []);
      renderReliability("unavailable", "Open a document to inspect native text extraction status for the current page.");
      pageFrame.hidden = true;
      pageImage.hidden = true;
      annotationLayer.hidden = true;
      renderPlaceholder.hidden = false;
      renderPlaceholder.textContent = "The PDF page stage is ready, but no document is open yet.";
      resizeAnnotationLayer();
      redrawAnnotations();
      renderReadingPanel();
      return;
    }

    documentTitle.textContent = document.documentName;
    pageCountCopy.textContent = document.pageCount === null
      ? "Page count not reported by the current backend"
      : `${document.pageCount} pages detected`;
    pageLabel.textContent = `Page ${state.pageIndex + 1}${document.pageCount ? ` of ${document.pageCount}` : ""}`;

    renderReliability(
      extraction?.reliability ?? "unavailable",
      state.pageError ?? extraction?.warning ?? null,
    );

    const notes = document.notes.length > 0
      ? document.notes
      : ["The PDF backend did not return additional notes for this document."];

    renderBackendNotes(notes);

    if (state.pageRender) {
      pageFrame.hidden = false;
      pageImage.hidden = false;
      pageImage.src = state.pageImageDataUrl ?? `data:${state.pageRender.mimeType};base64,${state.pageRender.dataBase64}`;
      annotationLayer.hidden = false;
      renderPlaceholder.hidden = true;
    } else {
      pageFrame.hidden = true;
      pageImage.hidden = true;
      annotationLayer.hidden = true;
      renderPlaceholder.hidden = false;
      renderPlaceholder.textContent = state.pageError
        ?? "The PDF engine boundary is connected, but page rendering is not configured on this machine yet.";
    }

    resizeAnnotationLayer();
    renderReadingPanel();
  }

  async function extractNativeText(documentPath: string, pageIndex: number) {
    try {
      state.nativeExtraction = await invoke<PageTextExtraction>("extract_pdf_page_text", {
        request: {
          documentPath,
          pageIndex,
        } satisfies ExtractPdfTextRequest,
      });

      state.readingStatus = state.nativeExtraction.reliability === "native_reliable"
        ? "Native PDF text is ready for local reading."
        : "Native PDF text is weak or unavailable. OCR fallback is available if needed.";
    } catch (error) {
      state.nativeExtraction = {
        pageIndex,
        sourceKind: "native",
        reliability: "unavailable",
        warning: String(error),
        spans: [],
      };
      state.readingStatus = "Native text extraction failed for this page.";
    }
  }

  async function refreshPageState() {
    const document = state.document;

    if (!document) {
      return;
    }

    stopSpeaking();
    state.pageRender = null;
    state.pageImageDataUrl = null;
    state.pageError = null;
    state.nativeExtraction = null;
    state.ocrExtraction = null;
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

    await refreshRenderedPageImage();

    await extractNativeText(document.documentPath, state.pageIndex);
    renderDocumentState();
  }

  async function runOcrFallback() {
    const document = state.document;
    const nativeExtraction = state.nativeExtraction;

    if (!document || !nativeExtraction || !canUseOcrFallback(nativeExtraction.reliability)) {
      return;
    }

    state.isOcrRunning = true;
    state.readingStatus = "Running local OCR fallback for the current page.";
    renderReadingPanel();

    try {
      state.ocrExtraction = await invoke<PageTextExtraction>("extract_pdf_page_ocr", {
        request: {
          documentPath: document.documentPath,
          pageIndex: state.pageIndex,
        } satisfies ExtractPdfTextRequest,
      });

      state.readingStatus = state.ocrExtraction.reliability === "ocr_reliable"
        ? "OCR fallback recovered readable text for this page."
        : "OCR fallback completed, but the result is still weak. Treat follow-along as untrusted.";
    } catch (error) {
      state.ocrExtraction = {
        pageIndex: state.pageIndex,
        sourceKind: "ocr",
        reliability: "unavailable",
        warning: String(error),
        spans: [],
      };
      state.readingStatus = "OCR fallback failed for the current page.";
    } finally {
      state.isOcrRunning = false;
      renderDocumentState();
    }
  }

  function importCurrentPageToCanvas() {
    if (!state.document || !state.pageRender || !state.pageImageDataUrl) {
      return;
    }

    window.dispatchEvent(new CustomEvent("rpdf:request-pdf-page-import", {
      detail: {
        sourcePdfPath: state.document.documentPath,
        pageIndex: state.pageIndex,
        assetPath: state.pageImageDataUrl,
        width: state.pageRender.width,
        height: state.pageRender.height,
        recolor: state.recolor,
      },
    }));

    state.readingStatus = "Imported the current PDF page into Canvas Mode.";
    renderReadingPanel();
  }

  async function openDocument(requestedPath?: string) {
    const documentPath = (requestedPath ?? pathInput.value).trim();

    if (!documentPath) {
      setOpenError("Enter a PDF path first.");
      return;
    }

    pathInput.value = documentPath;
    stopSpeaking();
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
      pushRecentPdfPath(documentPath);
      backendName.textContent = `${bootstrap?.activePdfBackend.backendName ?? "PDF backend"}: ${state.document.backendReady ? "ready" : "boundary only"}`;
      await refreshPageState();
    } catch (error) {
      state.document = null;
      state.pageIndex = 0;
      state.pageRender = null;
      state.pageError = null;
      state.nativeExtraction = null;
      state.ocrExtraction = null;
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
  readButton.addEventListener("click", () => {
    void speakCurrentPage();
  });
  stopButton.addEventListener("click", () => {
    stopSpeaking();
  });
  ocrButton.addEventListener("click", () => {
    void runOcrFallback();
  });
  importCanvasButton.addEventListener("click", () => {
    importCurrentPageToCanvas();
  });
  recolorEnabledInput.addEventListener("change", () => {
    state.recolor = {
      ...state.recolor,
      enabled: recolorEnabledInput.checked,
    };
    void refreshRenderedPageImage().then(() => {
      renderDocumentState();
    });
  });
  recolorForegroundInput.addEventListener("input", () => {
    state.recolor = {
      ...state.recolor,
      foreground: recolorForegroundInput.value,
    };
    void refreshRenderedPageImage().then(() => {
      renderDocumentState();
    });
  });
  recolorBackgroundInput.addEventListener("input", () => {
    state.recolor = {
      ...state.recolor,
      background: recolorBackgroundInput.value,
    };
    void refreshRenderedPageImage().then(() => {
      renderDocumentState();
    });
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
  pageImage.addEventListener("load", resizeAnnotationLayer);
  window.addEventListener("resize", resizeAnnotationLayer);
  window.addEventListener("rpdf:preferences-changed", onPreferencesChanged as EventListener);

  backendName.textContent = `${bootstrap?.activePdfBackend.backendName ?? "Backend unavailable"}: ${bootstrap?.activePdfBackend.configured ? "ready" : "boundary only"}`;
  renderBackendNotes(bootstrap?.activePdfBackend.notes ?? []);
  renderRecentPdfPaths();
  renderDocumentState();

  function onPreferencesChanged(event: CustomEvent<{
    speechRate?: number;
    recolorEnabled?: boolean;
    recolorForeground?: string;
    recolorBackground?: string;
  }>) {
    if (typeof event.detail.speechRate === "number") {
      speechRate = event.detail.speechRate;
    }

    state.recolor = {
      enabled: event.detail.recolorEnabled ?? state.recolor.enabled,
      foreground: event.detail.recolorForeground ?? state.recolor.foreground,
      background: event.detail.recolorBackground ?? state.recolor.background,
    };
    syncRecolorControls();
    void refreshRenderedPageImage().then(() => {
      renderDocumentState();
    });
  }

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

      const readingCache = [state.nativeExtraction, state.ocrExtraction]
        .filter((entry): entry is PageTextExtraction => entry !== null)
        .map((entry) => ({
          pageIndex: entry.pageIndex,
          reliability: entry.reliability,
          sourceKind: entry.sourceKind,
          cacheKey: null,
        }));

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
        recolor: state.recolor,
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

      stopSpeaking();
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
      state.pageImageDataUrl = null;
      state.pageError = null;
      state.nativeExtraction = null;
      state.ocrExtraction = null;
      state.recolor = snapshot.document.recolor;
      pathInput.value = snapshot.document.sourcePdfPath;
      pushRecentPdfPath(snapshot.document.sourcePdfPath);
      await refreshPageState();
      redrawAnnotations();
    },
    destroy() {
      stopSpeaking();
      window.removeEventListener("resize", resizeAnnotationLayer);
      annotationLayer.removeEventListener("pointerdown", onPointerDown);
      annotationLayer.removeEventListener("pointermove", onPointerMove);
      annotationLayer.removeEventListener("pointerup", onPointerUp);
      annotationLayer.removeEventListener("pointercancel", onPointerUp);
      pageImage.removeEventListener("load", resizeAnnotationLayer);
      window.removeEventListener("rpdf:preferences-changed", onPreferencesChanged as EventListener);
      container.replaceChildren();
    },
  };
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
