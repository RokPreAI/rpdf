use super::util;
use crate::model::{CanvasItem, HighlightMode, IncompatibleExportReason, SvgCompatibility};
use std::process::Command;

#[derive(Debug, Default)]
pub struct AppServices {
    pub reading_support: ReadingSupportService,
    pub canvas_export: CanvasExportService,
}

#[derive(Debug, Default)]
pub struct ReadingSupportService;

impl ReadingSupportService {
    pub fn pick_pdf_path(&self) -> Result<Option<String>, String> {
        util::pick_pdf_path()
    }

    pub fn best_effort_pdf_page_count(&self, path: &str) -> usize {
        util::best_effort_pdf_page_count(path)
    }

    pub fn best_effort_extract_pdf_text(&self, path: &str) -> String {
        util::best_effort_extract_pdf_text(path)
    }

    pub fn build_reading_spans(&self, text: &str, mode: HighlightMode) -> Vec<String> {
        util::build_reading_spans(text, mode)
    }

    pub fn start_local_tts(&self, text: &str) {
        let _ = Command::new("spd-say").arg(text).spawn();
    }
}

#[derive(Debug, Default)]
pub struct CanvasExportService;

impl CanvasExportService {
    pub fn first_incompatibility(
        &self,
        items: &[&CanvasItem],
    ) -> Option<IncompatibleExportReason> {
        items.iter().find_map(|item| match item.svg_compatibility() {
            SvgCompatibility::Compatible => None,
            SvgCompatibility::Incompatible(reason) => Some(reason),
        })
    }

    pub fn build_svg_document(&self, items: &[&CanvasItem]) -> String {
        util::build_svg_document(items)
    }

    pub fn write_svg_document(&self, export_path: &str, svg: String) -> Result<(), String> {
        std::fs::write(export_path, svg).map_err(|error| error.to_string())
    }
}
