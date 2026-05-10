use crate::contracts::dto::{
    AppBootstrapDto,
    AppModeDto,
    ExtractPdfTextRequestDto,
    OpenPdfDocumentRequestDto,
    OpenPdfDocumentResponseDto,
    PageTextExtractionDto,
    PdfBackendStatusDto,
    ReadingReliabilityStateDto,
    RenderPdfPageRequestDto,
    RenderPdfPageResponseDto,
};
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

        Ok(OpenPdfDocumentResponseDto {
            document_path: normalized_path.to_string(),
            document_name,
            page_count: None,
            backend_ready: backend_status.configured,
            notes: backend_status.notes,
        })
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
}
