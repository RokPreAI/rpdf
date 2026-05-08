mod canvas;
mod pdf;
mod services;
mod util;

use crate::model::{
    AnnotationAppearanceSet, AnnotationLayerRole, AnnotationPalette, AnnotationVisibility,
    AutosaveState, BackgroundPattern, BackgroundPatternStyle, BrushStyle, CanvasDocument,
    CanvasItem, DocumentMetadata, HighlightMode, ImportedAssetSource, ImportedImageItem,
    ImportedPdfPageItem, PdfDocumentSession, PdfPenStrokeAnnotation, PdfSource, PdfTextNote,
    PdfViewState, PdfViewportState, PenStrokeItem, PenToolKind, PlaybackState, Point,
    ReadingReliability, ReadingSupportState, RecolorExportMode, Rect, RgbaColor, SelectionTarget,
    Size, StrokePoint, TextItem, TextStyle, TextSupportSource, TtsState, ViewportState,
    WarningCode, WorkspaceMode,
};
use eframe::egui;
use services::AppServices;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use util::{default_palette, muted_grid_color};

pub struct RpdfApp {
    startup: StartupState,
    shell: ShellState,
    services: AppServices,
}

impl Default for RpdfApp {
    fn default() -> Self {
        Self {
            startup: StartupState::default(),
            shell: ShellState::default(),
            services: AppServices::default(),
        }
    }
}

