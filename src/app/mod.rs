mod canvas;
mod pdf;
mod util;

use crate::model::{
    AnnotationAppearanceSet, AnnotationLayerRole, AnnotationPalette, AnnotationVisibility,
    AutosaveState, BackgroundPattern, BackgroundPatternStyle, BrushStyle, CanvasDocument, CanvasItem,
    DocumentMetadata, HighlightMode, ImportedAssetSource, ImportedImageItem, ImportedPdfPageItem,
    PdfDocumentSession, PdfPenStrokeAnnotation, PdfSource, PdfTextNote, PdfViewState, PdfViewportState,
    PenStrokeItem, PenToolKind, PlaybackState, Point, ReadingReliability, ReadingSupportState, Rect,
    RecolorExportMode, RgbaColor, SelectionTarget, Size, StrokePoint, TextItem, TextStyle,
    TextSupportSource, TtsState, ViewportState, WarningCode, WorkspaceMode,
};
use eframe::egui;
use std::time::Instant;
use util::{
    default_palette, muted_grid_color,
};

pub struct RpdfApp {
    startup: StartupState,
    shell: ShellState,
}

impl Default for RpdfApp {
    fn default() -> Self {
        Self {
            startup: StartupState::default(),
            shell: ShellState::default(),
        }
    }
}

impl eframe::App for RpdfApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick_pdf_reading_support();

        egui::TopBottomPanel::top("mode_switcher").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("rpdf");
                ui.separator();
                ui.label("Workspace:");
                ui.selectable_value(
                    &mut self.shell.mode,
                    WorkspaceMode::InfiniteCanvas,
                    "Infinite Canvas",
                );
                ui.selectable_value(
                    &mut self.shell.mode,
                    WorkspaceMode::PdfDocument,
                    "PDF Mode",
                );
                ui.separator();
                ui.label(format!("Offline startup: {}", self.startup.offline_ready));
            });
        });

        egui::SidePanel::left("session_summary")
            .resizable(false)
            .default_width(220.0)
            .show(ctx, |ui| {
                ui.heading("Session");
                ui.label(format!("Default mode: {:?}", self.startup.default_mode));
                ui.label(format!(
                    "Last opened path: {}",
                    self.startup.last_opened_path.as_deref().unwrap_or("none")
                ));
                ui.separator();

                match self.shell.mode {
                    WorkspaceMode::InfiniteCanvas => self.render_canvas_summary(ui),
                    WorkspaceMode::PdfDocument => self.render_pdf_summary(ui),
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| match self.shell.mode {
            WorkspaceMode::InfiniteCanvas => self.render_canvas_workspace(ui),
            WorkspaceMode::PdfDocument => self.render_pdf_workspace(ui),
        });
    }
}

impl RpdfApp {
    fn render_canvas_summary(&mut self, ui: &mut egui::Ui) {
        ui.heading("Canvas Root");
        ui.label(format!("Items: {}", self.shell.canvas.items.len()));
        ui.label(format!("Zoom: {:.2}", self.shell.canvas.viewport.zoom));
        ui.label(format!(
            "Origin: ({:.0}, {:.0})",
            self.shell.canvas.viewport.origin.x, self.shell.canvas.viewport.origin.y
        ));
        ui.label(format!("Selection: {:?}", self.shell.canvas.selection));
        ui.label(format!("Dirty: {}", self.shell.canvas.autosave.dirty));
        ui.label(format!(
            "Active stroke points: {}",
            self.shell
                .canvas_interaction
                .active_stroke
                .as_ref()
                .map_or(0, |stroke| stroke.points.len())
        ));
        ui.label(format!("Tool: {:?}", self.shell.annotation_tools.current_tool));
        ui.label(format!("Export target: {:?}", self.shell.canvas.selection));
    }

    fn render_pdf_summary(&self, ui: &mut egui::Ui) {
        ui.heading("PDF Root");
        ui.label(format!("Page: {}", self.shell.pdf.viewport.page_index + 1));
        ui.label(format!("Page count: {}", self.shell.pdf_interaction.page_count));
        ui.label(format!("Zoom: {:.2}", self.shell.pdf.viewport.zoom));
        ui.label(format!(
            "Text source: {:?}",
            self.shell.pdf.reading_support.text_source
        ));
        ui.label(format!(
            "Reading reliability: {:?}",
            self.shell.pdf.reading_support.reliability
        ));
        ui.label(format!("Tool: {:?}", self.shell.annotation_tools.current_tool));
        ui.label(format!("PDF annotations: {}", self.shell.pdf.annotations.len()));
    }

    fn render_annotation_toolbar(&mut self, ui: &mut egui::Ui, mode: WorkspaceMode) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Annotation tool:");
                ui.selectable_value(
                    &mut self.shell.annotation_tools.current_tool,
                    AnnotationTool::Ink,
                    "Ink",
                );
                ui.selectable_value(
                    &mut self.shell.annotation_tools.current_tool,
                    AnnotationTool::Highlighter,
                    "Highlighter",
                );
            });

            ui.horizontal(|ui| {
                ui.label("Note:");
                ui.text_edit_singleline(&mut self.shell.annotation_tools.pending_note_text);
                if ui.button("Add note").clicked() {
                    match mode {
                        WorkspaceMode::InfiniteCanvas => self.add_canvas_note(),
                        WorkspaceMode::PdfDocument => self.add_pdf_note(),
                    }
                }
            });
        });
    }
}

