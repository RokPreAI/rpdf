use super::util;
use crate::model::{
    CanvasDocument, CanvasItem, HighlightMode, IncompatibleExportReason, PdfDocumentSession,
    ReadingReliability, SvgCompatibility, TextSupportSource, UserVisibleWarning, WarningCode,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Default)]
pub struct AppServices {
    pub reading_support: ReadingSupportService,
    pub canvas_export: CanvasExportService,
    pub persistence: PersistenceService,
}

#[derive(Debug, Default)]
pub struct ReadingSupportService;

#[derive(Debug, Clone)]
pub struct ReadingSupportResolution {
    pub spans: Vec<String>,
    pub text_source: TextSupportSource,
    pub reliability: ReadingReliability,
    pub effective_highlight_mode: HighlightMode,
    pub warning: Option<UserVisibleWarning>,
}

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

    pub fn resolve_reading_support(
        &self,
        path: &str,
        preferred_mode: HighlightMode,
    ) -> ReadingSupportResolution {
        let native_text = self.best_effort_extract_pdf_text(path);
        let native_quality = native_text_quality(&native_text);
        if native_quality.is_usable() {
            return ReadingSupportResolution {
                spans: self.build_reading_spans(&native_text, preferred_mode),
                text_source: TextSupportSource::NativePdfText,
                reliability: native_quality.reliability(),
                effective_highlight_mode: preferred_mode,
                warning: None,
            };
        }

        let ocr_text = self.best_effort_ocr_pdf_text(path);
        if native_text_quality(&ocr_text).is_usable() {
            let effective_highlight_mode = HighlightMode::ManualFallback;
            return ReadingSupportResolution {
                spans: self.build_reading_spans(&ocr_text, effective_highlight_mode),
                text_source: TextSupportSource::OcrDerivedText,
                reliability: ReadingReliability::BestEffort,
                effective_highlight_mode,
                warning: Some(UserVisibleWarning {
                    code: WarningCode::OcrFallbackUsed,
                    message: "Native PDF text was too weak, so offline OCR fallback was used. Highlighting is running in manual fallback mode.".to_string(),
                }),
            };
        }

        ReadingSupportResolution {
            spans: Vec::new(),
            text_source: TextSupportSource::Unavailable,
            reliability: ReadingReliability::Unreliable,
            effective_highlight_mode: HighlightMode::ManualFallback,
            warning: Some(UserVisibleWarning {
                code: WarningCode::ReadingSupportUnavailable,
                message: "Native PDF text was too weak and offline OCR could not recover usable reading text.".to_string(),
            }),
        }
    }

    fn best_effort_ocr_pdf_text(&self, path: &str) -> String {
        let temp_dir = match create_ocr_temp_dir() {
            Ok(path) => path,
            Err(_) => return String::new(),
        };

        let render_prefix = temp_dir.join("page");
        let render_status = Command::new("pdftoppm")
            .arg("-f")
            .arg("1")
            .arg("-l")
            .arg("3")
            .arg("-r")
            .arg("150")
            .arg("-gray")
            .arg("-png")
            .arg(path)
            .arg(&render_prefix)
            .status();

        if !matches!(render_status, Ok(status) if status.success()) {
            let _ = std::fs::remove_dir_all(&temp_dir);
            return String::new();
        }

        let mut image_paths = match std::fs::read_dir(&temp_dir) {
            Ok(entries) => entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|entry| entry.extension().and_then(|ext| ext.to_str()) == Some("png"))
                .collect::<Vec<_>>(),
            Err(_) => {
                let _ = std::fs::remove_dir_all(&temp_dir);
                return String::new();
            }
        };
        image_paths.sort();

        let mut combined = Vec::new();
        for image_path in image_paths {
            let output = Command::new("tesseract")
                .arg(&image_path)
                .arg("stdout")
                .arg("--psm")
                .arg("6")
                .output();
            let Ok(output) = output else {
                continue;
            };
            if !output.status.success() {
                continue;
            }

            let recognized = String::from_utf8_lossy(&output.stdout);
            let normalized = normalize_extracted_text(&recognized);
            if !normalized.is_empty() {
                combined.push(normalized);
            }
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
        combined.join(" ")
    }
}

