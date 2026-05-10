mod app;
mod contracts;
mod domain;
mod infrastructure;

use app::commands::{
    extract_pdf_page_text,
    get_app_bootstrap,
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
            render_pdf_page,
            extract_pdf_page_text
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
