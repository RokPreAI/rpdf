use super::util::{
    accent_color, canvas_item_id, default_background_style, default_recolor_profile, file_label,
    item_kind_label, latest_pressure, note_text_style, push_stroke_point, to_color32,
};
use super::*;

impl RpdfApp {
    pub(super) fn render_canvas_workspace(&mut self, ui: &mut egui::Ui) {
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

    fn render_canvas_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Background:");
                if ui
                    .selectable_label(
                        matches!(
                            self.shell.canvas_mode.document.background,
                            BackgroundPattern::Dots(_)
                        ),
                        "Dots",
                    )
                    .clicked()
                {
                    self.shell.canvas_mode.document.background =
                        BackgroundPattern::Dots(default_background_style());
                    self.mark_canvas_dirty();
                }
                if ui
                    .selectable_label(
                        matches!(
                            self.shell.canvas_mode.document.background,
                            BackgroundPattern::Lines(_)
                        ),
                        "Lines",
                    )
                    .clicked()
                {
                    self.shell.canvas_mode.document.background =
                        BackgroundPattern::Lines(default_background_style());
                    self.mark_canvas_dirty();
                }
                if ui
                    .selectable_label(
                        matches!(
                            self.shell.canvas_mode.document.background,
                            BackgroundPattern::Squares(_)
                        ),
                        "Squares",
                    )
                    .clicked()
                {
                    self.shell.canvas_mode.document.background =
                        BackgroundPattern::Squares(default_background_style());
                    self.mark_canvas_dirty();
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Text:");
                ui.text_edit_singleline(&mut self.shell.canvas_mode.ui.pending_text);
                if ui.button("Add text").clicked() {
                    self.add_canvas_text();
                }
            });

            ui.horizontal(|ui| {
                ui.label("Save path:");
                ui.text_edit_singleline(&mut self.shell.canvas_mode.ui.document_path);
                if ui.button("Save canvas").clicked() {
                    self.save_canvas_document();
                }
                if ui.button("Load canvas").clicked() {
                    self.load_canvas_document();
                }
                if ui
                    .add_enabled(
                        self.services.persistence.has_canvas_recovery_snapshot(),
                        egui::Button::new("Recover autosave"),
                    )
                    .clicked()
                {
                    self.recover_canvas_document();
                }
            });
            if !self.shell.canvas_mode.ui.save_status.is_empty() {
                ui.label(&self.shell.canvas_mode.ui.save_status);
            }

            ui.horizontal(|ui| {
                ui.label("Image path:");
                ui.text_edit_singleline(&mut self.shell.canvas_mode.ui.pending_image_path);
                if ui.button("Import image").clicked() {
                    self.import_canvas_image();
                }
            });

            ui.horizontal(|ui| {
                ui.label("PDF path:");
                ui.text_edit_singleline(&mut self.shell.canvas_mode.ui.pending_pdf_path);
                ui.label("Page:");
                ui.add(
                    egui::DragValue::new(&mut self.shell.canvas_mode.ui.pending_pdf_page)
                        .range(1..=9999),
                );
                if ui.button("Import page").clicked() {
                    self.import_canvas_pdf_page();
                }
            });

            ui.separator();
            ui.label("Selection:");
            ui.horizontal(|ui| {
                if ui.button("Whole canvas").clicked() {
                    self.shell.canvas_mode.document.selection = SelectionTarget::WholeCanvas;
                }
                if ui.button("Clear item selection").clicked() {
                    self.shell.canvas_mode.document.selection =
                        SelectionTarget::ItemIds(Vec::new());
                }
            });

            let current_ids = match &self.shell.canvas_mode.document.selection {
                SelectionTarget::ItemIds(ids) => ids.clone(),
                _ => Vec::new(),
            };
            let mut selected_ids = current_ids;

