use tauri::State;

use crate::app::services::AppServices;
use crate::contracts::dto::{
    AppBootstrapDto,
    ExtractPdfTextRequestDto,
    LoadCanvasProjectRequestDto,
    LoadPdfStudySessionRequestDto,
    OpenPdfDocumentRequestDto,
    OpenPdfDocumentResponseDto,
    PageTextExtractionDto,
    PdfBackendStatusDto,
    SaveCanvasProjectRequestDto,
    SavePdfStudySessionRequestDto,
    RenderPdfPageRequestDto,
    RenderPdfPageResponseDto,
};
use crate::domain::canvas::CanvasDocument;
use crate::domain::pdf::PdfStudyDocument;

#[tauri::command]
pub fn get_app_bootstrap(services: State<'_, AppServices>) -> AppBootstrapDto {
    services.bootstrap()
}

#[tauri::command]
pub fn get_pdf_backend_status(services: State<'_, AppServices>) -> PdfBackendStatusDto {
    services.pdf_backend_status()
}

#[tauri::command]
pub fn open_pdf_document(
    request: OpenPdfDocumentRequestDto,
    services: State<'_, AppServices>,
) -> Result<OpenPdfDocumentResponseDto, String> {
    services.open_pdf_document(&request)
}

#[tauri::command]
pub fn save_canvas_project(
    request: SaveCanvasProjectRequestDto,
    services: State<'_, AppServices>,
) -> Result<(), String> {
    services.save_canvas_project(&request)
}

#[tauri::command]
pub fn load_canvas_project(
    request: LoadCanvasProjectRequestDto,
    services: State<'_, AppServices>,
) -> Result<CanvasDocument, String> {
    services.load_canvas_project(&request)
}

#[tauri::command]
pub fn save_pdf_study_session(
    request: SavePdfStudySessionRequestDto,
    services: State<'_, AppServices>,
) -> Result<(), String> {
    services.save_pdf_study_session(&request)
}

#[tauri::command]
pub fn load_pdf_study_session(
    request: LoadPdfStudySessionRequestDto,
    services: State<'_, AppServices>,
) -> Result<PdfStudyDocument, String> {
    services.load_pdf_study_session(&request)
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
