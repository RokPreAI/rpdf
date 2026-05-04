use eframe::egui;
use std::fs;
use std::process::Command;
use std::time::Instant;
use crate::model::{
    AnnotationAppearanceSet, AnnotationLayerRole, AnnotationPalette, AnnotationVisibility,
    AutosaveState, BackgroundPattern, BackgroundPatternStyle, BrushStyle, CanvasDocument, CanvasItem,
    DocumentMetadata, HighlightMode, ImportedAssetSource, ImportedImageItem, ImportedPdfPageItem,
    PdfDocumentSession, PdfPenStrokeAnnotation, PdfSource, PdfTextNote, PdfViewState, PdfViewportState,
    PenStrokeItem, PenToolKind, PlaybackState, Point, ReadingReliability, ReadingSupportState, RecolorExportMode,
    RecolorState, Rect, RgbaColor, SelectionTarget, Size, StrokePoint, TextItem, TextStyle,
    TextSupportSource, TtsState, ViewportState, WarningCode, WorkspaceMode,
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
                ui.label(format!("Last opened path: {}", self.startup.last_opened_path.as_deref().unwrap_or("none")));
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
        ui.label(format!(
            "PDF annotations: {}",
            self.shell.pdf.annotations.len()
        ));
    }

    fn render_canvas_workspace(&mut self, ui: &mut egui::Ui) {
        ui.heading("Infinite Canvas Mode");
        ui.horizontal(|ui| {
            ui.label("Primary drag: draw");
            ui.separator();
            ui.label("Secondary drag: pan");
            ui.separator();
            ui.label("Scroll: zoom");
        });

        ui.add_space(8.0);
        self.render_annotation_toolbar(ui, WorkspaceMode::InfiniteCanvas);
        ui.add_space(8.0);
        self.render_canvas_toolbar(ui);
        ui.add_space(8.0);

        let available = ui.available_size();
        let (response, painter) = ui.allocate_painter(available, egui::Sense::click_and_drag());
        let canvas_rect = response.rect;

        self.apply_canvas_zoom(ui, &response);
        self.apply_canvas_pan(ui, &response);
        self.handle_canvas_drawing(ui, &response);

        painter.rect_filled(canvas_rect, 12.0, egui::Color32::from_rgb(18, 22, 28));
        self.paint_workspace_surface(&painter, canvas_rect);
        self.paint_canvas_background(&painter, canvas_rect);
        self.paint_imported_items(&painter, canvas_rect);
        self.paint_text_items(&painter, canvas_rect);
        self.paint_existing_strokes(&painter, canvas_rect);
        self.paint_active_stroke(&painter, canvas_rect);
    }

    fn render_pdf_workspace(&mut self, ui: &mut egui::Ui) {
        ui.heading("PDF Mode");
        self.render_annotation_toolbar(ui, WorkspaceMode::PdfDocument);
        ui.add_space(8.0);
        self.render_pdf_toolbar(ui);
        ui.add_space(8.0);

        let available = ui.available_size();
        let (response, painter) = ui.allocate_painter(available, egui::Sense::click_and_drag());
        let rect = response.rect;
        painter.rect_filled(rect, 12.0, egui::Color32::from_rgb(20, 21, 25));

        let page_rect = centered_page_rect(rect, self.shell.pdf.viewport.zoom);
        self.handle_pdf_drawing(ui, page_rect);
        let (page_fill, page_stroke) = self.current_pdf_page_colors();
        painter.rect_filled(page_rect, 8.0, page_fill);
        painter.rect_stroke(
            page_rect,
            8.0,
            egui::Stroke::new(2.0, page_stroke),
            egui::StrokeKind::Middle,
        );

        let source_label = match &self.shell.pdf.source {
            PdfSource::FilePath(path) if !path.as_os_str().is_empty() => file_label(path),
            _ => "No PDF opened".to_string(),
        };

        painter.text(
            page_rect.center_top() + egui::vec2(0.0, 28.0),
            egui::Align2::CENTER_TOP,
            format!(
                "{}\nPage {} of {}",
                source_label,
                self.shell.pdf.viewport.page_index + 1,
                self.shell.pdf_interaction.page_count
            ),
            egui::FontId::proportional(22.0),
            page_stroke,
        );

        painter.text(
            page_rect.center(),
            egui::Align2::CENTER_CENTER,
            "PDF document viewport\nannotation, recoloring, and TTS attach here next",
            egui::FontId::proportional(18.0),
            page_stroke,
        );
        self.paint_pdf_reading_highlight(&painter, page_rect);
        self.paint_pdf_annotations(&painter, page_rect);
        self.paint_pdf_active_stroke(&painter, page_rect);
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
    pub page_count: usize,
    pub active_stroke: Option<PendingStroke>,
    pub next_annotation_id: u64,
    pub reading_session: Option<ReadingPlaybackSession>,
}

