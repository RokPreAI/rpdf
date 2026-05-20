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
    pub app_config: AppConfigDto,
    pub app_config_path: String,
    pub app_config_warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppConfigDto {
    pub version: u32,
    pub theme: AppThemeConfigDto,
    pub canvas: AppCanvasConfigDto,
    pub shortcuts: AppShortcutsConfigDto,
}

impl Default for AppConfigDto {
    fn default() -> Self {
        Self {
            version: 1,
            theme: AppThemeConfigDto::default(),
            canvas: AppCanvasConfigDto::default(),
            shortcuts: AppShortcutsConfigDto::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppThemeConfigDto {
    pub bg: String,
    pub bg_dark: String,
    pub bg_darker: String,
    pub bg_highlight: String,
    pub bg_panel: String,
    pub fg: String,
    pub fg_dark: String,
    pub fg_gutter: String,
    pub blue: String,
    pub cyan: String,
    pub green: String,
    pub yellow: String,
    pub orange: String,
    pub red: String,
    pub magenta: String,
    pub purple: String,
}

impl Default for AppThemeConfigDto {
    fn default() -> Self {
        Self {
            bg: "#1a1b26".to_string(),
            bg_dark: "#16161e".to_string(),
            bg_darker: "#0c0e14".to_string(),
            bg_highlight: "#292e42".to_string(),
            bg_panel: "rgba(15, 23, 42, 0.82)".to_string(),
            fg: "#c0caf5".to_string(),
            fg_dark: "#a9b1d6".to_string(),
            fg_gutter: "#3b4261".to_string(),
            blue: "#7aa2f7".to_string(),
            cyan: "#7dcfff".to_string(),
            green: "#9ece6a".to_string(),
            yellow: "#e0af68".to_string(),
            orange: "#ff9e64".to_string(),
            red: "#f7768e".to_string(),
            magenta: "#bb9af7".to_string(),
            purple: "#9d7cd8".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppCanvasConfigDto {
    pub background_pattern: String,
}

impl Default for AppCanvasConfigDto {
    fn default() -> Self {
        Self {
            background_pattern: "dots".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppShortcutsConfigDto {
    pub tools: AppToolShortcutsDto,
    pub colors: AppColorShortcutsDto,
}

impl Default for AppShortcutsConfigDto {
    fn default() -> Self {
        Self {
            tools: AppToolShortcutsDto::default(),
            colors: AppColorShortcutsDto::default(),
        }
    }
}

fn default_text_tool_shortcut() -> String {
    "t".to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppToolShortcutsDto {
    pub select: String,
    pub pan: String,
    pub pen: String,
    pub rectangle: String,
    pub ellipse: String,
    pub line: String,
    pub arrow: String,
    #[serde(default = "default_text_tool_shortcut")]
    pub text: String,
    pub eraser: String,
}

impl Default for AppToolShortcutsDto {
    fn default() -> Self {
        Self {
            select: "v".to_string(),
            pan: "h".to_string(),
            pen: "p".to_string(),
            rectangle: "r".to_string(),
            ellipse: "o".to_string(),
            line: "l".to_string(),
            arrow: "a".to_string(),
            text: default_text_tool_shortcut(),
            eraser: "e".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppColorShortcutsDto {
    pub fg: String,
    pub blue: String,
    pub cyan: String,
    pub green: String,
    pub yellow: String,
    pub orange: String,
    pub red: String,
    pub magenta: String,
    pub purple: String,
}

impl Default for AppColorShortcutsDto {
    fn default() -> Self {
        Self {
            fg: "1".to_string(),
            blue: "2".to_string(),
            cyan: "3".to_string(),
            green: "4".to_string(),
            yellow: "5".to_string(),
            orange: "6".to_string(),
            red: "7".to_string(),
            magenta: "8".to_string(),
            purple: "9".to_string(),
        }
    }
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
pub struct SaveSvgExportRequestDto {
    pub file_path: Option<String>,
    pub suggested_file_name: String,
    pub svg_markup: String,
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
pub struct SpeakTextRequestDto {
    pub text: String,
    pub rate: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalSpeechBackendDto {
    pub backend_key: String,
    pub backend_name: String,
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
    pub source_kind: String,
    pub reliability: ReadingReliabilityStateDto,
    pub warning: Option<String>,
    pub spans: Vec<TextSpanDto>,
}
