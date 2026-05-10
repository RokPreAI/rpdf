mod app;
mod contracts;
mod domain;
mod infrastructure;

use app::commands::{
    extract_pdf_page_ocr,
    extract_pdf_page_text,
    get_app_bootstrap,
    load_canvas_project,
    load_pdf_study_session,
    open_pdf_document,
    save_canvas_project,
    save_pdf_study_session,
    get_pdf_backend_status,
    render_pdf_page,
};
use app::services::AppServices;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppServices::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_app_bootstrap,
            get_pdf_backend_status,
            open_pdf_document,
            save_canvas_project,
            load_canvas_project,
            save_pdf_study_session,
            load_pdf_study_session,
            render_pdf_page,
            extract_pdf_page_text,
            extract_pdf_page_ocr
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