            for item in &self.shell.canvas_mode.document.items {
                let item_id = canvas_item_id(item).to_string();
                let mut selected = selected_ids.iter().any(|id| id == &item_id);
                if ui
                    .checkbox(
                        &mut selected,
                        format!("{} ({})", item_id, item_kind_label(item)),
                    )
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
                self.shell.canvas_mode.document.selection = SelectionTarget::ItemIds(selected_ids);
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("SVG path:");
                ui.text_edit_singleline(&mut self.shell.canvas_mode.ui.export_path);
                if ui.button("Export SVG").clicked() {
                    self.export_canvas_svg();
                }
            });
            if !self.shell.canvas_mode.ui.export_status.is_empty() {
                ui.label(&self.shell.canvas_mode.ui.export_status);
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

        let previous_zoom = self.shell.canvas_mode.document.viewport.zoom;
        let next_zoom = (previous_zoom * (1.0 + scroll_delta * 0.001)).clamp(0.2, 4.0);
        self.shell.canvas_mode.document.viewport.zoom = next_zoom;
    }

    fn apply_canvas_pan(&mut self, ui: &egui::Ui, response: &egui::Response) {
        if !response.hovered() || !ui.input(|input| input.pointer.secondary_down()) {
            return;
        }

        let delta = ui.input(|input| input.pointer.delta());
        self.shell.canvas_mode.document.viewport.origin.x -=
            delta.x / self.shell.canvas_mode.document.viewport.zoom;
        self.shell.canvas_mode.document.viewport.origin.y -=
            delta.y / self.shell.canvas_mode.document.viewport.zoom;
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
                    .canvas_mode
                    .ui
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
        let Some(active_stroke) = self.shell.canvas_mode.ui.active_stroke.take() else {
            return;
        };

        if active_stroke.points.len() < 2 {
            return;
        }

        let stroke_id = self.next_canvas_item_id();
        let tool = self.shell.shared_ui.annotation_tools.current_tool;
        let stroke_color = match tool {
            AnnotationTool::Ink => accent_color(),
            AnnotationTool::Highlighter => RgbaColor {
                red: 255,
                green: 214,
                blue: 10,
                alpha: 150,
            },
        };

        self.shell
            .canvas_mode
            .document
            .items
            .push(CanvasItem::PenStroke(PenStrokeItem {
                item_id: format!("stroke-{stroke_id}"),
                points: active_stroke.points,
                brush: BrushStyle {
                    color: AnnotationAppearanceSet {
                        normal_view: stroke_color,
                        recolored_view: stroke_color,
                    },
                    width: if tool == AnnotationTool::Highlighter {
                        12.0
                    } else {
                        4.0
                    },
                    tool: match tool {
                        AnnotationTool::Ink => PenToolKind::Ink,
                        AnnotationTool::Highlighter => PenToolKind::Highlighter,
                    },
                },
                layer_role: AnnotationLayerRole::CanvasMarkup,
            }));
        self.mark_canvas_dirty();
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
        let style = match &self.shell.canvas_mode.document.background {
            BackgroundPattern::None => return,
            BackgroundPattern::Dots(style)
            | BackgroundPattern::Lines(style)
            | BackgroundPattern::Squares(style) => style,
        };

        let spacing = (style.spacing * self.shell.canvas_mode.document.viewport.zoom).max(8.0);
        let workspace = self.world_rect_to_screen(rect, self.canvas_world_rect());
        let color = to_color32(style.color);

        match &self.shell.canvas_mode.document.background {
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
                        [
                            egui::pos2(workspace.left(), y),
                            egui::pos2(workspace.right(), y),
                        ],
                        egui::Stroke::new(style.line_width, color),
                    );
                    y += spacing;
                }
            }
            BackgroundPattern::Squares(_) => {
                let mut x = workspace.left();
                while x <= workspace.right() {
                    painter.line_segment(
                        [
                            egui::pos2(x, workspace.top()),
                            egui::pos2(x, workspace.bottom()),
                        ],
                        egui::Stroke::new(style.line_width, color),
                    );
                    x += spacing;
                }

                let mut y = workspace.top();
                while y <= workspace.bottom() {
                    painter.line_segment(
                        [
                            egui::pos2(workspace.left(), y),
                            egui::pos2(workspace.right(), y),
                        ],
                        egui::Stroke::new(style.line_width, color),
                    );
                    y += spacing;
                }
            }
            BackgroundPattern::None => {}
        }
    }

    fn paint_imported_items(&self, painter: &egui::Painter, rect: egui::Rect) {
        for item in &self.shell.canvas_mode.document.items {
            match item {
                CanvasItem::ImportedImage(image) => self.paint_imported_image(painter, rect, image),
                CanvasItem::ImportedPdfPage(page) => {
                    self.paint_imported_pdf_page(painter, rect, page)
                }
                _ => {}
            }
        }
    }

    fn paint_text_items(&self, painter: &egui::Painter, rect: egui::Rect) {
        for item in &self.shell.canvas_mode.document.items {
            let CanvasItem::Text(text) = item else {
                continue;
            };
            let screen_pos = self.canvas_to_screen(rect, text.bounds.origin);
            painter.text(
                screen_pos,
                egui::Align2::LEFT_TOP,
                &text.text,
                egui::FontId::proportional(
                    text.style.font_size
                        * self
                            .shell
                            .canvas_mode
                            .document
                            .viewport
                            .zoom
                            .clamp(0.7, 1.4),
                ),
                to_color32(text.style.color.normal_view),
            );
        }
    }

    fn paint_existing_strokes(&self, painter: &egui::Painter, rect: egui::Rect) {
        for item in &self.shell.canvas_mode.document.items {
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
        let Some(stroke) = &self.shell.canvas_mode.ui.active_stroke else {
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
            rect.center().x
                + (point.x - self.shell.canvas_mode.document.viewport.origin.x)
                    * self.shell.canvas_mode.document.viewport.zoom,
            rect.center().y
                + (point.y - self.shell.canvas_mode.document.viewport.origin.y)
                    * self.shell.canvas_mode.document.viewport.zoom,
        )
    }

    fn screen_to_canvas(&self, rect: egui::Rect, pos: egui::Pos2) -> Point {
        Point {
            x: self.shell.canvas_mode.document.viewport.origin.x
                + (pos.x - rect.center().x) / self.shell.canvas_mode.document.viewport.zoom,
            y: self.shell.canvas_mode.document.viewport.origin.y
                + (pos.y - rect.center().y) / self.shell.canvas_mode.document.viewport.zoom,
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
        let text = self.shell.canvas_mode.ui.pending_text.trim().to_string();
        if text.is_empty() {
            return;
        }

        let item_id = self.next_canvas_item_id();
        self.shell
            .canvas_mode
            .document
            .items
            .push(CanvasItem::Text(TextItem {
                item_id: format!("text-{item_id}"),
                bounds: Rect {
                    origin: self.shell.canvas_mode.document.viewport.origin,
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
        self.mark_canvas_dirty();
        self.shell.canvas_mode.ui.pending_text.clear();
    }

    fn import_canvas_image(&mut self) {
        let path = self
            .shell
            .canvas_mode
            .ui
            .pending_image_path
            .trim()
            .to_string();
        if path.is_empty() {
            return;
        }

        let item_id = self.next_canvas_item_id();
        let size = Size {
            width: 320.0,
            height: 220.0,
        };
        self.shell
            .canvas_mode
            .document
            .items
            .push(CanvasItem::ImportedImage(ImportedImageItem {
                item_id: format!("image-{item_id}"),
                source: ImportedAssetSource::FilePath(path.clone().into()),
                bounds: Rect {
                    origin: self
                        .shifted_canvas_origin(60.0 * item_id as f32, 40.0 * item_id as f32),
                    size,
                },
            }));
        self.mark_canvas_dirty();
        self.shell.canvas_mode.ui.pending_image_path.clear();
    }

    fn import_canvas_pdf_page(&mut self) {
        let path = self
            .shell
            .canvas_mode
            .ui
            .pending_pdf_path
            .trim()
            .to_string();
        if path.is_empty() {
            return;
        }

        let item_id = self.next_canvas_item_id();
        self.shell
            .canvas_mode
            .document
            .items
            .push(CanvasItem::ImportedPdfPage(ImportedPdfPageItem {
                item_id: format!("pdf-page-{item_id}"),
                source: PdfSource::FilePath(path.clone().into()),
                page_index: self.shell.canvas_mode.ui.pending_pdf_page.saturating_sub(1),
                bounds: Rect {
                    origin: self
                        .shifted_canvas_origin(80.0 * item_id as f32, 48.0 * item_id as f32),
                    size: Size {
                        width: 420.0,
                        height: 560.0,
                    },
                },
                recolor_override: None,
            }));
        self.mark_canvas_dirty();
        self.shell.canvas_mode.ui.pending_pdf_path.clear();
    }

    fn shifted_canvas_origin(&self, dx: f32, dy: f32) -> Point {
        Point {
            x: self.shell.canvas_mode.document.viewport.origin.x + dx,
            y: self.shell.canvas_mode.document.viewport.origin.y + dy,
        }
    }

    fn next_canvas_item_id(&mut self) -> u64 {
        let next = self.shell.canvas_mode.ui.next_item_id;
        self.shell.canvas_mode.ui.next_item_id += 1;
        next
    }

    pub(super) fn add_canvas_note(&mut self) {
        let note = self
            .shell
            .shared_ui
            .annotation_tools
            .pending_note_text
            .trim()
            .to_string();
        if note.is_empty() {
            return;
        }

        let item_id = self.next_canvas_item_id();
        self.shell
            .canvas_mode
            .document
            .items
            .push(CanvasItem::Text(TextItem {
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
        self.mark_canvas_dirty();
        self.shell
            .shared_ui
            .annotation_tools
            .pending_note_text
            .clear();
    }

    fn apply_recolor_to_selected_canvas_pdf_pages(&mut self, enable: bool) {
        let selected_ids = match &self.shell.canvas_mode.document.selection {
            SelectionTarget::ItemIds(ids) => ids.clone(),
            _ => Vec::new(),
        };

        if selected_ids.is_empty() {
            return;
        }

        let profile = self
            .shell
            .pdf_mode
            .session
            .view
            .recolor
            .current_profile
            .clone()
            .unwrap_or_else(default_recolor_profile);

        for item in &mut self.shell.canvas_mode.document.items {
            let CanvasItem::ImportedPdfPage(page) = item else {
                continue;
            };
            if selected_ids.iter().any(|id| id == &page.item_id) {
                page.recolor_override = enable.then_some(profile.clone());
            }
        }
        self.mark_canvas_dirty();
    }

    fn export_canvas_svg(&mut self) {
        let export_path = self.shell.canvas_mode.ui.export_path.trim().to_string();
        if export_path.is_empty() {
            self.shell.canvas_mode.ui.export_status =
                "SVG export needs a target file path.".to_string();
            return;
        }

        let selected_items = self.selected_canvas_items();
        let target_items = if selected_items.is_empty() {
            self.shell
                .canvas_mode
                .document
                .items
                .iter()
                .collect::<Vec<_>>()
        } else {
            selected_items
        };

        if target_items.is_empty() {
            self.shell.canvas_mode.ui.export_status =
                "No canvas items are available for export.".to_string();
            return;
        }

        if let Some(reason) = self
            .services
            .canvas_export
            .first_incompatibility(&target_items)
        {
            self.shell.canvas_mode.ui.export_status =
                format!("SVG export unavailable for this target: {:?}.", reason);
            return;
        }

        let svg = self
            .services
            .canvas_export
            .build_svg_document(&target_items);
        match self
            .services
            .canvas_export
            .write_svg_document(&export_path, svg)
        {
            Ok(()) => {
                self.shell.canvas_mode.ui.export_status = format!("Exported SVG to {export_path}");
            }
            Err(error) => {
                self.shell.canvas_mode.ui.export_status = format!("SVG export failed: {error}");
            }
        }
    }

    fn selected_canvas_items(&self) -> Vec<&CanvasItem> {
        match &self.shell.canvas_mode.document.selection {
            SelectionTarget::ItemIds(ids) if !ids.is_empty() => self
                .shell
                .canvas_mode
                .document
                .items
                .iter()
                .filter(|item| ids.iter().any(|id| id == canvas_item_id(item)))
                .collect(),
            _ => Vec::new(),
        }
    }

    fn save_canvas_document(&mut self) {
        let path = self.shell.canvas_mode.ui.document_path.trim().to_string();
        if path.is_empty() {
            self.shell.canvas_mode.ui.save_status =
                "Canvas save needs a target file path.".to_string();
            return;
        }

        match self
            .services
            .persistence
            .save_canvas_document(&path, &self.shell.canvas_mode.document)
        {
            Ok(()) => {
                self.shell.canvas_mode.document.autosave.dirty = false;
                self.shell.canvas_mode.ui.save_status = format!("Saved canvas document to {path}");
            }
            Err(error) => {
                self.shell.canvas_mode.ui.save_status = format!("Canvas save failed: {error}");
            }
        }
    }

    fn load_canvas_document(&mut self) {
        let path = self.shell.canvas_mode.ui.document_path.trim().to_string();
        if path.is_empty() {
            self.shell.canvas_mode.ui.save_status =
                "Canvas load needs a source file path.".to_string();
            return;
        }

        match self.services.persistence.load_canvas_document(&path) {
            Ok(document) => {
                self.shell.canvas_mode.document = document;
                self.shell.canvas_mode.ui.active_stroke = None;
                self.shell.canvas_mode.ui.save_status =
                    format!("Loaded canvas document from {path}");
            }
            Err(error) => {
                self.shell.canvas_mode.ui.save_status = format!("Canvas load failed: {error}");
            }
        }
    }

    fn recover_canvas_document(&mut self) {
        match self.services.persistence.recover_canvas_document() {
            Ok(Some(document)) => {
                self.shell.canvas_mode.document = document;
                self.shell.canvas_mode.ui.active_stroke = None;
                self.shell.canvas_mode.ui.save_status =
                    "Recovered latest canvas autosave snapshot.".to_string();
            }
            Ok(None) => {
                self.shell.canvas_mode.ui.save_status =
                    "No canvas autosave snapshot was found.".to_string();
            }
            Err(error) => {
                self.shell.canvas_mode.ui.save_status = format!("Canvas recovery failed: {error}");
            }
        }
    }
}
