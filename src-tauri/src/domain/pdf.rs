use serde::{Deserialize, Serialize};

use crate::domain::canvas::DocumentVersion;
use crate::domain::reading::ReadingReliabilityState;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PdfStudyDocument {
    pub version: DocumentVersion,
    pub id: String,
    pub source_pdf_path: String,
    pub page_count: Option<u32>,
    pub current_page_index: u32,
    pub annotations: Vec<PdfPageAnnotationLayer>,
    pub recolor: PdfRecolorSettings,
    pub reading_cache: Vec<PdfPageReadingCache>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PdfPageAnnotationLayer {
    pub page_index: u32,
    pub strokes: Vec<PdfStrokeAnnotation>,
    pub notes: Vec<PdfTextNote>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PdfStrokeAnnotation {
    pub color: String,
    pub width: f32,
    pub points: Vec<PdfPoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PdfTextNote {
    pub text: String,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PdfPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PdfRecolorSettings {
    pub enabled: bool,
    pub foreground: String,
    pub background: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PdfPageReadingCache {
    pub page_index: u32,
    pub reliability: ReadingReliabilityState,
    pub source_kind: ReadingSourceKind,
    pub cache_key: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadingSourceKind {
    Native,
    Ocr,
}
