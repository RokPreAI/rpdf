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

export type AppBootstrap = {
  supportedModes: AppMode[];
  activePdfBackend: PdfBackendStatus;
  reliabilityStates: ReadingReliabilityState[];
};

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
  reliability: ReadingReliabilityState;
  warning: string | null;
  spans: TextSpan[];
};

export type WorkspaceController = {
  destroy(): void;
};