impl Default for PdfInteractionState {
    fn default() -> Self {
        Self {
            pending_open_path: String::new(),
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
                message: "PDF reading support will appear here after document loading is implemented.".to_string(),
            }),
        },
        view: PdfViewState {
            recolor: RecolorState {
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

fn default_palette() -> AnnotationPalette {
    AnnotationPalette {
        ink_color: accent_color(),
        highlighter_color: RgbaColor {
            red: 255,
            green: 214,
            blue: 10,
            alpha: 150,
        },
        text_color: RgbaColor {
            red: 240,
            green: 240,
            blue: 240,
            alpha: 255,
        },
    }
}

fn accent_color() -> RgbaColor {
    RgbaColor {
        red: 58,
        green: 128,
        blue: 247,
        alpha: 255,
    }
}

fn muted_grid_color() -> RgbaColor {
    RgbaColor {
        red: 96,
        green: 104,
        blue: 122,
        alpha: 255,
    }
}

impl RpdfApp {
    fn render_annotation_toolbar(&mut self, ui: &mut egui::Ui, mode: WorkspaceMode) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Annotation tool:");
                ui.selectable_value(&mut self.shell.annotation_tools.current_tool, AnnotationTool::Ink, "Ink");
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

    fn render_pdf_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("PDF path:");
                ui.text_edit_singleline(&mut self.shell.pdf_interaction.pending_open_path);
                if ui.button("Open PDF").clicked() {
                    self.open_pdf_document();
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Previous").clicked() {
                    self.step_pdf_page(-1);
                }
                if ui.button("Next").clicked() {
                    self.step_pdf_page(1);
                }

                let mut display_page = self.shell.pdf.viewport.page_index + 1;
                ui.label("Page:");
                if ui
                    .add(
                        egui::DragValue::new(&mut display_page)
                            .range(1..=self.shell.pdf_interaction.page_count.max(1)),
                    )
                    .changed()
                {
                    self.shell.pdf.viewport.page_index = display_page
                        .saturating_sub(1)
                        .min(self.shell.pdf_interaction.page_count.saturating_sub(1));
                }

                ui.label(format!("of {}", self.shell.pdf_interaction.page_count));
                ui.separator();
                ui.label("Document-focused workspace");
            });

            ui.separator();
            let mut recolor_enabled = self.shell.pdf.view.recolor.current_profile.is_some();
            if ui.checkbox(&mut recolor_enabled, "Enable recolor view").changed() {
                if recolor_enabled {
                    self.shell.pdf.view.recolor.current_profile = Some(default_recolor_profile());
                } else {
                    self.shell.pdf.view.recolor.current_profile = None;
                }
            }

            if let Some(profile) = self.shell.pdf.view.recolor.current_profile.as_mut() {
                ui.horizontal(|ui| {
                    ui.label("Foreground");
                    let mut foreground = to_color32(profile.foreground);
                    if ui.color_edit_button_srgba(&mut foreground).changed() {
                        profile.foreground = from_color32(foreground);
                    }

                    ui.label("Background");
                    let mut background = to_color32(profile.background);
                    if ui.color_edit_button_srgba(&mut background).changed() {
                        profile.background = from_color32(background);
                    }
                });
            }

            ui.horizontal(|ui| {
                ui.label("PDF export recolor:");
                ui.selectable_value(
                    &mut self.shell.pdf.view.recolor.export_mode,
                    RecolorExportMode::PreserveOriginalAppearance,
                    "Preserve original",
                );
                ui.selectable_value(
                    &mut self.shell.pdf.view.recolor.export_mode,
                    RecolorExportMode::IncludeCurrentRecoloring,
                    "Include recolor",
                );
            });

            ui.collapsing("Annotation palettes", |ui| {
                render_palette_editor(
                    ui,
                    "Normal",
                    &mut self.shell.pdf.view.annotation_visibility.normal_view,
                );
                render_palette_editor(
                    ui,
                    "Recolored",
                    &mut self.shell.pdf.view.annotation_visibility.recolored_view,
                );
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Highlight mode:");
                ui.selectable_value(&mut self.shell.pdf.reading_support.highlight_mode, HighlightMode::Word, "Word");
                ui.selectable_value(&mut self.shell.pdf.reading_support.highlight_mode, HighlightMode::Line, "Line");
                ui.selectable_value(
                    &mut self.shell.pdf.reading_support.highlight_mode,
                    HighlightMode::Sentence,
                    "Sentence",
                );
            });

            ui.horizontal(|ui| {
                if ui.button("Start TTS").clicked() {
                    self.start_pdf_tts();
                }
                if ui.button("Stop TTS").clicked() {
                    self.stop_pdf_tts();
                }
                ui.label(format!("Playback: {:?}", self.shell.pdf.reading_support.tts.playback));
            });

            if let Some(warning) = &self.shell.pdf.reading_support.warning {
                ui.label(format!("Reading warning: {}", warning.message));
            }
        });
    }

    fn render_canvas_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Background:");
                if ui
                    .selectable_label(matches!(self.shell.canvas.background, BackgroundPattern::Dots(_)), "Dots")
                    .clicked()
                {
                    self.shell.canvas.background = BackgroundPattern::Dots(default_background_style());
                }
                if ui
                    .selectable_label(matches!(self.shell.canvas.background, BackgroundPattern::Lines(_)), "Lines")
                    .clicked()
                {
                    self.shell.canvas.background = BackgroundPattern::Lines(default_background_style());
                }
                if ui
                    .selectable_label(matches!(self.shell.canvas.background, BackgroundPattern::Squares(_)), "Squares")
                    .clicked()
                {
                    self.shell.canvas.background = BackgroundPattern::Squares(default_background_style());
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Text:");
                ui.text_edit_singleline(&mut self.shell.canvas_interaction.pending_text);
                if ui.button("Add text").clicked() {
                    self.add_canvas_text();
                }
            });

            ui.horizontal(|ui| {
                ui.label("Image path:");
                ui.text_edit_singleline(&mut self.shell.canvas_interaction.pending_image_path);
                if ui.button("Import image").clicked() {
                    self.import_canvas_image();
                }
            });

            ui.horizontal(|ui| {
                ui.label("PDF path:");
                ui.text_edit_singleline(&mut self.shell.canvas_interaction.pending_pdf_path);
                ui.label("Page:");
                ui.add(egui::DragValue::new(&mut self.shell.canvas_interaction.pending_pdf_page).range(1..=9999));
                if ui.button("Import page").clicked() {
                    self.import_canvas_pdf_page();
                }
            });

            ui.separator();
            ui.label("Selection:");
            ui.horizontal(|ui| {
                if ui.button("Whole canvas").clicked() {
                    self.shell.canvas.selection = SelectionTarget::WholeCanvas;
                }
                if ui.button("Clear item selection").clicked() {
                    self.shell.canvas.selection = SelectionTarget::ItemIds(Vec::new());
                }
            });

            let current_ids = match &self.shell.canvas.selection {
                SelectionTarget::ItemIds(ids) => ids.clone(),
                _ => Vec::new(),
            };
            let mut selected_ids = current_ids;

            for item in &self.shell.canvas.items {
                let item_id = canvas_item_id(item).to_string();
                let mut selected = selected_ids.iter().any(|id| id == &item_id);
                if ui
                    .checkbox(&mut selected, format!("{} ({})", item_id, item_kind_label(item)))
                    .changed()
                {
                    if selected {
                        selected_ids.push(item_id.clone());
                    } else {
                        selected_ids.retain(|id| id != &item_id);
                    }
                }
            }

            if !selected_ids.is_empty() {
                selected_ids.sort();
                selected_ids.dedup();
                self.shell.canvas.selection = SelectionTarget::ItemIds(selected_ids);
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("SVG path:");
                ui.text_edit_singleline(&mut self.shell.canvas_interaction.export_path);
                if ui.button("Export SVG").clicked() {
                    self.export_canvas_svg();
                }
            });
            if !self.shell.canvas_interaction.export_status.is_empty() {
                ui.label(&self.shell.canvas_interaction.export_status);
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Apply recolor to selected PDF pages").clicked() {
                    self.apply_recolor_to_selected_canvas_pdf_pages(true);
                }
                if ui.button("Clear recolor on selected PDF pages").clicked() {
                    self.apply_recolor_to_selected_canvas_pdf_pages(false);
                }
            });
        });
    }

    fn apply_canvas_zoom(&mut self, ui: &egui::Ui, response: &egui::Response) {
        if !response.hovered() {
            return;
        }

        let scroll_delta = ui.input(|input| input.smooth_scroll_delta.y);
        if scroll_delta == 0.0 {
            return;
        }

        let previous_zoom = self.shell.canvas.viewport.zoom;
        let next_zoom = (previous_zoom * (1.0 + scroll_delta * 0.001)).clamp(0.2, 4.0);
        self.shell.canvas.viewport.zoom = next_zoom;
    }

    fn apply_canvas_pan(&mut self, ui: &egui::Ui, response: &egui::Response) {
        if !response.hovered() || !ui.input(|input| input.pointer.secondary_down()) {
            return;
        }

        let delta = ui.input(|input| input.pointer.delta());
        self.shell.canvas.viewport.origin.x -= delta.x / self.shell.canvas.viewport.zoom;
        self.shell.canvas.viewport.origin.y -= delta.y / self.shell.canvas.viewport.zoom;
    }

    fn handle_canvas_drawing(&mut self, ui: &egui::Ui, response: &egui::Response) {
        let pointer_pos = ui.input(|input| input.pointer.interact_pos());
        let primary_down = ui.input(|input| input.pointer.primary_down());

        if primary_down {
            if let Some(screen_pos) = pointer_pos.filter(|pos| response.rect.contains(*pos)) {
                let canvas_point = self.screen_to_canvas(response.rect, screen_pos);
                let pressure = latest_pressure(ui, response.rect).unwrap_or(0.5);

                let active_stroke = self
                    .shell
                    .canvas_interaction
                    .active_stroke
                    .get_or_insert_with(|| PendingStroke { points: Vec::new() });

                push_stroke_point(active_stroke, canvas_point, pressure);
                return;
            }
        }

        if !primary_down {
            self.commit_active_stroke();
        }
    }

    fn commit_active_stroke(&mut self) {
        let Some(active_stroke) = self.shell.canvas_interaction.active_stroke.take() else {
            return;
        };

        if active_stroke.points.len() < 2 {
            return;
        }

        let stroke_id = self.next_canvas_item_id();
        let tool = self.shell.annotation_tools.current_tool;
        let stroke_color = match tool {
            AnnotationTool::Ink => accent_color(),
            AnnotationTool::Highlighter => RgbaColor {
                red: 255,
                green: 214,
                blue: 10,
                alpha: 150,
            },
        };

        self.shell.canvas.items.push(CanvasItem::PenStroke(PenStrokeItem {
            item_id: format!("stroke-{stroke_id}"),
            points: active_stroke.points,
            brush: BrushStyle {
                color: AnnotationAppearanceSet {
                    normal_view: stroke_color,
                    recolored_view: stroke_color,
                },
                width: if tool == AnnotationTool::Highlighter { 12.0 } else { 4.0 },
                tool: match tool {
                    AnnotationTool::Ink => PenToolKind::Ink,
                    AnnotationTool::Highlighter => PenToolKind::Highlighter,
                },
            },
            layer_role: AnnotationLayerRole::CanvasMarkup,
        }));
        self.shell.canvas.autosave.dirty = true;
    }

    fn handle_pdf_drawing(&mut self, ui: &egui::Ui, page_rect: egui::Rect) {
        let pointer_pos = ui.input(|input| input.pointer.interact_pos());
        let primary_down = ui.input(|input| input.pointer.primary_down());

        if primary_down {
            if let Some(screen_pos) = pointer_pos.filter(|pos| page_rect.contains(*pos)) {
                let page_point = screen_to_pdf_page(page_rect, screen_pos);
                let pressure = latest_pressure(ui, page_rect).unwrap_or(0.5);
                let active = self
                    .shell
                    .pdf_interaction
                    .active_stroke
                    .get_or_insert_with(|| PendingStroke { points: Vec::new() });
                push_stroke_point(active, page_point, pressure);
                return;
            }
        }

        if !primary_down {
            self.commit_pdf_stroke();
        }
    }

    fn commit_pdf_stroke(&mut self) {
        let Some(active_stroke) = self.shell.pdf_interaction.active_stroke.take() else {
            return;
        };

        if active_stroke.points.len() < 2 {
            return;
        }

        let id = self.next_pdf_annotation_id();
        let tool = self.shell.annotation_tools.current_tool;

        self.shell.pdf.annotations.push(crate::model::PdfAnnotation::PenStroke(
            PdfPenStrokeAnnotation {
                annotation_id: format!("pdf-stroke-{id}"),
                page_index: self.shell.pdf.viewport.page_index,
                stroke: PenStrokeItem {
                    item_id: format!("pdf-stroke-item-{id}"),
                    points: active_stroke.points,
                    brush: preview_brush(tool),
                    layer_role: AnnotationLayerRole::PdfMarkup,
                },
            },
        ));
    }

    fn paint_workspace_surface(&self, painter: &egui::Painter, rect: egui::Rect) {
        let workspace = self.canvas_world_rect();
        let screen_rect = self.world_rect_to_screen(rect, workspace);
        painter.rect_filled(screen_rect, 16.0, egui::Color32::from_rgb(26, 31, 39));
        painter.rect_stroke(
            screen_rect,
            16.0,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(70, 80, 98)),
            egui::StrokeKind::Middle,
        );
    }

    fn paint_canvas_background(&self, painter: &egui::Painter, rect: egui::Rect) {
        let style = match &self.shell.canvas.background {
            BackgroundPattern::None => return,
            BackgroundPattern::Dots(style)
            | BackgroundPattern::Lines(style)
            | BackgroundPattern::Squares(style) => style,
        };

        let spacing = (style.spacing * self.shell.canvas.viewport.zoom).max(8.0);
        let workspace = self.world_rect_to_screen(rect, self.canvas_world_rect());
        let color = to_color32(style.color);

        match &self.shell.canvas.background {
            BackgroundPattern::Dots(_) => {
                let mut x = workspace.left();
                while x <= workspace.right() {
                    let mut y = workspace.top();
                    while y <= workspace.bottom() {
                        painter.circle_filled(egui::pos2(x, y), style.line_width.max(1.0), color);
                        y += spacing;
                    }
                    x += spacing;
                }
            }
            BackgroundPattern::Lines(_) => {
                let mut y = workspace.top();
                while y <= workspace.bottom() {
                    painter.line_segment(
                        [egui::pos2(workspace.left(), y), egui::pos2(workspace.right(), y)],
                        egui::Stroke::new(style.line_width, color),
                    );
                    y += spacing;
                }
            }
            BackgroundPattern::Squares(_) => {
                let mut x = workspace.left();
                while x <= workspace.right() {
                    painter.line_segment(
                        [egui::pos2(x, workspace.top()), egui::pos2(x, workspace.bottom())],
                        egui::Stroke::new(style.line_width, color),
                    );
                    x += spacing;
                }

                let mut y = workspace.top();
                while y <= workspace.bottom() {
                    painter.line_segment(
                        [egui::pos2(workspace.left(), y), egui::pos2(workspace.right(), y)],
                        egui::Stroke::new(style.line_width, color),
                    );
                    y += spacing;
                }
            }
            BackgroundPattern::None => {}
        }
    }

    fn paint_imported_items(&self, painter: &egui::Painter, rect: egui::Rect) {
        for item in &self.shell.canvas.items {
            match item {
                CanvasItem::ImportedImage(image) => self.paint_imported_image(painter, rect, image),
                CanvasItem::ImportedPdfPage(page) => self.paint_imported_pdf_page(painter, rect, page),
                _ => {}
            }
        }
    }

    fn paint_text_items(&self, painter: &egui::Painter, rect: egui::Rect) {
        for item in &self.shell.canvas.items {
            let CanvasItem::Text(text) = item else {
                continue;
            };
            let screen_pos = self.canvas_to_screen(rect, text.bounds.origin);
            painter.text(
                screen_pos,
                egui::Align2::LEFT_TOP,
                &text.text,
                egui::FontId::proportional(text.style.font_size * self.shell.canvas.viewport.zoom.clamp(0.7, 1.4)),
                to_color32(text.style.color.normal_view),
            );
        }
    }

    fn paint_existing_strokes(&self, painter: &egui::Painter, rect: egui::Rect) {
        for item in &self.shell.canvas.items {
            let CanvasItem::PenStroke(stroke) = item else {
                continue;
            };
            self.paint_stroke(
                painter,
                rect,
                &stroke.points,
                stroke.brush.width,
                stroke.brush.color.normal_view,
            );
        }
    }

    fn paint_active_stroke(&self, painter: &egui::Painter, rect: egui::Rect) {
        let Some(stroke) = &self.shell.canvas_interaction.active_stroke else {
            return;
        };

        self.paint_stroke(painter, rect, &stroke.points, 4.0, accent_color());
    }

    fn paint_pdf_annotations(&self, painter: &egui::Painter, page_rect: egui::Rect) {
        for annotation in &self.shell.pdf.annotations {
            match annotation {
                crate::model::PdfAnnotation::PenStroke(stroke)
                    if stroke.page_index == self.shell.pdf.viewport.page_index =>
                {
                    self.paint_pdf_stroke(
                        painter,
                        page_rect,
                        &stroke.stroke.points,
                        stroke.stroke.brush.width,
                        self.pdf_annotation_color_for_brush(&stroke.stroke.brush),
                    );
                }
                crate::model::PdfAnnotation::TextNote(note)
                    if note.page_index == self.shell.pdf.viewport.page_index =>
                {
                    let pos = pdf_page_to_screen(page_rect, note.anchor.origin);
                    painter.text(
                        pos,
                        egui::Align2::LEFT_TOP,
                        &note.text,
                        egui::FontId::proportional(18.0),
                        to_color32(self.current_annotation_palette().text_color),
                    );
                }
                _ => {}
            }
        }
    }

    fn paint_pdf_active_stroke(&self, painter: &egui::Painter, page_rect: egui::Rect) {
        let Some(stroke) = &self.shell.pdf_interaction.active_stroke else {
            return;
        };
        let preview = preview_brush(self.shell.annotation_tools.current_tool);
        self.paint_pdf_stroke(
            painter,
            page_rect,
            &stroke.points,
            preview.width,
            self.pdf_annotation_color_for_brush(&preview),
        );
    }

    fn paint_stroke(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        points: &[StrokePoint],
        base_width: f32,
        color: RgbaColor,
    ) {
        if points.len() < 2 {
            return;
        }

        for segment in points.windows(2) {
            let start = self.canvas_to_screen(rect, segment[0].position);
            let end = self.canvas_to_screen(rect, segment[1].position);
            let width = base_width * ((segment[0].pressure + segment[1].pressure) * 0.5).max(0.2);
            painter.line_segment(
                [start, end],
                egui::Stroke::new(width, to_color32(color)),
            );
        }
    }

    fn paint_pdf_stroke(
        &self,
        painter: &egui::Painter,
        page_rect: egui::Rect,
        points: &[StrokePoint],
        base_width: f32,
        color: RgbaColor,
    ) {
        if points.len() < 2 {
            return;
        }

        for segment in points.windows(2) {
            let start = pdf_page_to_screen(page_rect, segment[0].position);
            let end = pdf_page_to_screen(page_rect, segment[1].position);
            let width = base_width * ((segment[0].pressure + segment[1].pressure) * 0.5).max(0.2);
            painter.line_segment([start, end], egui::Stroke::new(width, to_color32(color)));
        }
    }

    fn canvas_world_rect(&self) -> egui::Rect {
        egui::Rect::from_center_size(egui::Pos2::ZERO, egui::vec2(4000.0, 2400.0))
    }

    fn paint_imported_image(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        image: &ImportedImageItem,
    ) {
        let screen_rect = self.world_rect_to_screen(
            rect,
            egui::Rect::from_min_size(
                egui::pos2(image.bounds.origin.x, image.bounds.origin.y),
                egui::vec2(image.bounds.size.width, image.bounds.size.height),
            ),
        );

        painter.rect_filled(screen_rect, 10.0, egui::Color32::from_rgb(58, 69, 89));
        painter.rect_stroke(
            screen_rect,
            10.0,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(126, 159, 255)),
            egui::StrokeKind::Middle,
        );

        let label = match &image.source {
            ImportedAssetSource::FilePath(path) => format!("Image\n{}", file_label(path)),
        };
        painter.text(
            screen_rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(18.0),
            egui::Color32::WHITE,
        );
    }

    fn paint_imported_pdf_page(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        page: &ImportedPdfPageItem,
    ) {
        let screen_rect = self.world_rect_to_screen(
            rect,
            egui::Rect::from_min_size(
                egui::pos2(page.bounds.origin.x, page.bounds.origin.y),
                egui::vec2(page.bounds.size.width, page.bounds.size.height),
            ),
        );

        let fill = page
            .recolor_override
            .as_ref()
            .map(|profile| to_color32(profile.background))
            .unwrap_or_else(|| egui::Color32::from_rgb(238, 238, 235));
        let stroke = page
            .recolor_override
            .as_ref()
            .map(|profile| to_color32(profile.foreground))
            .unwrap_or_else(|| egui::Color32::from_rgb(66, 71, 78));

        painter.rect_filled(screen_rect, 8.0, fill);
        painter.rect_stroke(
            screen_rect,
            8.0,
            egui::Stroke::new(2.0, stroke),
            egui::StrokeKind::Middle,
        );

        let source_name = match &page.source {
            PdfSource::FilePath(path) => file_label(path),
        };
        painter.text(
            screen_rect.center_top() + egui::vec2(0.0, 24.0),
            egui::Align2::CENTER_TOP,
            format!("PDF Page {}\n{}", page.page_index + 1, source_name),
            egui::FontId::proportional(18.0),
            stroke,
        );
    }

    fn canvas_to_screen(&self, rect: egui::Rect, point: Point) -> egui::Pos2 {
        egui::pos2(
            rect.center().x + (point.x - self.shell.canvas.viewport.origin.x) * self.shell.canvas.viewport.zoom,
            rect.center().y + (point.y - self.shell.canvas.viewport.origin.y) * self.shell.canvas.viewport.zoom,
        )
    }

    fn screen_to_canvas(&self, rect: egui::Rect, pos: egui::Pos2) -> Point {
        Point {
            x: self.shell.canvas.viewport.origin.x
                + (pos.x - rect.center().x) / self.shell.canvas.viewport.zoom,
            y: self.shell.canvas.viewport.origin.y
                + (pos.y - rect.center().y) / self.shell.canvas.viewport.zoom,
        }
    }

    fn world_rect_to_screen(&self, rect: egui::Rect, world_rect: egui::Rect) -> egui::Rect {
        let min = self.canvas_to_screen(
            rect,
            Point {
                x: world_rect.min.x,
                y: world_rect.min.y,
            },
        );
        let max = self.canvas_to_screen(
            rect,
            Point {
                x: world_rect.max.x,
                y: world_rect.max.y,
            },
        );
        egui::Rect::from_two_pos(min, max)
    }

    fn add_canvas_text(&mut self) {
        let text = self.shell.canvas_interaction.pending_text.trim().to_string();
        if text.is_empty() {
            return;
        }

        let item_id = self.next_canvas_item_id();
        self.shell.canvas.items.push(CanvasItem::Text(TextItem {
            item_id: format!("text-{item_id}"),
            bounds: Rect {
                origin: self.shell.canvas.viewport.origin,
                size: Size {
                    width: 320.0,
                    height: 80.0,
                },
            },
            text,
            style: TextStyle {
                font_family: "Proportional".to_string(),
                font_size: 24.0,
                color: AnnotationAppearanceSet {
                    normal_view: RgbaColor {
                        red: 240,
                        green: 240,
                        blue: 240,
                        alpha: 255,
                    },
                    recolored_view: RgbaColor {
                        red: 240,
                        green: 240,
                        blue: 240,
                        alpha: 255,
                    },
                },
            },
        }));
        self.shell.canvas.autosave.dirty = true;
        self.shell.canvas_interaction.pending_text.clear();
    }

    fn import_canvas_image(&mut self) {
        let path = self.shell.canvas_interaction.pending_image_path.trim().to_string();
        if path.is_empty() {
            return;
        }

        let item_id = self.next_canvas_item_id();
        let size = Size {
            width: 320.0,
            height: 220.0,
        };
        self.shell.canvas.items.push(CanvasItem::ImportedImage(ImportedImageItem {
            item_id: format!("image-{item_id}"),
            source: ImportedAssetSource::FilePath(path.clone().into()),
            bounds: Rect {
                origin: self.shifted_canvas_origin(60.0 * item_id as f32, 40.0 * item_id as f32),
                size,
            },
        }));
        self.shell.canvas.autosave.dirty = true;
        self.shell.canvas_interaction.pending_image_path.clear();
    }

    fn import_canvas_pdf_page(&mut self) {
        let path = self.shell.canvas_interaction.pending_pdf_path.trim().to_string();
        if path.is_empty() {
            return;
        }

        let item_id = self.next_canvas_item_id();
        self.shell.canvas.items.push(CanvasItem::ImportedPdfPage(ImportedPdfPageItem {
            item_id: format!("pdf-page-{item_id}"),
            source: PdfSource::FilePath(path.clone().into()),
            page_index: self.shell.canvas_interaction.pending_pdf_page.saturating_sub(1),
            bounds: Rect {
                origin: self.shifted_canvas_origin(80.0 * item_id as f32, 48.0 * item_id as f32),
                size: Size {
                    width: 420.0,
                    height: 560.0,
                },
            },
            recolor_override: None,
        }));
        self.shell.canvas.autosave.dirty = true;
        self.shell.canvas_interaction.pending_pdf_path.clear();
    }

    fn shifted_canvas_origin(&self, dx: f32, dy: f32) -> Point {
        Point {
            x: self.shell.canvas.viewport.origin.x + dx,
            y: self.shell.canvas.viewport.origin.y + dy,
        }
    }

    fn next_canvas_item_id(&mut self) -> u64 {
        let next = self.shell.canvas_interaction.next_item_id;
        self.shell.canvas_interaction.next_item_id += 1;
        next
    }

    fn open_pdf_document(&mut self) {
        let path = self.shell.pdf_interaction.pending_open_path.trim().to_string();
        if path.is_empty() {
            return;
        }

        self.shell.pdf.source = PdfSource::FilePath(path.clone().into());
        self.shell.pdf.metadata.title = Some(file_label(std::path::Path::new(&path)));
        self.shell.pdf.viewport.page_index = 0;
        self.shell.pdf.viewport.scroll_offset = Point { x: 0.0, y: 0.0 };
        self.shell.pdf_interaction.page_count = best_effort_pdf_page_count(&path).max(1);
        self.startup.last_opened_path = Some(path);
    }

    fn step_pdf_page(&mut self, delta: isize) {
        let current = self.shell.pdf.viewport.page_index as isize;
        let max = self.shell.pdf_interaction.page_count.saturating_sub(1) as isize;
        self.shell.pdf.viewport.page_index = (current + delta).clamp(0, max) as usize;
    }

    fn add_canvas_note(&mut self) {
        let note = self.shell.annotation_tools.pending_note_text.trim().to_string();
        if note.is_empty() {
            return;
        }

        let item_id = self.next_canvas_item_id();
        self.shell.canvas.items.push(CanvasItem::Text(TextItem {
            item_id: format!("canvas-note-{item_id}"),
            bounds: Rect {
                origin: self.shifted_canvas_origin(40.0, 40.0),
                size: Size {
                    width: 280.0,
                    height: 70.0,
                },
            },
            text: note,
            style: note_text_style(),
        }));
        self.shell.annotation_tools.pending_note_text.clear();
    }

    fn add_pdf_note(&mut self) {
        let note = self.shell.annotation_tools.pending_note_text.trim().to_string();
        if note.is_empty() {
            return;
        }

        let id = self.next_pdf_annotation_id();
        self.shell.pdf.annotations.push(crate::model::PdfAnnotation::TextNote(PdfTextNote {
            note_id: format!("pdf-note-{id}"),
            page_index: self.shell.pdf.viewport.page_index,
            anchor: Rect {
                origin: Point { x: 64.0, y: 96.0 },
                size: Size {
                    width: 240.0,
                    height: 60.0,
                },
            },
            text: note,
            style: note_text_style(),
        }));
        self.shell.annotation_tools.pending_note_text.clear();
    }

    fn next_pdf_annotation_id(&mut self) -> u64 {
        let next = self.shell.pdf_interaction.next_annotation_id;
        self.shell.pdf_interaction.next_annotation_id += 1;
        next
    }

    fn apply_recolor_to_selected_canvas_pdf_pages(&mut self, enable: bool) {
        let selected_ids = match &self.shell.canvas.selection {
            SelectionTarget::ItemIds(ids) => ids.clone(),
            _ => Vec::new(),
        };

        if selected_ids.is_empty() {
            return;
        }

        let profile = self
            .shell
            .pdf
            .view
            .recolor
            .current_profile
            .clone()
            .unwrap_or_else(default_recolor_profile);

        for item in &mut self.shell.canvas.items {
            let CanvasItem::ImportedPdfPage(page) = item else {
                continue;
            };
            if selected_ids.iter().any(|id| id == &page.item_id) {
                page.recolor_override = enable.then_some(profile.clone());
            }
        }
    }

    fn current_annotation_palette(&self) -> &AnnotationPalette {
        if self.shell.pdf.view.recolor.current_profile.is_some() {
            &self.shell.pdf.view.annotation_visibility.recolored_view
        } else {
            &self.shell.pdf.view.annotation_visibility.normal_view
        }
    }

    fn pdf_annotation_color_for_brush(&self, brush: &BrushStyle) -> RgbaColor {
        let palette = self.current_annotation_palette();
        match brush.tool {
            PenToolKind::Ink => palette.ink_color,
            PenToolKind::Highlighter => palette.highlighter_color,
        }
    }

    fn current_pdf_page_colors(&self) -> (egui::Color32, egui::Color32) {
        if let Some(profile) = &self.shell.pdf.view.recolor.current_profile {
            (to_color32(profile.background), to_color32(profile.foreground))
        } else {
            (
                egui::Color32::from_rgb(242, 242, 238),
                egui::Color32::from_rgb(78, 83, 92),
            )
        }
    }

    fn start_pdf_tts(&mut self) {
        let path = match &self.shell.pdf.source {
            PdfSource::FilePath(path) if !path.as_os_str().is_empty() => path.clone(),
            _ => {
                self.shell.pdf.reading_support.warning = Some(crate::model::UserVisibleWarning {
                    code: WarningCode::ReadingSupportUnavailable,
                    message: "Open a PDF before starting TTS.".to_string(),
                });
                return;
            }
        };

        let extracted = best_effort_extract_pdf_text(&path.to_string_lossy());
        if extracted.trim().is_empty() {
            self.shell.pdf.reading_support.text_source = TextSupportSource::Unavailable;
            self.shell.pdf.reading_support.reliability = ReadingReliability::Unreliable;
            self.shell.pdf.reading_support.warning = Some(crate::model::UserVisibleWarning {
                code: WarningCode::WeakNativeText,
                message: "Could not extract usable PDF text for TTS in the current pre-OCR mode.".to_string(),
            });
            self.shell.pdf_interaction.reading_session = None;
            self.shell.pdf.reading_support.tts.playback = PlaybackState::Stopped;
            self.shell.pdf.reading_support.tts.active_span = None;
            return;
        }

        let spans = build_reading_spans(&extracted, self.shell.pdf.reading_support.highlight_mode);
        if spans.is_empty() {
            return;
        }

        let excerpt = spans.join(" ");
        let _ = Command::new("spd-say").arg(&excerpt).spawn();

        self.shell.pdf.reading_support.text_source = TextSupportSource::NativePdfText;
        self.shell.pdf.reading_support.reliability = ReadingReliability::BestEffort;
        self.shell.pdf.reading_support.warning = None;
        self.shell.pdf.reading_support.tts.playback = PlaybackState::Playing;
        self.shell.pdf_interaction.reading_session = Some(ReadingPlaybackSession {
            spans,
            started_at: Instant::now(),
            span_duration_ms: 1200,
        });
    }

    fn stop_pdf_tts(&mut self) {
        self.shell.pdf.reading_support.tts.playback = PlaybackState::Stopped;
        self.shell.pdf.reading_support.tts.active_span = None;
        self.shell.pdf_interaction.reading_session = None;
    }

    fn tick_pdf_reading_support(&mut self) {
        let Some(session) = &self.shell.pdf_interaction.reading_session else {
            return;
        };

        if self.shell.pdf.reading_support.tts.playback != PlaybackState::Playing {
            return;
        }

        let elapsed_ms = session.started_at.elapsed().as_millis() as u64;
        let index = (elapsed_ms / session.span_duration_ms) as usize;
        if index >= session.spans.len() {
            self.stop_pdf_tts();
            return;
        }

        let span_height = 44.0;
        let y = 120.0 + span_height * index as f32;
        self.shell.pdf.reading_support.tts.active_span = Some(crate::model::ReadingSpan {
            page_index: self.shell.pdf.viewport.page_index,
            bounds: Rect {
                origin: Point { x: 48.0, y },
                size: Size {
                    width: 460.0,
                    height: 34.0,
                },
            },
            text: session.spans[index].clone(),
        });
    }

    fn paint_pdf_reading_highlight(&self, painter: &egui::Painter, page_rect: egui::Rect) {
        let Some(span) = &self.shell.pdf.reading_support.tts.active_span else {
            return;
        };
        if span.page_index != self.shell.pdf.viewport.page_index {
            return;
        }

        let highlight_rect = egui::Rect::from_min_size(
            pdf_page_to_screen(page_rect, span.bounds.origin),
            egui::vec2(span.bounds.size.width, span.bounds.size.height),
        );
        let color = match self.shell.pdf.reading_support.highlight_mode {
            HighlightMode::Word => egui::Color32::from_rgba_premultiplied(255, 214, 10, 180),
            HighlightMode::Line => egui::Color32::from_rgba_premultiplied(58, 128, 247, 90),
            HighlightMode::Sentence => egui::Color32::from_rgba_premultiplied(96, 104, 122, 120),
            HighlightMode::ManualFallback => egui::Color32::from_rgba_premultiplied(255, 255, 255, 40),
        };
        painter.rect_filled(highlight_rect, 6.0, color);
        painter.text(
            highlight_rect.left_top() + egui::vec2(8.0, 8.0),
            egui::Align2::LEFT_TOP,
            &span.text,
            egui::FontId::proportional(16.0),
            egui::Color32::BLACK,
        );
    }

    fn export_canvas_svg(&mut self) {
        let export_path = self.shell.canvas_interaction.export_path.trim().to_string();
        if export_path.is_empty() {
            self.shell.canvas_interaction.export_status =
                "SVG export needs a target file path.".to_string();
            return;
        }

        let selected_items = self.selected_canvas_items();
        let target_items = if selected_items.is_empty() {
            self.shell.canvas.items.iter().collect::<Vec<_>>()
        } else {
            selected_items
        };

        if target_items.is_empty() {
            self.shell.canvas_interaction.export_status =
                "No canvas items are available for export.".to_string();
            return;
        }

        if let Some(reason) = target_items
            .iter()
            .find_map(|item| match item.svg_compatibility() {
                crate::model::SvgCompatibility::Compatible => None,
                crate::model::SvgCompatibility::Incompatible(reason) => Some(reason),
            })
        {
            self.shell.canvas_interaction.export_status =
                format!("SVG export unavailable for this target: {:?}.", reason);
            return;
        }

        let svg = build_svg_document(&target_items);
        match fs::write(&export_path, svg) {
            Ok(()) => {
                self.shell.canvas_interaction.export_status =
                    format!("Exported SVG to {export_path}");
            }
            Err(error) => {
                self.shell.canvas_interaction.export_status =
                    format!("SVG export failed: {error}");
            }
        }
    }

    fn selected_canvas_items(&self) -> Vec<&CanvasItem> {
        match &self.shell.canvas.selection {
            SelectionTarget::ItemIds(ids) if !ids.is_empty() => self
                .shell
                .canvas
                .items
                .iter()
                .filter(|item| ids.iter().any(|id| id == canvas_item_id(item)))
                .collect(),
            _ => Vec::new(),
        }
    }
}