#[derive(Debug, Clone)]
pub struct StartupState {
    pub default_mode: WorkspaceMode,
    pub last_opened_path: Option<String>,
    pub offline_ready: bool,
}

impl Default for StartupState {
    fn default() -> Self {
        Self {
            default_mode: WorkspaceMode::InfiniteCanvas,
            last_opened_path: None,
            offline_ready: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShellState {
    pub mode: WorkspaceMode,
    pub canvas: CanvasDocument,
    pub canvas_interaction: CanvasInteractionState,
    pub annotation_tools: AnnotationToolState,
    pub pdf: PdfDocumentSession,
    pub pdf_interaction: PdfInteractionState,
}

impl Default for ShellState {
    fn default() -> Self {
        Self {
            mode: WorkspaceMode::InfiniteCanvas,
            canvas: default_canvas_document(),
            canvas_interaction: CanvasInteractionState::default(),
            annotation_tools: AnnotationToolState::default(),
            pdf: default_pdf_session(),
            pdf_interaction: PdfInteractionState::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CanvasInteractionState {
    pub active_stroke: Option<PendingStroke>,
    pub next_stroke_id: u64,
    pub next_item_id: u64,
    pub pending_text: String,
    pub pending_image_path: String,
    pub pending_pdf_path: String,
    pub pending_pdf_page: usize,
    pub export_path: String,
    pub export_status: String,
}

#[derive(Debug, Clone)]
pub struct PendingStroke {
    pub points: Vec<StrokePoint>,
}

#[derive(Debug, Clone)]
pub struct PdfInteractionState {
    pub pending_open_path: String,
    pub status_message: String,
    pub page_count: usize,
    pub active_stroke: Option<PendingStroke>,
    pub next_annotation_id: u64,
    pub reading_session: Option<ReadingPlaybackSession>,
}

impl Default for PdfInteractionState {
    fn default() -> Self {
        Self {
            pending_open_path: String::new(),
            status_message: String::new(),
            page_count: 1,
            active_stroke: None,
            next_annotation_id: 0,
            reading_session: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReadingPlaybackSession {
    pub spans: Vec<String>,
    pub started_at: Instant,
    pub span_duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationTool {
    Ink,
    Highlighter,
}

#[derive(Debug, Clone)]
pub struct AnnotationToolState {
    pub current_tool: AnnotationTool,
    pub pending_note_text: String,
}

impl Default for AnnotationToolState {
    fn default() -> Self {
        Self {
            current_tool: AnnotationTool::Ink,
            pending_note_text: String::new(),
        }
    }
}

fn default_canvas_document() -> CanvasDocument {
    CanvasDocument {
        metadata: DocumentMetadata {
            document_id: "canvas-default".to_string(),
            title: Some("Untitled canvas".to_string()),
            created_unix_ms: 0,
            modified_unix_ms: 0,
        },
        viewport: ViewportState {
            origin: Point { x: 0.0, y: 0.0 },
            zoom: 1.0,
        },
        background: BackgroundPattern::Dots(BackgroundPatternStyle {
            spacing: 24.0,
            line_width: 1.0,
            color: muted_grid_color(),
        }),
        items: Vec::new(),
        selection: SelectionTarget::WholeCanvas,
        autosave: AutosaveState {
            recovery_snapshot_id: None,
            dirty: false,
        },
    }
}

fn default_pdf_session() -> PdfDocumentSession {
    PdfDocumentSession {
        metadata: DocumentMetadata {
            document_id: "pdf-default".to_string(),
            title: Some("No PDF opened".to_string()),
            created_unix_ms: 0,
            modified_unix_ms: 0,
        },
        source: PdfSource::FilePath("".into()),
        viewport: PdfViewportState {
            page_index: 0,
            scroll_offset: Point { x: 0.0, y: 0.0 },
            zoom: 1.0,
        },
        annotations: Vec::new(),
        reading_support: ReadingSupportState {
            tts: TtsState {
                playback: PlaybackState::Stopped,
                active_span: None,
            },
            highlight_mode: HighlightMode::Line,
            text_source: TextSupportSource::Unavailable,
            reliability: ReadingReliability::BestEffort,
            warning: Some(crate::model::UserVisibleWarning {
                code: WarningCode::ReadingSupportUnavailable,
                message:
                    "PDF reading support will appear here after document loading is implemented."
                        .to_string(),
            }),
        },
        view: PdfViewState {
            recolor: crate::model::RecolorState {
                current_profile: None,
                export_mode: RecolorExportMode::PreserveOriginalAppearance,
            },
            annotation_visibility: AnnotationVisibility {
                normal_view: default_palette(),
                recolored_view: default_palette(),
            },
        },
        autosave: AutosaveState {
            recovery_snapshot_id: None,
            dirty: false,
        },
    }
}
