//! Core editable document and state models for `rpdf`.
//!
//! These types define the persistent and in-memory boundaries required by the
//! current specification without choosing a renderer, storage engine, or UI
//! toolkit.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceMode {
    InfiniteCanvas,
    PdfDocument,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasDocument {
    pub metadata: DocumentMetadata,
    pub viewport: ViewportState,
    pub background: BackgroundPattern,
    pub items: Vec<CanvasItem>,
    pub selection: SelectionTarget,
    pub autosave: AutosaveState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfDocumentSession {
    pub metadata: DocumentMetadata,
    pub source: PdfSource,
    pub viewport: PdfViewportState,
    pub annotations: Vec<PdfAnnotation>,
    pub reading_support: ReadingSupportState,
    pub view: PdfViewState,
    pub autosave: AutosaveState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub document_id: String,
    pub title: Option<String>,
    pub created_unix_ms: u64,
    pub modified_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewportState {
    pub origin: Point,
    pub zoom: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfViewportState {
    pub page_index: usize,
    pub scroll_offset: Point,
    pub zoom: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RgbaColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CanvasItem {
    PenStroke(PenStrokeItem),
    Text(TextItem),
    ImportedImage(ImportedImageItem),
    ImportedPdfPage(ImportedPdfPageItem),
}

impl CanvasItem {
    pub fn svg_compatibility(&self) -> SvgCompatibility {
        match self {
            CanvasItem::PenStroke(_) | CanvasItem::Text(_) => SvgCompatibility::Compatible,
            CanvasItem::ImportedImage(_) => {
                SvgCompatibility::Incompatible(IncompatibleExportReason::RasterContent)
            }
            CanvasItem::ImportedPdfPage(_) => {
                SvgCompatibility::Incompatible(IncompatibleExportReason::ImportedPdfPage)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PenStrokeItem {
    pub item_id: String,
    pub points: Vec<StrokePoint>,
    pub brush: BrushStyle,
    pub layer_role: AnnotationLayerRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StrokePoint {
    pub position: Point,
    pub pressure: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrushStyle {
    pub color: AnnotationAppearanceSet,
    pub width: f32,
    pub tool: PenToolKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PenToolKind {
    Ink,
    Highlighter,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextItem {
    pub item_id: String,
    pub bounds: Rect,
    pub text: String,
    pub style: TextStyle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f32,
    pub color: AnnotationAppearanceSet,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportedImageItem {
    pub item_id: String,
    pub source: ImportedAssetSource,
    pub bounds: Rect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportedPdfPageItem {
    pub item_id: String,
    pub source: PdfSource,
    pub page_index: usize,
    pub bounds: Rect,
    pub recolor_override: Option<RecolorProfile>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportedAssetSource {
    FilePath(PathBuf),
    ClipboardImage {
        width: usize,
        height: usize,
        pasted_unix_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SelectionTarget {
    None,
    WholeCanvas,
    ItemIds(Vec<String>),
    PdfPageSelection(PdfPageSelection),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfPageSelection {
    pub page_indices: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnnotationLayerRole {
    CanvasMarkup,
    PdfMarkup,
    ReadingHighlightFallback,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PdfAnnotation {
    PenStroke(PdfPenStrokeAnnotation),
    TextNote(PdfTextNote),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfPenStrokeAnnotation {
    pub annotation_id: String,
    pub page_index: usize,
    pub stroke: PenStrokeItem,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfTextNote {
    pub note_id: String,
    pub page_index: usize,
    pub anchor: Rect,
    pub text: String,
    pub style: TextStyle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadingSupportState {
    pub tts: TtsState,
    pub highlight_mode: HighlightMode,
    pub text_source: TextSupportSource,
    pub reliability: ReadingReliability,
    pub warning: Option<UserVisibleWarning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TtsState {
    pub playback: PlaybackState,
    pub active_span: Option<ReadingSpan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HighlightMode {
    Word,
    Line,
    Sentence,
    ManualFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextSupportSource {
    NativePdfText,
    OcrDerivedText,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadingReliability {
    Reliable,
    BestEffort,
    Unreliable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserVisibleWarning {
    pub code: WarningCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningCode {
    WeakNativeText,
    OcrFallbackUsed,
    ReadingSupportUnavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadingSpan {
    pub page_index: usize,
    pub bounds: Rect,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfViewState {
    pub recolor: RecolorState,
    pub annotation_visibility: AnnotationVisibility,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecolorState {
    pub current_profile: Option<RecolorProfile>,
    pub export_mode: RecolorExportMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecolorProfile {
    pub foreground: RgbaColor,
    pub background: RgbaColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecolorExportMode {
    PreserveOriginalAppearance,
    IncludeCurrentRecoloring,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationVisibility {
    pub normal_view: AnnotationPalette,
    pub recolored_view: AnnotationPalette,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationPalette {
    pub ink_color: RgbaColor,
    pub highlighter_color: RgbaColor,
    pub text_color: RgbaColor,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationAppearanceSet {
    pub normal_view: RgbaColor,
    pub recolored_view: RgbaColor,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BackgroundPattern {
    None,
    Dots(BackgroundPatternStyle),
    Lines(BackgroundPatternStyle),
    Squares(BackgroundPatternStyle),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackgroundPatternStyle {
    pub spacing: f32,
    pub line_width: f32,
    pub color: RgbaColor,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PdfSource {
    FilePath(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutosaveState {
    pub recovery_snapshot_id: Option<String>,
    pub dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportTarget {
    pub selection: SelectionTarget,
    pub scope: ExportScope,
    pub svg_compatibility: SvgCompatibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportScope {
    WholeCanvas,
    SelectedItems,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SvgCompatibility {
    Compatible,
    Incompatible(IncompatibleExportReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncompatibleExportReason {
    RasterContent,
    ImportedPdfPage,
    MixedUnsupportedSelection,
}