fn latest_pressure(ui: &egui::Ui, rect: egui::Rect) -> Option<f32> {
    ui.input(|input| {
        input.events.iter().rev().find_map(|event| match event {
            egui::Event::Touch { pos, force, .. } if rect.contains(*pos) => *force,
            _ => None,
        })
    })
}

fn push_stroke_point(stroke: &mut PendingStroke, position: Point, pressure: f32) {
    let next_point = StrokePoint { position, pressure };
    let should_push = stroke.points.last().is_none_or(|previous| {
        let dx = previous.position.x - next_point.position.x;
        let dy = previous.position.y - next_point.position.y;
        let dp = previous.pressure - next_point.pressure;
        (dx * dx + dy * dy) > 1.0 || dp.abs() > 0.05
    });

    if should_push {
        stroke.points.push(next_point);
    }
}

fn to_color32(color: RgbaColor) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(color.red, color.green, color.blue, color.alpha)
}

fn default_background_style() -> BackgroundPatternStyle {
    BackgroundPatternStyle {
        spacing: 24.0,
        line_width: 1.0,
        color: muted_grid_color(),
    }
}

fn file_label(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.display().to_string())
}

fn best_effort_pdf_page_count(path: &str) -> usize {
    let Ok(bytes) = fs::read(path) else {
        return 1;
    };
    let content = String::from_utf8_lossy(&bytes);
    content.matches("/Type /Page").count().max(1)
}