#[derive(Debug, Default)]
pub struct CanvasExportService;

#[derive(Debug, Default)]
pub struct PersistenceService;

impl CanvasExportService {
    pub fn first_incompatibility(&self, items: &[&CanvasItem]) -> Option<IncompatibleExportReason> {
        items
            .iter()
            .find_map(|item| match item.svg_compatibility() {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CanvasSaveFile {
    schema_version: u32,
    saved_unix_ms: u64,
    document: CanvasDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PdfSaveFile {
    schema_version: u32,
    saved_unix_ms: u64,
    session: PdfDocumentSession,
}

impl PersistenceService {
    pub fn save_canvas_document(
        &self,
        export_path: &str,
        document: &CanvasDocument,
    ) -> Result<(), String> {
        let mut saved_document = document.clone();
        saved_document.autosave.dirty = false;
        self.write_json_file(
            export_path,
            &CanvasSaveFile {
                schema_version: 1,
                saved_unix_ms: current_unix_ms(),
                document: saved_document,
            },
        )
    }

    pub fn load_canvas_document(&self, path: &str) -> Result<CanvasDocument, String> {
        let mut save_file: CanvasSaveFile = self.read_json_file(path)?;
        if save_file.schema_version != 1 {
            return Err(format!(
                "Unsupported canvas save schema version: {}",
                save_file.schema_version
            ));
        }
        save_file.document.autosave.dirty = false;
        Ok(save_file.document)
    }

    pub fn save_pdf_session(
        &self,
        export_path: &str,
        session: &PdfDocumentSession,
    ) -> Result<(), String> {
        let mut saved_session = session.clone();
        saved_session.autosave.dirty = false;
        self.write_json_file(
            export_path,
            &PdfSaveFile {
                schema_version: 1,
                saved_unix_ms: current_unix_ms(),
                session: saved_session,
            },
        )
    }

    pub fn load_pdf_session(&self, path: &str) -> Result<PdfDocumentSession, String> {
        let mut save_file: PdfSaveFile = self.read_json_file(path)?;
        if save_file.schema_version != 1 {
            return Err(format!(
                "Unsupported PDF save schema version: {}",
                save_file.schema_version
            ));
        }
        save_file.session.autosave.dirty = false;
        Ok(save_file.session)
    }

    pub fn write_canvas_recovery_snapshot(
        &self,
        document: &CanvasDocument,
    ) -> Result<String, String> {
        let path = self.recovery_path("canvas-recovery.json");
        self.save_canvas_document(&path, document)?;
        Ok(path)
    }

    pub fn write_pdf_recovery_snapshot(
        &self,
        session: &PdfDocumentSession,
    ) -> Result<String, String> {
        let path = self.recovery_path("pdf-recovery.json");
        self.save_pdf_session(&path, session)?;
        Ok(path)
    }

    pub fn has_canvas_recovery_snapshot(&self) -> bool {
        std::path::Path::new(&self.recovery_path("canvas-recovery.json")).exists()
    }

    pub fn has_pdf_recovery_snapshot(&self) -> bool {
        std::path::Path::new(&self.recovery_path("pdf-recovery.json")).exists()
    }

    pub fn recover_canvas_document(&self) -> Result<Option<CanvasDocument>, String> {
        let path = self.recovery_path("canvas-recovery.json");
        if !std::path::Path::new(&path).exists() {
            return Ok(None);
        }
        self.load_canvas_document(&path).map(Some)
    }

    pub fn recover_pdf_session(&self) -> Result<Option<PdfDocumentSession>, String> {
        let path = self.recovery_path("pdf-recovery.json");
        if !std::path::Path::new(&path).exists() {
            return Ok(None);
        }
        self.load_pdf_session(&path).map(Some)
    }

    fn write_json_file<T: Serialize>(&self, path: &str, value: &T) -> Result<(), String> {
        let output_path = PathBuf::from(path);
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let serialized = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
        std::fs::write(output_path, serialized).map_err(|error| error.to_string())
    }

    fn read_json_file<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, String> {
        let content = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
        serde_json::from_str(&content).map_err(|error| error.to_string())
    }

    fn recovery_path(&self, file_name: &str) -> String {
        recovery_root().join(file_name).display().to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextQuality {
    Unusable,
    Weak,
    Good,
}

impl TextQuality {
    fn is_usable(self) -> bool {
        !matches!(self, Self::Unusable)
    }

    fn reliability(self) -> ReadingReliability {
        match self {
            Self::Good => ReadingReliability::Reliable,
            Self::Weak => ReadingReliability::BestEffort,
            Self::Unusable => ReadingReliability::Unreliable,
        }
    }
}

fn native_text_quality(text: &str) -> TextQuality {
    let normalized = normalize_extracted_text(text);
    if normalized.is_empty() {
        return TextQuality::Unusable;
    }

    let words = normalized.split_whitespace().collect::<Vec<_>>();
    let word_count = words.len();
    let alpha_count = normalized
        .chars()
        .filter(|character| character.is_ascii_alphabetic())
        .count();
    let long_word_count = words.iter().filter(|word| word.len() >= 20).count();
    let average_word_length =
        words.iter().map(|word| word.len()).sum::<usize>() as f32 / word_count.max(1) as f32;

    if word_count < 12 || alpha_count < 48 {
        return TextQuality::Unusable;
    }

    if average_word_length > 9.5 || long_word_count * 3 > word_count {
        return TextQuality::Weak;
    }

    TextQuality::Good
}

fn normalize_extracted_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn create_ocr_temp_dir() -> Result<PathBuf, std::io::Error> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let path = std::env::temp_dir().join(format!("rpdf-ocr-{}-{}", std::process::id(), unique));
    std::fs::create_dir(&path)?;
    Ok(path)
}

fn recovery_root() -> PathBuf {
    if let Ok(state_home) = std::env::var("XDG_STATE_HOME") {
        return PathBuf::from(state_home).join("rpdf").join("recovery");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("rpdf")
            .join("recovery");
    }
    std::env::temp_dir().join("rpdf").join("recovery")
}

fn current_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{
        PersistenceService, TextQuality, current_unix_ms, native_text_quality,
        normalize_extracted_text,
    };
    use crate::model::{
        AnnotationAppearanceSet, AnnotationVisibility, AutosaveState, BackgroundPattern,
        BackgroundPatternStyle, CanvasDocument, DocumentMetadata, HighlightMode,
        PdfDocumentSession, PdfSource, PdfViewState, PdfViewportState, PlaybackState, Point,
        ReadingReliability, ReadingSupportState, RecolorExportMode, RecolorState, RgbaColor,
        SelectionTarget, TextSupportSource, TtsState, ViewportState, WarningCode,
    };

    #[test]
    fn normalizes_whitespace_for_quality_checks() {
        assert_eq!(
            normalize_extracted_text("alpha\n beta\tgamma"),
            "alpha beta gamma"
        );
    }

    #[test]
    fn rejects_very_short_or_sparse_text() {
        assert_eq!(native_text_quality("figure 1"), TextQuality::Unusable);
    }

    #[test]
    fn marks_reasonable_running_text_as_good() {
        let text = "This PDF contains readable study notes with sentences that have ordinary word lengths and enough alphabetic content to support follow along reading.";
        assert_eq!(native_text_quality(text), TextQuality::Good);
    }

    #[test]
    fn marks_garbled_long_token_text_as_weak() {
        let text = "AABBCCDDEEFFGGHHIIJJKKLLMMNNOOPP syntheticcontentwithoutspaces repeatedtoken repeatedtoken repeatedtoken repeatedtoken repeatedtoken repeatedtoken repeatedtoken repeatedtoken repeatedtoken repeatedtoken";
        assert_eq!(native_text_quality(text), TextQuality::Weak);
    }

    #[test]
    fn round_trips_canvas_save_files() {
        let service = PersistenceService;
        let path = unique_test_path("canvas-save.json");
        let document = sample_canvas_document();

        service
            .save_canvas_document(&path, &document)
            .expect("save canvas document");
        let loaded = service
            .load_canvas_document(&path)
            .expect("load canvas document");

        assert_eq!(loaded.metadata.document_id, document.metadata.document_id);
        assert_eq!(loaded.background, document.background);
        assert!(!loaded.autosave.dirty);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn round_trips_pdf_save_files() {
        let service = PersistenceService;
        let path = unique_test_path("pdf-save.json");
        let session = sample_pdf_session();

        service
            .save_pdf_session(&path, &session)
            .expect("save pdf session");
        let loaded = service.load_pdf_session(&path).expect("load pdf session");

        assert_eq!(loaded.metadata.document_id, session.metadata.document_id);
        assert_eq!(loaded.source, session.source);
        assert!(!loaded.autosave.dirty);

        let _ = std::fs::remove_file(path);
    }

    fn unique_test_path(name: &str) -> String {
        std::env::temp_dir()
            .join(format!("rpdf-test-{}-{}", current_unix_ms(), name))
            .display()
            .to_string()
    }

    fn sample_canvas_document() -> CanvasDocument {
        CanvasDocument {
            metadata: DocumentMetadata {
                document_id: "canvas-test".to_string(),
                title: Some("Canvas".to_string()),
                created_unix_ms: 1,
                modified_unix_ms: 2,
            },
            viewport: ViewportState {
                origin: Point { x: 10.0, y: 20.0 },
                zoom: 1.2,
            },
            background: BackgroundPattern::Dots(BackgroundPatternStyle {
                spacing: 24.0,
                line_width: 1.0,
                color: RgbaColor {
                    red: 1,
                    green: 2,
                    blue: 3,
                    alpha: 255,
                },
            }),
            items: Vec::new(),
            selection: SelectionTarget::WholeCanvas,
            autosave: AutosaveState {
                recovery_snapshot_id: Some("snapshot".to_string()),
                dirty: true,
            },
        }
    }

    fn sample_pdf_session() -> PdfDocumentSession {
        PdfDocumentSession {
            metadata: DocumentMetadata {
                document_id: "pdf-test".to_string(),
                title: Some("PDF".to_string()),
                created_unix_ms: 1,
                modified_unix_ms: 2,
            },
            source: PdfSource::FilePath("/tmp/sample.pdf".into()),
            viewport: PdfViewportState {
                page_index: 2,
                scroll_offset: Point { x: 4.0, y: 8.0 },
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
                    message: "missing text".to_string(),
                }),
            },
            view: PdfViewState {
                recolor: RecolorState {
                    current_profile: None,
                    export_mode: RecolorExportMode::PreserveOriginalAppearance,
                },
                annotation_visibility: AnnotationVisibility {
                    normal_view: sample_palette(),
                    recolored_view: sample_palette(),
                },
            },
            autosave: AutosaveState {
                recovery_snapshot_id: Some("snapshot".to_string()),
                dirty: true,
            },
        }
    }

    fn sample_palette() -> crate::model::AnnotationPalette {
        crate::model::AnnotationPalette {
            ink_color: sample_appearance().normal_view,
            highlighter_color: sample_appearance().normal_view,
            text_color: sample_appearance().normal_view,
        }
    }

    fn sample_appearance() -> AnnotationAppearanceSet {
        AnnotationAppearanceSet {
            normal_view: RgbaColor {
                red: 10,
                green: 20,
                blue: 30,
                alpha: 255,
            },
            recolored_view: RgbaColor {
                red: 40,
                green: 50,
                blue: 60,
                alpha: 255,
            },
        }
    }
}
