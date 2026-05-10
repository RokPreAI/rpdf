use crate::contracts::dto::{
    ExtractPdfTextRequestDto,
    PageTextExtractionDto,
    PdfBackendStatusDto,
    ReadingReliabilityStateDto,
    RenderPdfPageRequestDto,
    RenderPdfPageResponseDto,
};

pub trait PdfEngineAdapter {
    fn backend_status(&self) -> PdfBackendStatusDto;
    fn render_page(
        &self,
        request: &RenderPdfPageRequestDto,
    ) -> Result<RenderPdfPageResponseDto, String>;
    fn extract_page_text(
        &self,
        request: &ExtractPdfTextRequestDto,
    ) -> Result<PageTextExtractionDto, String>;
}

#[derive(Default)]
pub struct PdfiumEngineAdapter;

impl PdfiumEngineAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl PdfEngineAdapter for PdfiumEngineAdapter {
    fn backend_status(&self) -> PdfBackendStatusDto {
        PdfBackendStatusDto {
            backend_key: "pdfium".to_string(),
            backend_name: "Pdfium adapter boundary".to_string(),
            configured: false,
            notes: vec![
                "The repo now assumes a Pdfium-style Rust integration path.".to_string(),
                "Runtime rendering and extraction are still intentionally unimplemented.".to_string(),
                "This boundary exists so later work does not need to reopen the engine decision."
                    .to_string(),
            ],
        }
    }

    fn render_page(
        &self,
        request: &RenderPdfPageRequestDto,
    ) -> Result<RenderPdfPageResponseDto, String> {
        Err(format!(
            "Pdfium page rendering is not implemented yet for {} page {}.",
            request.document_path, request.page_index
        ))
    }

    fn extract_page_text(
        &self,
        request: &ExtractPdfTextRequestDto,
    ) -> Result<PageTextExtractionDto, String> {
        Ok(PageTextExtractionDto {
            page_index: request.page_index,
            reliability: ReadingReliabilityStateDto::Unavailable,
            warning: Some(
                "Text extraction contract is wired, but Pdfium extraction is not implemented yet."
                    .to_string(),
            ),
            spans: Vec::new(),
        })
    }
}
