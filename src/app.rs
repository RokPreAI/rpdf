use eframe::egui;

use crate::model::{
    AnnotationAppearanceSet, AnnotationLayerRole, AnnotationPalette, AnnotationVisibility,
    AutosaveState, BackgroundPattern, BackgroundPatternStyle, BrushStyle, CanvasDocument, CanvasItem,
    DocumentMetadata, HighlightMode, PdfDocumentSession, PdfSource, PdfViewState, PdfViewportState,
    PenStrokeItem, PenToolKind, PlaybackState, Point, ReadingReliability, ReadingSupportState,
    RecolorExportMode, RecolorState, RgbaColor, SelectionTarget, StrokePoint, TextSupportSource,
    TtsState, ViewportState, WarningCode, WorkspaceMode,
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
    fn render_canvas_summary(&self, ui: &mut egui::Ui) {
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
    }

    fn render_pdf_summary(&self, ui: &mut egui::Ui) {
        ui.heading("PDF Root");
        ui.label(format!("Page: {}", self.shell.pdf.viewport.page_index + 1));
        ui.label(format!("Zoom: {:.2}", self.shell.pdf.viewport.zoom));
        ui.label(format!(
            "Text source: {:?}",
            self.shell.pdf.reading_support.text_source
        ));
        ui.label(format!(
            "Reading reliability: {:?}",
            self.shell.pdf.reading_support.reliability
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

        let available = ui.available_size();
        let (response, painter) = ui.allocate_painter(available, egui::Sense::click_and_drag());
        let canvas_rect = response.rect;

        self.apply_canvas_zoom(ui, &response);
        self.apply_canvas_pan(ui, &response);
        self.handle_canvas_drawing(ui, &response);

        painter.rect_filled(canvas_rect, 12.0, egui::Color32::from_rgb(18, 22, 28));
        self.paint_workspace_surface(&painter, canvas_rect);
        self.paint_existing_strokes(&painter, canvas_rect);
        self.paint_active_stroke(&painter, canvas_rect);
    }

    fn render_pdf_workspace(&self, ui: &mut egui::Ui) {
        ui.heading("PDF Mode");
        ui.label("This workspace is the stable attachment point for PDF viewing, annotation, recoloring, and reading support.");
        ui.separator();
        ui.group(|ui| {
            ui.label("Current placeholders:");
            ui.label("- document opening and page navigation will attach here");
            ui.label("- annotation tools will attach here");
            ui.label("- text-to-speech and follow-along highlighting will attach here");
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
    pub pdf: PdfDocumentSession,
}

impl Default for ShellState {
    fn default() -> Self {
        Self {
            mode: WorkspaceMode::InfiniteCanvas,
            canvas: default_canvas_document(),
            canvas_interaction: CanvasInteractionState::default(),
            pdf: default_pdf_session(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CanvasInteractionState {
    pub active_stroke: Option<PendingStroke>,
    pub next_stroke_id: u64,
}

#[derive(Debug, Clone)]
pub struct PendingStroke {
    pub points: Vec<StrokePoint>,
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
                let pressure = latest_pressure(ui, response).unwrap_or(0.5);

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

        let stroke_id = self.shell.canvas_interaction.next_stroke_id;
        self.shell.canvas_interaction.next_stroke_id += 1;

        self.shell.canvas.items.push(CanvasItem::PenStroke(PenStrokeItem {
            item_id: format!("stroke-{stroke_id}"),
            points: active_stroke.points,
            brush: BrushStyle {
                color: AnnotationAppearanceSet {
                    normal_view: accent_color(),
                    recolored_view: accent_color(),
                },
                width: 4.0,
                tool: PenToolKind::Ink,
            },
            layer_role: AnnotationLayerRole::CanvasMarkup,
        }));
        self.shell.canvas.autosave.dirty = true;
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

    fn canvas_world_rect(&self) -> egui::Rect {
        egui::Rect::from_center_size(egui::Pos2::ZERO, egui::vec2(4000.0, 2400.0))
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
}

fn latest_pressure(ui: &egui::Ui, response: &egui::Response) -> Option<f32> {
    ui.input(|input| {
        input.events.iter().rev().find_map(|event| match event {
            egui::Event::Touch { pos, force, .. } if response.rect.contains(*pos) => *force,
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
