use crate::contracts::dto::{
    AppBootstrapDto,
    AppModeDto,
    ExtractPdfTextRequestDto,
    LoadCanvasProjectRequestDto,
    LoadPdfStudySessionRequestDto,
    OpenPdfDocumentRequestDto,
    OpenPdfDocumentResponseDto,
    PageTextExtractionDto,
    PdfBackendStatusDto,
    ReadingReliabilityStateDto,
    SaveCanvasProjectRequestDto,
    SavePdfStudySessionRequestDto,
    SaveSvgExportRequestDto,
    SpeakTextRequestDto,
    RenderPdfPageRequestDto,
    RenderPdfPageResponseDto,
};
use crate::domain::canvas::CanvasDocument;
use crate::domain::pdf::PdfStudyDocument;
use crate::infrastructure::local_tts;
use crate::infrastructure::pdf_engine::{PdfEngineAdapter, PdfiumEngineAdapter};

pub struct AppServices {
    pdf_engine: PdfiumEngineAdapter,
}

impl Default for AppServices {
    fn default() -> Self {
        Self {
            pdf_engine: PdfiumEngineAdapter::new(),
        }
    }
}

impl AppServices {
    pub fn bootstrap(&self) -> AppBootstrapDto {
        AppBootstrapDto {
            supported_modes: vec![AppModeDto::Canvas, AppModeDto::Pdf],
            active_pdf_backend: self.pdf_engine.backend_status(),
            reliability_states: vec![
                ReadingReliabilityStateDto::NativeReliable,
                ReadingReliabilityStateDto::NativeWeak,
                ReadingReliabilityStateDto::OcrReliable,
                ReadingReliabilityStateDto::OcrWeak,
                ReadingReliabilityStateDto::Unavailable,
            ],
        }
    }

    pub fn pdf_backend_status(&self) -> PdfBackendStatusDto {
        self.pdf_engine.backend_status()
    }

    pub fn open_pdf_document(
        &self,
        request: &OpenPdfDocumentRequestDto,
    ) -> Result<OpenPdfDocumentResponseDto, String> {
        let normalized_path = request.document_path.trim();

        if normalized_path.is_empty() {
            return Err("Document path is required.".to_string());
        }

        let pdf_path = std::path::Path::new(normalized_path);

        if !pdf_path.exists() {
            return Err(format!("PDF file was not found: {normalized_path}"));
        }

        let extension = pdf_path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase());

        if extension.as_deref() != Some("pdf") {
            return Err("Only .pdf files can be opened in PDF Mode.".to_string());
        }

        let backend_status = self.pdf_engine.backend_status();
        let document_name = pdf_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(normalized_path)
            .to_string();
        let page_count = pdf_page_count(normalized_path).ok();

        Ok(OpenPdfDocumentResponseDto {
            document_path: normalized_path.to_string(),
            document_name,
            page_count,
            backend_ready: backend_status.configured,
            notes: backend_status.notes,
        })
    }

    pub fn save_canvas_project(
        &self,
        request: &SaveCanvasProjectRequestDto,
    ) -> Result<(), String> {
        write_json_document(&request.file_path, &request.document)
    }

    pub fn load_canvas_project(
        &self,
        request: &LoadCanvasProjectRequestDto,
    ) -> Result<CanvasDocument, String> {
        read_json_document(&request.file_path)
    }

    pub fn save_pdf_study_session(
        &self,
        request: &SavePdfStudySessionRequestDto,
    ) -> Result<(), String> {
        write_json_document(&request.file_path, &request.document)
    }

    pub fn load_pdf_study_session(
        &self,
        request: &LoadPdfStudySessionRequestDto,
    ) -> Result<PdfStudyDocument, String> {
        read_json_document(&request.file_path)
    }

    pub fn save_svg_export(
        &self,
        request: &SaveSvgExportRequestDto,
    ) -> Result<Option<String>, String> {
        let svg_markup = request.svg_markup.trim();

        if svg_markup.is_empty() {
            return Err("SVG export markup was empty.".to_string());
        }

        let suggested_file_name = default_svg_file_name(&request.suggested_file_name);
        let selected_path = rfd::FileDialog::new()
            .add_filter("SVG image", &["svg"])
            .set_file_name(&suggested_file_name)
            .save_file();

        let Some(mut export_path) = selected_path else {
            return Ok(None);
        };

        if export_path.extension().is_none() {
            export_path.set_extension("svg");
        }

        if let Some(parent) = export_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| format!("Could not create export directory: {error}"))?;
            }
        }

        std::fs::write(&export_path, svg_markup.as_bytes())
            .map_err(|error| format!("Could not write SVG export: {error}"))?;

        Ok(Some(export_path.display().to_string()))
    }

    pub fn render_pdf_page(
        &self,
        request: &RenderPdfPageRequestDto,
    ) -> Result<RenderPdfPageResponseDto, String> {
        self.pdf_engine.render_page(request)
    }

    pub fn extract_pdf_page_text(
        &self,
        request: &ExtractPdfTextRequestDto,
    ) -> Result<PageTextExtractionDto, String> {
        self.pdf_engine.extract_page_text(request)
    }

    pub fn extract_pdf_page_ocr(
        &self,
        request: &ExtractPdfTextRequestDto,
    ) -> Result<PageTextExtractionDto, String> {
        self.pdf_engine.extract_page_text_with_ocr(request)
    }

    pub fn speak_text_locally(
        &self,
        request: &SpeakTextRequestDto,
    ) -> Result<(), String> {
        local_tts::speak_text(request)
    }

    pub fn stop_local_speech(&self) -> Result<(), String> {
        local_tts::stop_speaking()
    }
}

fn pdf_page_count(document_path: &str) -> Result<u32, String> {
    let output = std::process::Command::new("pdfinfo")
        .arg(document_path)
        .output()
        .map_err(|error| format!("Could not run pdfinfo: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pdfinfo failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let trimmed = line.trim();

        if let Some(value) = trimmed.strip_prefix("Pages:") {
            return value
                .trim()
                .parse::<u32>()
                .map_err(|error| format!("Could not parse page count from pdfinfo: {error}"));
        }
    }

    Err("pdfinfo did not report a page count.".to_string())
}

fn default_svg_file_name(suggested_file_name: &str) -> String {
    let trimmed = suggested_file_name.trim();

    if trimmed.is_empty() {
        return "canvas-export.svg".to_string();
    }

    if trimmed.to_ascii_lowercase().ends_with(".svg") {
        return trimmed.to_string();
    }

    format!("{trimmed}.svg")
}

fn write_json_document<T>(file_path: &str, document: &T) -> Result<(), String>
where
    T: serde::Serialize,
{
    let normalized_path = file_path.trim();

    if normalized_path.is_empty() {
        return Err("A file path is required.".to_string());
    }

    let path = std::path::Path::new(normalized_path);

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("Could not create parent directories: {error}"))?;
        }
    }

    let serialized = serde_json::to_string_pretty(document)
        .map_err(|error| format!("Could not serialize document: {error}"))?;

    std::fs::write(path, serialized)
        .map_err(|error| format!("Could not write document file: {error}"))?;

    Ok(())
}

fn read_json_document<T>(file_path: &str) -> Result<T, String>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let normalized_path = file_path.trim();

    if normalized_path.is_empty() {
        return Err("A file path is required.".to_string());
    }

    let path = std::path::Path::new(normalized_path);
    let contents = std::fs::read_to_string(path)
        .map_err(|error| format!("Could not read document file: {error}"))?;

    serde_json::from_str(&contents)
        .map_err(|error| format!("Could not deserialize document file: {error}"))
}