fn centered_page_rect(rect: egui::Rect, zoom: f32) -> egui::Rect {
    let width = (560.0 * zoom.clamp(0.75, 1.4)).min((rect.width() - 48.0).max(120.0));
    let height = (760.0 * zoom.clamp(0.75, 1.4)).min((rect.height() - 48.0).max(160.0));
    egui::Rect::from_center_size(rect.center(), egui::vec2(width, height))
}

fn pdf_page_to_screen(page_rect: egui::Rect, point: Point) -> egui::Pos2 {
    egui::pos2(page_rect.left() + point.x, page_rect.top() + point.y)
}

fn screen_to_pdf_page(page_rect: egui::Rect, pos: egui::Pos2) -> Point {
    Point {
        x: pos.x - page_rect.left(),
        y: pos.y - page_rect.top(),
    }
}

fn best_effort_extract_pdf_text(path: &str) -> String {
    let Ok(bytes) = fs::read(path) else {
        return String::new();
    };

    let printable = bytes
        .into_iter()
        .map(|byte| match byte {
            b'\n' | b'\r' | b'\t' => ' ',
            32..=126 => byte as char,
            _ => ' ',
        })
        .collect::<String>();

    printable
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn build_reading_spans(text: &str, mode: HighlightMode) -> Vec<String> {
    match mode {
        HighlightMode::Word => text
            .split_whitespace()
            .take(24)
            .map(ToOwned::to_owned)
            .collect(),
        HighlightMode::Line => text
            .split_whitespace()
            .collect::<Vec<_>>()
            .chunks(8)
            .take(16)
            .map(|chunk| chunk.join(" "))
            .collect(),
        HighlightMode::Sentence => text
            .split(['.', '!', '?'])
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .take(12)
            .map(ToOwned::to_owned)
            .collect(),
        HighlightMode::ManualFallback => vec![text.chars().take(120).collect()],
    }
}

fn note_text_style() -> TextStyle {
    TextStyle {
        font_family: "Proportional".to_string(),
        font_size: 20.0,
        color: AnnotationAppearanceSet {
            normal_view: RgbaColor {
                red: 245,
                green: 245,
                blue: 245,
                alpha: 255,
            },
            recolored_view: RgbaColor {
                red: 245,
                green: 245,
                blue: 245,
                alpha: 255,
            },
        },
    }
}

fn preview_brush(tool: AnnotationTool) -> BrushStyle {
    let color = match tool {
        AnnotationTool::Ink => accent_color(),
        AnnotationTool::Highlighter => RgbaColor {
            red: 255,
            green: 214,
            blue: 10,
            alpha: 150,
        },
    };

    BrushStyle {
        color: AnnotationAppearanceSet {
            normal_view: color,
            recolored_view: color,
        },
        width: if tool == AnnotationTool::Highlighter { 12.0 } else { 4.0 },
        tool: match tool {
            AnnotationTool::Ink => PenToolKind::Ink,
            AnnotationTool::Highlighter => PenToolKind::Highlighter,
        },
    }
}

fn default_recolor_profile() -> crate::model::RecolorProfile {
    crate::model::RecolorProfile {
        foreground: RgbaColor {
            red: 223,
            green: 228,
            blue: 236,
            alpha: 255,
        },
        background: RgbaColor {
            red: 24,
            green: 27,
            blue: 34,
            alpha: 255,
        },
    }
}

fn from_color32(color: egui::Color32) -> RgbaColor {
    RgbaColor {
        red: color.r(),
        green: color.g(),
        blue: color.b(),
        alpha: color.a(),
    }
}

fn render_palette_editor(ui: &mut egui::Ui, label: &str, palette: &mut AnnotationPalette) {
    ui.label(label);
    ui.horizontal(|ui| {
        ui.label("Ink");
        let mut ink = to_color32(palette.ink_color);
        if ui.color_edit_button_srgba(&mut ink).changed() {
            palette.ink_color = from_color32(ink);
        }

        ui.label("Highlight");
        let mut highlight = to_color32(palette.highlighter_color);
        if ui.color_edit_button_srgba(&mut highlight).changed() {
            palette.highlighter_color = from_color32(highlight);
        }

        ui.label("Text");
        let mut text = to_color32(palette.text_color);
        if ui.color_edit_button_srgba(&mut text).changed() {
            palette.text_color = from_color32(text);
        }
    });
}

fn canvas_item_id(item: &CanvasItem) -> &str {
    match item {
        CanvasItem::PenStroke(stroke) => &stroke.item_id,
        CanvasItem::Text(text) => &text.item_id,
        CanvasItem::ImportedImage(image) => &image.item_id,
        CanvasItem::ImportedPdfPage(page) => &page.item_id,
    }
}

fn item_kind_label(item: &CanvasItem) -> &'static str {
    match item {
        CanvasItem::PenStroke(_) => "stroke",
        CanvasItem::Text(_) => "text",
        CanvasItem::ImportedImage(_) => "image",
        CanvasItem::ImportedPdfPage(_) => "pdf-page",
    }
}

