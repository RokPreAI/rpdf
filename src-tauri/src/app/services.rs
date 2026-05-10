use crate::contracts::dto::{
    AppBootstrapDto,
    AppModeDto,
    ExtractPdfTextRequestDto,
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
