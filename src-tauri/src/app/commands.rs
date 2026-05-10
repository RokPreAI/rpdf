use tauri::State;

use crate::app::services::AppServices;
use crate::contracts::dto::{
    AppBootstrapDto,
    ExtractPdfTextRequestDto,
    PageTextExtractionDto,
    PdfBackendStatusDto,
    RenderPdfPageRequestDto,
    RenderPdfPageResponseDto,
};

#[tauri::command]
pub fn get_app_bootstrap(services: State<'_, AppServices>) -> AppBootstrapDto {
    services.bootstrap()
}

#[tauri::command]
pub fn get_pdf_backend_status(services: State<'_, AppServices>) -> PdfBackendStatusDto {
    services.pdf_backend_status()
}

#[tauri::command]
pub fn render_pdf_page(
    request: RenderPdfPageRequestDto,
    services: State<'_, AppServices>,
) -> Result<RenderPdfPageResponseDto, String> {
    services.render_pdf_page(&request)
}

#[tauri::command]
pub fn extract_pdf_page_text(
    request: ExtractPdfTextRequestDto,
    services: State<'_, AppServices>,
) -> Result<PageTextExtractionDto, String> {
    services.extract_pdf_page_text(&request)
}