fn build_svg_document(items: &[&CanvasItem]) -> String {
    let mut svg = String::from(
        r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" viewBox="-2000 -1200 4000 2400">"#,
    );

    for item in items {
        match item {
            CanvasItem::PenStroke(stroke) => {
                let color = stroke.brush.color.normal_view;
                let points = stroke
                    .points
                    .iter()
                    .map(|point| format!("{},{}", point.position.x, point.position.y))
                    .collect::<Vec<_>>()
                    .join(" ");
                svg.push_str(&format!(
                    r#"<polyline fill="none" stroke="rgba({},{},{},{:.3})" stroke-width="{}" stroke-linecap="round" stroke-linejoin="round" points="{}" />"#,
                    color.red,
                    color.green,
                    color.blue,
                    f32::from(color.alpha) / 255.0,
                    stroke.brush.width,
                    points,
                ));
            }
            CanvasItem::Text(text) => {
                let color = text.style.color.normal_view;
                svg.push_str(&format!(
                    r#"<text x="{}" y="{}" font-size="{}" fill="rgba({},{},{},{:.3})">{}</text>"#,
                    text.bounds.origin.x,
                    text.bounds.origin.y + text.style.font_size,
                    text.style.font_size,
                    color.red,
                    color.green,
                    color.blue,
                    f32::from(color.alpha) / 255.0,
                    escape_svg_text(&text.text),
                ));
            }
            CanvasItem::ImportedImage(_) | CanvasItem::ImportedPdfPage(_) => {}
        }
    }

    svg.push_str("</svg>");
    svg
}

fn escape_svg_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
