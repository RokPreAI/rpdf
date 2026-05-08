use super::util;
use crate::model::{
    CanvasItem, HighlightMode, IncompatibleExportReason, ReadingReliability, SvgCompatibility,
    TextSupportSource, UserVisibleWarning, WarningCode,
};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Default)]
pub struct AppServices {
    pub reading_support: ReadingSupportService,
    pub canvas_export: CanvasExportService,
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

#[cfg(test)]
mod tests {
    use super::{TextQuality, native_text_quality, normalize_extracted_text};

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
}
