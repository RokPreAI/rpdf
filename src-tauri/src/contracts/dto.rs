use serde::{Deserialize, Serialize};

use crate::domain::canvas::CanvasDocument;
use crate::domain::pdf::PdfStudyDocument;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppModeDto {
    Canvas,
    Pdf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadingReliabilityStateDto {
    NativeReliable,
    NativeWeak,
    OcrReliable,
    OcrWeak,
    Unavailable,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfBackendStatusDto {
    pub backend_key: String,
    pub backend_name: String,
    pub configured: bool,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppBootstrapDto {
    pub supported_modes: Vec<AppModeDto>,
    pub active_pdf_backend: PdfBackendStatusDto,
    pub reliability_states: Vec<ReadingReliabilityStateDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPdfDocumentRequestDto {
    pub document_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPdfDocumentResponseDto {
    pub document_path: String,
    pub document_name: String,
    pub page_count: Option<u32>,
    pub backend_ready: bool,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCanvasProjectRequestDto {
    pub file_path: String,
    pub document: CanvasDocument,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadCanvasProjectRequestDto {
    pub file_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePdfStudySessionRequestDto {
    pub file_path: String,
    pub document: PdfStudyDocument,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadPdfStudySessionRequestDto {
    pub file_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPdfPageRequestDto {
    pub document_path: String,
    pub page_index: u32,
    pub target_width: Option<u32>,
    pub target_height: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPdfPageResponseDto {
    pub page_index: u32,
    pub mime_type: String,
    pub data_base64: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractPdfTextRequestDto {
    pub document_path: String,
    pub page_index: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextSpanDto {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageTextExtractionDto {
    pub page_index: u32,
    pub reliability: ReadingReliabilityStateDto,
    pub warning: Option<String>,
    pub spans: Vec<TextSpanDto>,
}
