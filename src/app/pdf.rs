use super::util::{
    centered_page_rect, default_recolor_profile, file_label, from_color32, note_text_style,
    pdf_page_to_screen, preview_brush, render_palette_editor, screen_to_pdf_page, to_color32,
};
use super::*;
impl RpdfApp {
    pub(super) fn render_pdf_workspace(&mut self, ui: &mut egui::Ui) {
        ui.heading("PDF Mode");
        self.render_status_banner(
            ui,
            BannerTone::Info,
            "Study flow",
            "Open a document, read or listen, then annotate in place. PDF Mode stays document-focused so reading support and markup do not drift into canvas behavior.",
        );
        ui.add_space(8.0);
        self.render_autosave_banner(
            ui,
            self.shell.pdf_mode.session.autosave.dirty,
            self.services.persistence.has_pdf_recovery_snapshot(),
            "PDF session",
        );
        ui.add_space(8.0);
        self.render_annotation_toolbar(ui, WorkspaceMode::PdfDocument);
        ui.add_space(8.0);
        self.render_pdf_toolbar(ui);
        ui.add_space(8.0);

        let available = ui.available_size();
        let (response, painter) = ui.allocate_painter(available, egui::Sense::click_and_drag());
        let rect = response.rect;
        painter.rect_filled(rect, 12.0, egui::Color32::from_rgb(20, 21, 25));

        let page_rect = centered_page_rect(rect, self.shell.pdf_mode.session.viewport.zoom);
        self.handle_pdf_drawing(ui, page_rect);
        let (page_fill, page_stroke) = self.current_pdf_page_colors();
        painter.rect_filled(page_rect, 8.0, page_fill);
        painter.rect_stroke(
            page_rect,
            8.0,
            egui::Stroke::new(2.0, page_stroke),
            egui::StrokeKind::Middle,
        );

        let source_label = match &self.shell.pdf_mode.session.source {
            PdfSource::FilePath(path) if !path.as_os_str().is_empty() => file_label(path),
            _ => "No PDF opened".to_string(),
        };

        painter.text(
            page_rect.center_top() + egui::vec2(0.0, 28.0),
            egui::Align2::CENTER_TOP,
            format!(
                "{}\nPage {} of {}",
                source_label,
                self.shell.pdf_mode.session.viewport.page_index + 1,
                self.shell.pdf_mode.ui.page_count
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

    fn render_pdf_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            egui::CollapsingHeader::new("Files and recovery")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("PDF path:");
                        ui.text_edit_singleline(&mut self.shell.pdf_mode.ui.pending_open_path);
                        if ui.button("Open PDF").clicked() {
                            self.open_pdf_document();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Save path:");
                        ui.text_edit_singleline(&mut self.shell.pdf_mode.ui.document_path);
                        if ui.button("Save session").clicked() {
                            self.save_pdf_session();
                        }
                        if ui.button("Load session").clicked() {
                            self.load_pdf_session();
                        }
                        if ui
                            .add_enabled(
                                self.services.persistence.has_pdf_recovery_snapshot(),
                                egui::Button::new("Recover autosave"),
                            )
                            .clicked()
                        {
                            self.recover_pdf_session();
                        }
                    });

                    self.render_feedback_message(ui, &self.shell.pdf_mode.ui.status_message);
                });