impl eframe::App for RpdfApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick_pdf_reading_support();
        self.tick_autosave();

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
                ui.selectable_value(&mut self.shell.mode, WorkspaceMode::PdfDocument, "PDF Mode");
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
        ui.label(
            self.shell
                .canvas_mode
                .document
                .metadata
                .title
                .as_deref()
                .unwrap_or("Untitled canvas"),
        );
        ui.label(format!(
            "Items: {}",
            self.shell.canvas_mode.document.items.len()
        ));
        ui.label(format!(
            "Zoom: {:.2}",
            self.shell.canvas_mode.document.viewport.zoom
        ));
        ui.label(format!(
            "Origin: ({:.0}, {:.0})",
            self.shell.canvas_mode.document.viewport.origin.x,
            self.shell.canvas_mode.document.viewport.origin.y
        ));
        ui.label(format!(
            "Selection: {:?}",
            self.shell.canvas_mode.document.selection
        ));
        ui.label(format!(
            "Dirty: {}",
            self.shell.canvas_mode.document.autosave.dirty
        ));
        ui.label(format!(
            "Recovery snapshot: {}",
            if self.services.persistence.has_canvas_recovery_snapshot() {
                "available"
            } else {
                "none"
            }
        ));
        ui.label(format!(
            "Active stroke points: {}",
            self.shell
                .canvas_mode
                .ui
                .active_stroke
                .as_ref()
                .map_or(0, |stroke| stroke.points.len())
        ));
        ui.label(format!(
            "Tool: {:?}",
            self.shell.shared_ui.annotation_tools.current_tool
        ));
        ui.label(format!(
            "Export target: {:?}",
            self.shell.canvas_mode.document.selection
        ));
    }

    fn render_pdf_summary(&self, ui: &mut egui::Ui) {
        ui.heading("PDF Root");
        ui.label(
            self.shell
                .pdf_mode
                .session
                .metadata
                .title
                .as_deref()
                .unwrap_or("No PDF opened"),
        );
        ui.label(format!(
            "Page: {}",
            self.shell.pdf_mode.session.viewport.page_index + 1
        ));
        ui.label(format!("Page count: {}", self.shell.pdf_mode.ui.page_count));
        ui.label(format!(
            "Zoom: {:.2}",
            self.shell.pdf_mode.session.viewport.zoom
        ));
        ui.label(format!(
            "Text source: {:?}",
            self.shell.pdf_mode.session.reading_support.text_source
        ));
        ui.label(format!(
            "Reading reliability: {:?}",
            self.shell.pdf_mode.session.reading_support.reliability
        ));
        ui.label(format!(
            "Tool: {:?}",
            self.shell.shared_ui.annotation_tools.current_tool
        ));
        ui.label(format!(
            "PDF annotations: {}",
            self.shell.pdf_mode.session.annotations.len()
        ));
        ui.label(format!(
            "Dirty: {}",
            self.shell.pdf_mode.session.autosave.dirty
        ));
        ui.label(format!(
            "Recovery snapshot: {}",
            if self.services.persistence.has_pdf_recovery_snapshot() {
                "available"
            } else {
                "none"
            }
        ));
    }

    fn render_annotation_toolbar(&mut self, ui: &mut egui::Ui, mode: WorkspaceMode) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Annotation tool:");
                ui.selectable_value(
                    &mut self.shell.shared_ui.annotation_tools.current_tool,
                    AnnotationTool::Ink,
                    "Ink",
                );
                ui.selectable_value(
                    &mut self.shell.shared_ui.annotation_tools.current_tool,
                    AnnotationTool::Highlighter,
                    "Highlighter",
                );
                ui.selectable_value(
                    &mut self.shell.shared_ui.annotation_tools.current_tool,
                    AnnotationTool::Selection,
                    "Selection",
                );
                ui.selectable_value(
                    &mut self.shell.shared_ui.annotation_tools.current_tool,
                    AnnotationTool::Eraser,
                    "Eraser",
                );
            });

            ui.horizontal(|ui| {
                ui.label("Note:");
                ui.text_edit_singleline(
                    &mut self.shell.shared_ui.annotation_tools.pending_note_text,
                );
                if ui.button("Add note").clicked() {
                    match mode {
                        WorkspaceMode::InfiniteCanvas => self.add_canvas_note(),
                        WorkspaceMode::PdfDocument => self.add_pdf_note(),
                    }
                }
            });
        });
    }

    fn render_status_banner(&self, ui: &mut egui::Ui, tone: BannerTone, title: &str, body: &str) {
        let (fill, stroke, text) = match tone {
            BannerTone::Info => (
                egui::Color32::from_rgb(31, 44, 64),
                egui::Color32::from_rgb(92, 132, 186),
                egui::Color32::from_rgb(229, 239, 252),
            ),
            BannerTone::Success => (
                egui::Color32::from_rgb(29, 54, 44),
                egui::Color32::from_rgb(96, 170, 132),
                egui::Color32::from_rgb(229, 250, 238),
            ),
            BannerTone::Warning => (
                egui::Color32::from_rgb(71, 53, 24),
                egui::Color32::from_rgb(214, 172, 87),
                egui::Color32::from_rgb(255, 244, 220),
            ),
        };

        egui::Frame::group(ui.style())
            .fill(fill)
            .stroke(egui::Stroke::new(1.0, stroke))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.label(egui::RichText::new(title).strong().color(text));
                ui.label(egui::RichText::new(body).color(text));
            });
    }

    fn render_feedback_message(&self, ui: &mut egui::Ui, message: &str) {
        if message.is_empty() {
            return;
        }

        let lowercase = message.to_ascii_lowercase();
        let tone = if lowercase.contains("failed")
            || lowercase.contains("does not exist")
            || lowercase.contains("unavailable")
            || lowercase.contains("no ")
            || lowercase.contains("needs ")
            || lowercase.contains("canceled")
        {
            BannerTone::Warning
        } else if lowercase.contains("saved")
            || lowercase.contains("loaded")
            || lowercase.contains("recovered")
            || lowercase.contains("opened")
            || lowercase.contains("exported")
        {
            BannerTone::Success
        } else {
            BannerTone::Info
        };

        self.render_status_banner(ui, tone, "Status", message);
    }

    fn render_autosave_banner(
        &self,
        ui: &mut egui::Ui,
        dirty: bool,
        has_recovery_snapshot: bool,
        subject: &str,
    ) {
        let (tone, body) = if dirty {
            (
                BannerTone::Warning,
                format!(
                    "{subject} has unsaved changes. Autosave snapshots update in the background every few seconds."
                ),
            )
        } else if has_recovery_snapshot {
            (
                BannerTone::Success,
                format!(
                    "{subject} is clean, and a recovery snapshot is available if the session is interrupted."
                ),
            )
        } else {
            (
                BannerTone::Info,
                format!(
                    "{subject} is clean. Save a session or make a change to create a recoverable snapshot."
                ),
            )
        };

        self.render_status_banner(ui, tone, "Autosave", &body);
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
    pub canvas_mode: CanvasModeState,
    pub pdf_mode: PdfModeState,
    pub shared_ui: SharedUiState,
}

