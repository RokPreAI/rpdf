use super::{AnnotationTool, PendingStroke};
use crate::model::{
    AnnotationAppearanceSet, AnnotationPalette, BackgroundPatternStyle, BrushStyle, CanvasItem,
    HighlightMode, PenToolKind, Point, RgbaColor, StrokePoint, TextStyle,
};
use eframe::egui;

pub(super) fn accent_color() -> RgbaColor {
    RgbaColor {
        red: 58,
        green: 128,
        blue: 247,
        alpha: 255,
    }
}

pub(super) fn muted_grid_color() -> RgbaColor {
    RgbaColor {
        red: 96,
        green: 104,
        blue: 122,
        alpha: 255,
    }
}

pub(super) fn default_palette() -> AnnotationPalette {
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

pub(super) fn latest_pressure(ui: &egui::Ui, rect: egui::Rect) -> Option<f32> {
    ui.input(|input| {
        input.events.iter().rev().find_map(|event| match event {
            egui::Event::Touch { pos, force, .. } if rect.contains(*pos) => *force,
            _ => None,
        })
    })
}

pub(super) fn push_stroke_point(stroke: &mut PendingStroke, position: Point, pressure: f32) {
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

pub(super) fn to_color32(color: RgbaColor) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(color.red, color.green, color.blue, color.alpha)
}

pub(super) fn from_color32(color: egui::Color32) -> RgbaColor {
    RgbaColor {
        red: color.r(),
        green: color.g(),
        blue: color.b(),
        alpha: color.a(),
    }
}

pub(super) fn default_background_style() -> BackgroundPatternStyle {
    BackgroundPatternStyle {
        spacing: 24.0,
        line_width: 1.0,
        color: muted_grid_color(),
    }
}

pub(super) fn file_label(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.display().to_string())
}

pub(super) fn best_effort_pdf_page_count(path: &str) -> usize {
    let Ok(bytes) = std::fs::read(path) else {
        return 1;
    };
    let content = String::from_utf8_lossy(&bytes);
    content.matches("/Type /Page").count().max(1)
}

pub(super) fn pick_pdf_path() -> Result<Option<String>, String> {
    Ok(rfd::FileDialog::new()
        .add_filter("PDF files", &["pdf"])
        .set_title("Open PDF")
        .pick_file()
        .map(|path| path.display().to_string()))
}

pub(super) fn centered_page_rect(rect: egui::Rect, zoom: f32) -> egui::Rect {
    let width = (560.0 * zoom.clamp(0.75, 1.4)).min((rect.width() - 48.0).max(120.0));
    let height = (760.0 * zoom.clamp(0.75, 1.4)).min((rect.height() - 48.0).max(160.0));
    egui::Rect::from_center_size(rect.center(), egui::vec2(width, height))
}

pub(super) fn pdf_page_to_screen(page_rect: egui::Rect, point: Point) -> egui::Pos2 {
    egui::pos2(page_rect.left() + point.x, page_rect.top() + point.y)
}

pub(super) fn screen_to_pdf_page(page_rect: egui::Rect, pos: egui::Pos2) -> Point {
    Point {
        x: pos.x - page_rect.left(),
        y: pos.y - page_rect.top(),
    }
}

pub(super) fn best_effort_extract_pdf_text(path: &str) -> String {
    if let Ok(output) = std::process::Command::new("pdftotext")
        .arg(path)
        .arg("-")
        .output()
        && output.status.success()
    {
        let extracted = String::from_utf8_lossy(&output.stdout);
        let normalized = extracted.split_whitespace().collect::<Vec<_>>().join(" ");
        if !normalized.is_empty() {
            return normalized;
        }
    }

    let Ok(bytes) = std::fs::read(path) else {
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

    printable.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn build_reading_spans(text: &str, mode: HighlightMode) -> Vec<String> {
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

pub(super) fn note_text_style() -> TextStyle {
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

pub(super) fn preview_brush(tool: AnnotationTool) -> BrushStyle {
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
        width: if tool == AnnotationTool::Highlighter {
            12.0
        } else {
            4.0
        },
        tool: match tool {
            AnnotationTool::Ink => PenToolKind::Ink,
            AnnotationTool::Highlighter => PenToolKind::Highlighter,
        },
    }
}

pub(super) fn default_recolor_profile() -> crate::model::RecolorProfile {
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

pub(super) fn render_palette_editor(
    ui: &mut egui::Ui,
    label: &str,
    palette: &mut AnnotationPalette,
) {
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

pub(super) fn canvas_item_id(item: &CanvasItem) -> &str {
    match item {
        CanvasItem::PenStroke(stroke) => &stroke.item_id,
        CanvasItem::Text(text) => &text.item_id,
        CanvasItem::ImportedImage(image) => &image.item_id,
        CanvasItem::ImportedPdfPage(page) => &page.item_id,
    }
}

pub(super) fn item_kind_label(item: &CanvasItem) -> &'static str {
    match item {
        CanvasItem::PenStroke(_) => "stroke",
        CanvasItem::Text(_) => "text",
        CanvasItem::ImportedImage(_) => "image",
        CanvasItem::ImportedPdfPage(_) => "pdf-page",
    }
}

pub(super) fn build_svg_document(items: &[&CanvasItem]) -> String {
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
