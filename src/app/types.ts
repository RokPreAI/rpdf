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

export type WorkspaceController = {
  destroy(): void;
};
