export type AppMode = "canvas" | "pdf";

export type ReadingReliabilityState =
  | "native_reliable"
  | "native_weak"
  | "ocr_reliable"
  | "ocr_weak"
  | "unavailable";

export type PdfBackendStatus = {
  backendKey: string;
  backendName: string;
  configured: boolean;
  notes: string[];
};

export type AppThemeConfig = {
  bg: string;
  bgDark: string;
  bgDarker: string;
  bgHighlight: string;
  bgPanel: string;
  fg: string;
  fgDark: string;
  fgGutter: string;
  blue: string;
  cyan: string;
  green: string;
  yellow: string;
  orange: string;
  red: string;
  magenta: string;
  purple: string;
};

export type AppToolShortcutsConfig = {
  select: string;
  pan: string;
  pen: string;
  rectangle: string;
  ellipse: string;
  line: string;
  arrow: string;
  text: string;
  eraser: string;
};

export type AppColorShortcutsConfig = {
  fg: string;
  blue: string;
  cyan: string;
  green: string;
  yellow: string;
  orange: string;
  red: string;
  magenta: string;
  purple: string;
};

export type AppConfig = {
  version: number;
  theme: AppThemeConfig;
  canvas: {
    backgroundPattern: CanvasBackgroundPattern;
  };
  shortcuts: {
    tools: AppToolShortcutsConfig;
    colors: AppColorShortcutsConfig;
  };
};

export type AppBootstrap = {
  supportedModes: AppMode[];
  activePdfBackend: PdfBackendStatus;
  reliabilityStates: ReadingReliabilityState[];
  appConfig: AppConfig;
  appConfigPath: string;
  appConfigWarnings: string[];
};

export type DocumentVersion = {
  major: number;
  minor: number;
};

export type CanvasBackgroundPattern = "dots" | "lines" | "squares" | "none";

export type CanvasPointDocument = {
  x: number;
  y: number;
  pressure: number;
};

export type CanvasStrokeDocument = {
  id?: string;
  color: string;
  width: number;
  order?: number;
  points: CanvasPointDocument[];
};

export type CanvasShapeKindDocument = "line" | "arrow" | "rectangle" | "ellipse";

export type CanvasShapeDocument = {
  id: string;
  kind: CanvasShapeKindDocument;
  color: string;
  width: number;
  order?: number;
  start: {
    x: number;
    y: number;
  };
  end: {
    x: number;
    y: number;
  };
};

export type CanvasTextDocument = {
  id: string;
  text: string;
  color: string;
  fontSize: number;
  order?: number;
  x: number;
  y: number;
};

export type CanvasImagePlacementDocument = {
  id: string;
  assetPath: string;
  x: number;
  y: number;
  width: number;
  height: number;
};

export type CanvasPdfPagePlacementDocument = {
  id: string;
  sourcePdfPath: string;
  pageIndex: number;
  assetPath: string;
  x: number;
  y: number;
  width: number;
  height: number;
  recolor: PdfRecolorSettingsDocument;
};

export type CanvasDocument = {
  version: DocumentVersion;
  id: string;
  backgroundPattern: CanvasBackgroundPattern;
  strokes: CanvasStrokeDocument[];
  shapes: CanvasShapeDocument[];
  texts: CanvasTextDocument[];
  images: CanvasImagePlacementDocument[];
  pdfPages: CanvasPdfPagePlacementDocument[];
};

export type CanvasSelectionTargetDocument = {
  kind: "stroke" | "shape" | "text" | "image" | "pdf_page";
  id: string;
};

export type CanvasSelectionDocument =
  | {
    targets: CanvasSelectionTargetDocument[];
  }
  | CanvasSelectionTargetDocument;

export type OpenPdfDocumentRequest = {
  documentPath: string;
};

export type OpenPdfDocumentResponse = {
  documentPath: string;
  documentName: string;
  pageCount: number | null;
  backendReady: boolean;
  notes: string[];
};

export type RenderPdfPageRequest = {
  documentPath: string;
  pageIndex: number;
  targetWidth?: number | null;
  targetHeight?: number | null;
};

export type RenderPdfPageResponse = {
  pageIndex: number;
  mimeType: string;
  dataBase64: string;
  width: number;
  height: number;
};

export type ExtractPdfTextRequest = {
  documentPath: string;
  pageIndex: number;
};

export type TextSpan = {
  text: string;
  x: number;
  y: number;
  width: number;
  height: number;
};

export type PageTextExtraction = {
  pageIndex: number;
  sourceKind: "native" | "ocr";
  reliability: ReadingReliabilityState;
  warning: string | null;
  spans: TextSpan[];
};

export type PdfPointDocument = {
  x: number;
  y: number;
};

export type PdfStrokeAnnotationDocument = {
  color: string;
  width: number;
  points: PdfPointDocument[];
};

export type PdfTextNoteDocument = {
  text: string;
  x: number;
  y: number;
};

export type PdfPageAnnotationLayerDocument = {
  pageIndex: number;
  strokes: PdfStrokeAnnotationDocument[];
  notes: PdfTextNoteDocument[];
};

export type PdfRecolorSettingsDocument = {
  enabled: boolean;
  foreground: string;
  background: string;
};

export type PdfPageReadingCacheDocument = {
  pageIndex: number;
  reliability: ReadingReliabilityState;
  sourceKind: "native" | "ocr";
  cacheKey: string | null;
};

export type PdfStudyDocument = {
  version: DocumentVersion;
  id: string;
  sourcePdfPath: string;
  pageCount: number | null;
  currentPageIndex: number;
  annotations: PdfPageAnnotationLayerDocument[];
  recolor: PdfRecolorSettingsDocument;
  readingCache: PdfPageReadingCacheDocument[];
};

export type WorkspaceDocumentSnapshot =
  | {
    kind: "canvas";
    document: CanvasDocument;
    selection?: CanvasSelectionDocument | null;
  }
  | {
    kind: "pdf";
    document: PdfStudyDocument;
  };

export type WorkspaceController = {
  destroy(): void;
  exportDocument(): WorkspaceDocumentSnapshot;
  importDocument(snapshot: WorkspaceDocumentSnapshot): Promise<void>;
};