            egui::CollapsingHeader::new("Navigation and view")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Previous").clicked() {
                            self.step_pdf_page(-1);
                        }
                        if ui.button("Next").clicked() {
                            self.step_pdf_page(1);
                        }

                        let mut display_page = self.shell.pdf_mode.session.viewport.page_index + 1;
                        ui.label("Page:");
                        if ui
                            .add(
                                egui::DragValue::new(&mut display_page)
                                    .range(1..=self.shell.pdf_mode.ui.page_count.max(1)),
                            )
                            .changed()
                        {
                            self.shell.pdf_mode.session.viewport.page_index = display_page
                                .saturating_sub(1)
                                .min(self.shell.pdf_mode.ui.page_count.saturating_sub(1));
                            self.mark_pdf_dirty();
                        }

                        ui.label(format!("of {}", self.shell.pdf_mode.ui.page_count));
                        ui.separator();
                        ui.label("Document-focused workspace");
                    });

                    let mut recolor_enabled = self
                        .shell
                        .pdf_mode
                        .session
                        .view
                        .recolor
                        .current_profile
                        .is_some();
                    if ui
                        .checkbox(&mut recolor_enabled, "Enable recolor view")
                        .changed()
                    {
                        if recolor_enabled {
                            self.shell.pdf_mode.session.view.recolor.current_profile =
                                Some(default_recolor_profile());
                        } else {
                            self.shell.pdf_mode.session.view.recolor.current_profile = None;
                        }
                        self.mark_pdf_dirty();
                    }

                    let mut recolor_profile_changed = false;
                    if let Some(profile) = self
                        .shell
                        .pdf_mode
                        .session
                        .view
                        .recolor
                        .current_profile
                        .as_mut()
                    {
                        ui.horizontal(|ui| {
                            ui.label("Foreground");
                            let mut foreground = to_color32(profile.foreground);
                            if ui.color_edit_button_srgba(&mut foreground).changed() {
                                profile.foreground = from_color32(foreground);
                                recolor_profile_changed = true;
                            }

                            ui.label("Background");
                            let mut background = to_color32(profile.background);
                            if ui.color_edit_button_srgba(&mut background).changed() {
                                profile.background = from_color32(background);
                                recolor_profile_changed = true;
                            }
                        });
                    }
                    if recolor_profile_changed {
                        self.mark_pdf_dirty();
                    }

                    ui.horizontal(|ui| {
                        ui.label("PDF export recolor:");
                        let preserve_changed = ui
                            .selectable_value(
                                &mut self.shell.pdf_mode.session.view.recolor.export_mode,
                                RecolorExportMode::PreserveOriginalAppearance,
                                "Preserve original",
                            )
                            .changed();
                        let recolor_changed = ui
                            .selectable_value(
                                &mut self.shell.pdf_mode.session.view.recolor.export_mode,
                                RecolorExportMode::IncludeCurrentRecoloring,
                                "Include recolor",
                            )
                            .changed();
                        if preserve_changed || recolor_changed {
                            self.mark_pdf_dirty();
                        }
                    });

                    ui.collapsing("Annotation palettes", |ui| {
                        render_palette_editor(
                            ui,
                            "Normal",
                            &mut self
                                .shell
                                .pdf_mode
                                .session
                                .view
                                .annotation_visibility
                                .normal_view,
                        );
                        render_palette_editor(
                            ui,
                            "Recolored",
                            &mut self
                                .shell
                                .pdf_mode
                                .session
                                .view
                                .annotation_visibility
                                .recolored_view,
                        );
                    });
                });

            egui::CollapsingHeader::new("Reading support")
                .default_open(true)
                .show(ui, |ui| {
                    self.render_pdf_reading_guidance(ui);

                    ui.horizontal(|ui| {
                        ui.label("Highlight mode:");
                        ui.selectable_value(
                            &mut self.shell.pdf_mode.session.reading_support.highlight_mode,
                            HighlightMode::Word,
                            "Word",
                        );
                        ui.selectable_value(
                            &mut self.shell.pdf_mode.session.reading_support.highlight_mode,
                            HighlightMode::Line,
                            "Line",
                        );
                        ui.selectable_value(
                            &mut self.shell.pdf_mode.session.reading_support.highlight_mode,
                            HighlightMode::Sentence,
                            "Sentence",
                        );
                        ui.selectable_value(
                            &mut self.shell.pdf_mode.session.reading_support.highlight_mode,
                            HighlightMode::ManualFallback,
                            "Manual fallback",
                        );
                    });

                    ui.horizontal(|ui| {
                        if ui.button("Start TTS").clicked() {
                            self.start_pdf_tts();
                        }
                        if ui.button("Stop TTS").clicked() {
                            self.stop_pdf_tts();
                        }
                        ui.label(format!(
                            "Playback: {:?}",
                            self.shell.pdf_mode.session.reading_support.tts.playback
                        ));
                    });

                    ui.label(format!(
                        "Text source: {:?} | Reliability: {:?}",
                        self.shell.pdf_mode.session.reading_support.text_source,
                        self.shell.pdf_mode.session.reading_support.reliability
                    ));
                });
        });
    }

    fn render_pdf_reading_guidance(&self, ui: &mut egui::Ui) {
        let source = self.shell.pdf_mode.session.reading_support.text_source;
        let playback = self.shell.pdf_mode.session.reading_support.tts.playback;

        let (tone, title, body) = match source {
            TextSupportSource::NativePdfText => (
                BannerTone::Success,
                "Reading support",
                if playback == PlaybackState::Playing {
                    "Native PDF text is active. Highlighting should follow the document with the selected mode."
                } else {
                    "Native PDF text is available. Start TTS when you want follow-along highlighting."
                },
            ),
            TextSupportSource::OcrDerivedText => (
                BannerTone::Warning,
                "Reading support",
                "OCR fallback is active. The app keeps annotation available, but highlight precision falls back to a coarse manual mode.",
            ),
            TextSupportSource::Unavailable => (
                BannerTone::Info,
                "Reading support",
                "Open a PDF, then start TTS to test native text extraction before relying on OCR fallback.",
            ),
        };
        self.render_status_banner(ui, tone, title, body);

        if let Some(warning) = &self.shell.pdf_mode.session.reading_support.warning {
            self.render_status_banner(ui, BannerTone::Warning, "Reading warning", &warning.message);
        }
    }

    fn handle_pdf_drawing(&mut self, ui: &egui::Ui, page_rect: egui::Rect) {
        let pointer_pos = ui.input(|input| input.pointer.interact_pos());
        let primary_down = ui.input(|input| input.pointer.primary_down());

        if primary_down {
            if let Some(screen_pos) = pointer_pos.filter(|pos| page_rect.contains(*pos)) {
                let page_point = screen_to_pdf_page(page_rect, screen_pos);
                let pressure = super::util::latest_pressure(ui, page_rect).unwrap_or(0.5);
                let active = self
                    .shell
                    .pdf_mode
                    .ui
                    .active_stroke
                    .get_or_insert_with(|| PendingStroke { points: Vec::new() });
                super::util::push_stroke_point(active, page_point, pressure);
                return;
            }
        }

        if !primary_down {
            self.commit_pdf_stroke();
        }
    }

    fn commit_pdf_stroke(&mut self) {
        let Some(active_stroke) = self.shell.pdf_mode.ui.active_stroke.take() else {
            return;
        };

        if active_stroke.points.len() < 2 {
            return;
        }

        let id = self.next_pdf_annotation_id();
        let tool = self.shell.shared_ui.annotation_tools.current_tool;

        self.shell
            .pdf_mode
            .session
            .annotations
            .push(crate::model::PdfAnnotation::PenStroke(
                PdfPenStrokeAnnotation {
                    annotation_id: format!("pdf-stroke-{id}"),
                    page_index: self.shell.pdf_mode.session.viewport.page_index,
                    stroke: PenStrokeItem {
                        item_id: format!("pdf-stroke-item-{id}"),
                        points: active_stroke.points,
                        brush: preview_brush(tool),
                        layer_role: AnnotationLayerRole::PdfMarkup,
                    },
                },
            ));
        self.mark_pdf_dirty();
    }

    fn paint_pdf_annotations(&self, painter: &egui::Painter, page_rect: egui::Rect) {
        for annotation in &self.shell.pdf_mode.session.annotations {
            match annotation {
                crate::model::PdfAnnotation::PenStroke(stroke)
                    if stroke.page_index == self.shell.pdf_mode.session.viewport.page_index =>
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
                    if note.page_index == self.shell.pdf_mode.session.viewport.page_index =>
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
        let Some(stroke) = &self.shell.pdf_mode.ui.active_stroke else {
            return;
        };
        let preview = preview_brush(self.shell.shared_ui.annotation_tools.current_tool);
        self.paint_pdf_stroke(
            painter,
            page_rect,
            &stroke.points,
            preview.width,
            self.pdf_annotation_color_for_brush(&preview),
        );
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

    fn open_pdf_document(&mut self) {
        let mut path = self.shell.pdf_mode.ui.pending_open_path.trim().to_string();
        if path.is_empty() {
            match self.services.reading_support.pick_pdf_path() {
                Ok(Some(selected_path)) => {
                    path = selected_path;
                    self.shell.pdf_mode.ui.pending_open_path = path.clone();
                }
                Ok(None) => {
                    self.shell.pdf_mode.ui.status_message = "PDF open canceled.".to_string();
                    return;
                }
                Err(error) => {
                    self.shell.pdf_mode.ui.status_message = format!("PDF picker failed: {error}");
                    return;
                }
            }
        }

        let pdf_path = std::path::Path::new(&path);
        if !pdf_path.exists() {
            self.shell.pdf_mode.ui.status_message = format!("PDF path does not exist: {path}");
            return;
        }

        self.shell.pdf_mode.session.source = PdfSource::FilePath(path.clone().into());
        self.shell.pdf_mode.session.metadata.title = Some(file_label(pdf_path));
        self.shell.pdf_mode.session.viewport.page_index = 0;
        self.shell.pdf_mode.session.viewport.scroll_offset = Point { x: 0.0, y: 0.0 };
        self.shell.pdf_mode.ui.page_count = self
            .services
            .reading_support
            .best_effort_pdf_page_count(&path)
            .max(1);
        self.shell.pdf_mode.ui.document_path = path.clone();
        self.stop_pdf_tts();
        self.shell.pdf_mode.session.reading_support.text_source = TextSupportSource::Unavailable;
        self.shell.pdf_mode.session.reading_support.reliability = ReadingReliability::BestEffort;
        self.shell.pdf_mode.session.reading_support.warning =
            Some(crate::model::UserVisibleWarning {
                code: WarningCode::ReadingSupportUnavailable,
                message: "PDF opened. Start TTS to evaluate native text and OCR fallback."
                    .to_string(),
            });
        self.mark_pdf_dirty();
        self.startup.last_opened_path = Some(path);
        self.shell.pdf_mode.ui.status_message = "Opened PDF document.".to_string();
    }

    fn step_pdf_page(&mut self, delta: isize) {
        let current = self.shell.pdf_mode.session.viewport.page_index as isize;
        let max = self.shell.pdf_mode.ui.page_count.saturating_sub(1) as isize;
        self.shell.pdf_mode.session.viewport.page_index = (current + delta).clamp(0, max) as usize;
        self.mark_pdf_dirty();
    }

    pub(super) fn add_pdf_note(&mut self) {
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

        let id = self.next_pdf_annotation_id();
        self.shell
            .pdf_mode
            .session
            .annotations
            .push(crate::model::PdfAnnotation::TextNote(PdfTextNote {
                note_id: format!("pdf-note-{id}"),
                page_index: self.shell.pdf_mode.session.viewport.page_index,
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
        self.mark_pdf_dirty();
        self.shell
            .shared_ui
            .annotation_tools
            .pending_note_text
            .clear();
    }

    fn next_pdf_annotation_id(&mut self) -> u64 {
        let next = self.shell.pdf_mode.ui.next_annotation_id;
        self.shell.pdf_mode.ui.next_annotation_id += 1;
        next
    }

    fn current_annotation_palette(&self) -> &AnnotationPalette {
        if self
            .shell
            .pdf_mode
            .session
            .view
            .recolor
            .current_profile
            .is_some()
        {
            &self
                .shell
                .pdf_mode
                .session
                .view
                .annotation_visibility
                .recolored_view
        } else {
            &self
                .shell
                .pdf_mode
                .session
                .view
                .annotation_visibility
                .normal_view
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
        if let Some(profile) = &self.shell.pdf_mode.session.view.recolor.current_profile {
            (
                to_color32(profile.background),
                to_color32(profile.foreground),
            )
        } else {
            (
                egui::Color32::from_rgb(242, 242, 238),
                egui::Color32::from_rgb(78, 83, 92),
            )
        }
    }

    fn start_pdf_tts(&mut self) {
        let path = match &self.shell.pdf_mode.session.source {
            PdfSource::FilePath(path) if !path.as_os_str().is_empty() => path.clone(),
            _ => {
                self.shell.pdf_mode.session.reading_support.warning =
                    Some(crate::model::UserVisibleWarning {
                        code: WarningCode::ReadingSupportUnavailable,
                        message: "Open a PDF before starting TTS.".to_string(),
                    });
                return;
            }
        };

        let resolution = self.services.reading_support.resolve_reading_support(
            &path.to_string_lossy(),
            self.shell.pdf_mode.session.reading_support.highlight_mode,
        );

        self.shell.pdf_mode.session.reading_support.text_source = resolution.text_source;
        self.shell.pdf_mode.session.reading_support.reliability = resolution.reliability;
        self.shell.pdf_mode.session.reading_support.highlight_mode =
            resolution.effective_highlight_mode;
        self.shell.pdf_mode.session.reading_support.warning = resolution.warning;

        if resolution.spans.is_empty() {
            self.shell.pdf_mode.ui.reading_session = None;
            self.shell.pdf_mode.session.reading_support.tts.playback = PlaybackState::Stopped;
            self.shell.pdf_mode.session.reading_support.tts.active_span = None;
            return;
        }

        let excerpt = resolution.spans.join(" ");
        self.services.reading_support.start_local_tts(&excerpt);

        self.shell.pdf_mode.session.reading_support.tts.playback = PlaybackState::Playing;
        self.shell.pdf_mode.ui.reading_session = Some(ReadingPlaybackSession {
            spans: resolution.spans,
            started_at: Instant::now(),
            span_duration_ms: 1200,
        });
    }

    fn stop_pdf_tts(&mut self) {
        self.shell.pdf_mode.session.reading_support.tts.playback = PlaybackState::Stopped;
        self.shell.pdf_mode.session.reading_support.tts.active_span = None;
        self.shell.pdf_mode.ui.reading_session = None;
    }

    pub(super) fn tick_pdf_reading_support(&mut self) {
        let Some(session) = &self.shell.pdf_mode.ui.reading_session else {
            return;
        };

        if self.shell.pdf_mode.session.reading_support.tts.playback != PlaybackState::Playing {
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
        self.shell.pdf_mode.session.reading_support.tts.active_span =
            Some(crate::model::ReadingSpan {
                page_index: self.shell.pdf_mode.session.viewport.page_index,
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
        let Some(span) = &self.shell.pdf_mode.session.reading_support.tts.active_span else {
            return;
        };
        if span.page_index != self.shell.pdf_mode.session.viewport.page_index {
            return;
        }

        let highlight_rect = egui::Rect::from_min_size(
            pdf_page_to_screen(page_rect, span.bounds.origin),
            egui::vec2(span.bounds.size.width, span.bounds.size.height),
        );
        let color = match self.shell.pdf_mode.session.reading_support.highlight_mode {
            HighlightMode::Word => egui::Color32::from_rgba_premultiplied(255, 214, 10, 180),
            HighlightMode::Line => egui::Color32::from_rgba_premultiplied(58, 128, 247, 90),
            HighlightMode::Sentence => egui::Color32::from_rgba_premultiplied(96, 104, 122, 120),
            HighlightMode::ManualFallback => {
                egui::Color32::from_rgba_premultiplied(255, 255, 255, 40)
            }
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

    fn save_pdf_session(&mut self) {
        let path = self.shell.pdf_mode.ui.document_path.trim().to_string();
        if path.is_empty() {
            self.shell.pdf_mode.ui.status_message =
                "PDF session save needs a target file path.".to_string();
            return;
        }

        match self
            .services
            .persistence
            .save_pdf_session(&path, &self.shell.pdf_mode.session)
        {
            Ok(()) => {
                self.shell.pdf_mode.session.autosave.dirty = false;
                self.shell.pdf_mode.ui.status_message = format!("Saved PDF session to {path}");
            }
            Err(error) => {
                self.shell.pdf_mode.ui.status_message = format!("PDF session save failed: {error}");
            }
        }
    }

    fn load_pdf_session(&mut self) {
        let path = self.shell.pdf_mode.ui.document_path.trim().to_string();
        if path.is_empty() {
            self.shell.pdf_mode.ui.status_message =
                "PDF session load needs a source file path.".to_string();
            return;
        }

        match self.services.persistence.load_pdf_session(&path) {
            Ok(session) => {
                self.shell.pdf_mode.ui.pending_open_path = match &session.source {
                    PdfSource::FilePath(path) => path.display().to_string(),
                };
                self.shell.pdf_mode.ui.page_count = match &session.source {
                    PdfSource::FilePath(path) if path.exists() => self
                        .services
                        .reading_support
                        .best_effort_pdf_page_count(&path.display().to_string())
                        .max(1),
                    _ => 1,
                };
                self.shell.pdf_mode.session = session;
                self.shell.pdf_mode.ui.active_stroke = None;
                self.shell.pdf_mode.ui.reading_session = None;
                self.shell.pdf_mode.ui.status_message = format!("Loaded PDF session from {path}");
            }
            Err(error) => {
                self.shell.pdf_mode.ui.status_message = format!("PDF session load failed: {error}");
            }
        }
    }

    fn recover_pdf_session(&mut self) {
        match self.services.persistence.recover_pdf_session() {
            Ok(Some(session)) => {
                self.shell.pdf_mode.ui.pending_open_path = match &session.source {
                    PdfSource::FilePath(path) => path.display().to_string(),
                };
                self.shell.pdf_mode.ui.page_count = match &session.source {
                    PdfSource::FilePath(path) if path.exists() => self
                        .services
                        .reading_support
                        .best_effort_pdf_page_count(&path.display().to_string())
                        .max(1),
                    _ => 1,
                };
                self.shell.pdf_mode.session = session;
                self.shell.pdf_mode.ui.active_stroke = None;
                self.shell.pdf_mode.ui.reading_session = None;
                self.shell.pdf_mode.ui.status_message =
                    "Recovered latest PDF autosave snapshot.".to_string();
            }
            Ok(None) => {
                self.shell.pdf_mode.ui.status_message =
                    "No PDF autosave snapshot was found.".to_string();
            }
            Err(error) => {
                self.shell.pdf_mode.ui.status_message = format!("PDF recovery failed: {error}");
            }
        }
    }
}
