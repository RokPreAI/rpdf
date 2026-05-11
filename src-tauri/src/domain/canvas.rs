use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentVersion {
    pub major: u16,
    pub minor: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasDocument {
    pub version: DocumentVersion,
    pub id: String,
    pub background_pattern: BackgroundPattern,
    pub strokes: Vec<CanvasStroke>,
    #[serde(default)]
    pub shapes: Vec<CanvasShape>,
    pub images: Vec<CanvasImagePlacement>,
    #[serde(default)]
    pub pdf_pages: Vec<CanvasPdfPagePlacement>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundPattern {
    Dots,
    Lines,
    Squares,
    None,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasStroke {
    pub color: String,
    pub width: f32,
    #[serde(default)]
    pub order: Option<u32>,
    pub points: Vec<CanvasPoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanvasShapeKind {
    Line,
    Rectangle,
    Ellipse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasShapePoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasShape {
    pub id: String,
    pub kind: CanvasShapeKind,
    pub color: String,
    pub width: f32,
    #[serde(default)]
    pub order: Option<u32>,
    pub start: CanvasShapePoint,
    pub end: CanvasShapePoint,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasPoint {
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasImagePlacement {
    pub id: String,
    pub asset_path: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasPdfPagePlacement {
    pub id: String,
    pub source_pdf_path: String,
    pub page_index: u32,
    pub asset_path: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub recolor: CanvasPdfPageRecolor,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasPdfPageRecolor {
    pub enabled: bool,
    pub foreground: String,
    pub background: String,
}