impl Default for ShellState {
    fn default() -> Self {
        Self {
            mode: WorkspaceMode::InfiniteCanvas,
            canvas_mode: CanvasModeState::default(),
            pdf_mode: PdfModeState::default(),
            shared_ui: SharedUiState::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CanvasModeState {
    pub document: CanvasDocument,
    pub ui: CanvasInteractionState,
}

impl Default for CanvasModeState {
    fn default() -> Self {
        Self {
            document: default_canvas_document(),
            ui: CanvasInteractionState::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PdfModeState {
    pub session: PdfDocumentSession,
    pub ui: PdfInteractionState,
}

impl Default for PdfModeState {
    fn default() -> Self {
        Self {
            session: default_pdf_session(),
            ui: PdfInteractionState::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SharedUiState {
    pub annotation_tools: AnnotationToolState,
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
    pub document_path: String,
    pub save_status: String,
    pub export_path: String,
    pub export_status: String,
    pub last_autosave_unix_ms: u64,
}

#[derive(Debug, Clone)]
pub struct PendingStroke {
    pub points: Vec<StrokePoint>,
}

#[derive(Debug, Clone)]
pub struct PdfInteractionState {
    pub pending_open_path: String,
    pub document_path: String,
    pub status_message: String,
    pub page_count: usize,
    pub active_stroke: Option<PendingStroke>,
    pub selected_annotation_id: Option<String>,
    pub next_annotation_id: u64,
    pub reading_session: Option<ReadingPlaybackSession>,
    pub last_autosave_unix_ms: u64,
}

impl Default for PdfInteractionState {
    fn default() -> Self {
        Self {
            pending_open_path: String::new(),
            document_path: String::new(),
            status_message: String::new(),
            page_count: 1,
            active_stroke: None,
            selected_annotation_id: None,
            next_annotation_id: 0,
            reading_session: None,
            last_autosave_unix_ms: 0,
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
    Selection,
    Eraser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerTone {
    Info,
    Success,
    Warning,
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
    let now = current_unix_ms();
    CanvasDocument {
        metadata: DocumentMetadata {
            document_id: "canvas-default".to_string(),
            title: Some("Untitled canvas".to_string()),
            created_unix_ms: now,
            modified_unix_ms: now,
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
    let now = current_unix_ms();
    PdfDocumentSession {
        metadata: DocumentMetadata {
            document_id: "pdf-default".to_string(),
            title: Some("No PDF opened".to_string()),
            created_unix_ms: now,
            modified_unix_ms: now,
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

impl RpdfApp {
    fn tick_autosave(&mut self) {
        const AUTOSAVE_INTERVAL_MS: u64 = 2_000;
        let now = current_unix_ms();

        if self.shell.canvas_mode.document.autosave.dirty
            && now.saturating_sub(self.shell.canvas_mode.ui.last_autosave_unix_ms)
                >= AUTOSAVE_INTERVAL_MS
            && let Ok(snapshot_path) = self
                .services
                .persistence
                .write_canvas_recovery_snapshot(&self.shell.canvas_mode.document)
        {
            self.shell
                .canvas_mode
                .document
                .autosave
                .recovery_snapshot_id = Some(snapshot_path);
            self.shell.canvas_mode.ui.last_autosave_unix_ms = now;
        }

        if self.shell.pdf_mode.session.autosave.dirty
            && now.saturating_sub(self.shell.pdf_mode.ui.last_autosave_unix_ms)
                >= AUTOSAVE_INTERVAL_MS
            && let Ok(snapshot_path) = self
                .services
                .persistence
                .write_pdf_recovery_snapshot(&self.shell.pdf_mode.session)
        {
            self.shell.pdf_mode.session.autosave.recovery_snapshot_id = Some(snapshot_path);
            self.shell.pdf_mode.ui.last_autosave_unix_ms = now;
        }
    }

    fn mark_canvas_dirty(&mut self) {
        self.shell.canvas_mode.document.metadata.modified_unix_ms = current_unix_ms();
        self.shell.canvas_mode.document.autosave.dirty = true;
    }

    fn mark_pdf_dirty(&mut self) {
        self.shell.pdf_mode.session.metadata.modified_unix_ms = current_unix_ms();
        self.shell.pdf_mode.session.autosave.dirty = true;
    }
}

fn current_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}
