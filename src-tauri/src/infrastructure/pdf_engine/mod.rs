use crate::contracts::dto::{
    ExtractPdfTextRequestDto,
    PageTextExtractionDto,
    PdfBackendStatusDto,
    ReadingReliabilityStateDto,
    RenderPdfPageRequestDto,
    RenderPdfPageResponseDto,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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
    fn extract_page_text_with_ocr(
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
        let poppler_ready = command_available("pdftotext") && command_available("pdfinfo");
        let ocr_ready = command_available("pdftoppm") && command_available("tesseract");

        PdfBackendStatusDto {
            backend_key: "pdfium".to_string(),
            backend_name: "Pdfium adapter boundary".to_string(),
            configured: false,
            notes: vec![
                "The repo now assumes a Pdfium-style Rust integration path.".to_string(),
                "Page rendering currently uses local pdftoppm until Pdfium rendering is implemented.".to_string(),
                if poppler_ready {
                    "Native text extraction currently uses local Poppler tools until Pdfium extraction is implemented.".to_string()
                } else {
                    "Native text extraction tools are not fully available on this machine.".to_string()
                },
                if ocr_ready {
                    "OCR fallback can use local pdftoppm + tesseract when native text is weak or unavailable.".to_string()
                } else {
                    "OCR fallback tools are not fully available on this machine.".to_string()
                },
                "This boundary exists so later work does not need to reopen the engine decision."
                    .to_string(),
            ],
        }
    }

    fn render_page(
        &self,
        request: &RenderPdfPageRequestDto,
    ) -> Result<RenderPdfPageResponseDto, String> {
        render_pdf_page_png(request)
    }

    fn extract_page_text(
        &self,
        request: &ExtractPdfTextRequestDto,
    ) -> Result<PageTextExtractionDto, String> {
        let text = run_pdftotext(request)?;
        Ok(build_extraction_response(
            request.page_index,
            "native",
            text,
            "OCR fallback is available if native PDF text is weak or unavailable.",
        ))
    }

    fn extract_page_text_with_ocr(
        &self,
        request: &ExtractPdfTextRequestDto,
    ) -> Result<PageTextExtractionDto, String> {
        let text = run_tesseract_ocr(request)?;
        Ok(build_extraction_response(
            request.page_index,
            "ocr",
            text,
            "OCR fallback text can be weaker than native PDF text. Follow-along confidence may be limited.",
        ))
    }
}

fn command_available(command: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {command} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn run_pdftotext(request: &ExtractPdfTextRequestDto) -> Result<String, String> {
    let page_number = request.page_index + 1;
    let output = Command::new("pdftotext")
        .args([
            "-f",
            &page_number.to_string(),
            "-l",
            &page_number.to_string(),
            "-layout",
            &request.document_path,
            "-",
        ])
        .output()
        .map_err(|error| format!("Could not run pdftotext: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pdftotext failed: {}", stderr.trim()));
    }

    String::from_utf8(output.stdout)
        .map_err(|error| format!("pdftotext produced invalid UTF-8 output: {error}"))
}

fn run_tesseract_ocr(request: &ExtractPdfTextRequestDto) -> Result<String, String> {
    let page_number = request.page_index + 1;
    let prefix = temporary_ocr_prefix(page_number);
    let image_path = prefix.with_extension("png");

    let render_output = Command::new("pdftoppm")
        .args([
            "-f",
            &page_number.to_string(),
            "-l",
            &page_number.to_string(),
            "-singlefile",
            "-r",
            "180",
            "-png",
            &request.document_path,
            &prefix.to_string_lossy(),
        ])
        .output()
        .map_err(|error| format!("Could not run pdftoppm for OCR fallback: {error}"))?;

    if !render_output.status.success() {
        let stderr = String::from_utf8_lossy(&render_output.stderr);
        return Err(format!("pdftoppm failed: {}", stderr.trim()));
    }

    let tesseract_output = Command::new("tesseract")
        .args([
            &image_path.to_string_lossy(),
            "stdout",
            "--psm",
            "6",
        ])
        .output()
        .map_err(|error| format!("Could not run tesseract: {error}"))?;

    let _ = fs::remove_file(&image_path);

    if !tesseract_output.status.success() {
        let stderr = String::from_utf8_lossy(&tesseract_output.stderr);
        return Err(format!("tesseract failed: {}", stderr.trim()));
    }

    String::from_utf8(tesseract_output.stdout)
        .map_err(|error| format!("tesseract produced invalid UTF-8 output: {error}"))
}

fn render_pdf_page_png(
    request: &RenderPdfPageRequestDto,
) -> Result<RenderPdfPageResponseDto, String> {
    let page_number = request.page_index + 1;
    let prefix = temporary_ocr_prefix(page_number);
    let image_path = prefix.with_extension("png");
    let render_output = Command::new("pdftoppm")
        .args([
            "-f",
            &page_number.to_string(),
            "-l",
            &page_number.to_string(),
            "-singlefile",
            "-r",
            "180",
            "-png",
            &request.document_path,
            &prefix.to_string_lossy(),
        ])
        .output()
        .map_err(|error| format!("Could not run pdftoppm for PDF page rendering: {error}"))?;

    if !render_output.status.success() {
        let stderr = String::from_utf8_lossy(&render_output.stderr);
        return Err(format!("pdftoppm failed: {}", stderr.trim()));
    }

    let image_bytes = fs::read(&image_path)
        .map_err(|error| format!("Could not read rendered PDF page image: {error}"))?;
    let _ = fs::remove_file(&image_path);
    let (width, height) = png_dimensions(&image_bytes)?;

    Ok(RenderPdfPageResponseDto {
        page_index: request.page_index,
        mime_type: "image/png".to_string(),
        data_base64: encode_base64(&image_bytes),
        width,
        height,
    })
}

fn temporary_ocr_prefix(page_number: u32) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

    std::env::temp_dir().join(format!("rpdf2-ocr-page-{page_number}-{timestamp}"))
}

fn png_dimensions(image_bytes: &[u8]) -> Result<(u32, u32), String> {
    if image_bytes.len() < 24 || &image_bytes[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("Rendered page did not produce a valid PNG header.".to_string());
    }

    let width = u32::from_be_bytes([image_bytes[16], image_bytes[17], image_bytes[18], image_bytes[19]]);
    let height = u32::from_be_bytes([image_bytes[20], image_bytes[21], image_bytes[22], image_bytes[23]]);

    Ok((width, height))
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut index = 0;

    while index < bytes.len() {
        let first = bytes[index];
        let second = if index + 1 < bytes.len() { bytes[index + 1] } else { 0 };
        let third = if index + 2 < bytes.len() { bytes[index + 2] } else { 0 };
        let triple = ((first as u32) << 16) | ((second as u32) << 8) | third as u32;

        encoded.push(TABLE[((triple >> 18) & 0x3f) as usize] as char);
        encoded.push(TABLE[((triple >> 12) & 0x3f) as usize] as char);

        if index + 1 < bytes.len() {
          encoded.push(TABLE[((triple >> 6) & 0x3f) as usize] as char);
        } else {
          encoded.push('=');
        }

        if index + 2 < bytes.len() {
          encoded.push(TABLE[(triple & 0x3f) as usize] as char);
        } else {
          encoded.push('=');
        }

        index += 3;
    }

    encoded
}

fn build_extraction_response(
    page_index: u32,
    source_kind: &str,
    text: String,
    non_empty_warning: &str,
) -> PageTextExtractionDto {
    let normalized_text = text.replace('\u{000C}', "").trim().to_string();
    let text_length = normalized_text.chars().filter(|character| !character.is_whitespace()).count();
    let spans = text_to_spans(&normalized_text);

    let (reliability, warning) = if text_length >= 120 {
        (
            if source_kind == "ocr" {
                ReadingReliabilityStateDto::OcrReliable
            } else {
                ReadingReliabilityStateDto::NativeReliable
            },
            Some(non_empty_warning.to_string()),
        )
    } else if text_length >= 24 {
        (
            if source_kind == "ocr" {
                ReadingReliabilityStateDto::OcrWeak
            } else {
                ReadingReliabilityStateDto::NativeWeak
            },
            Some("Extracted text is short or sparse. Follow-along alignment may be unreliable.".to_string()),
        )
    } else {
        (
            ReadingReliabilityStateDto::Unavailable,
            Some(if source_kind == "ocr" {
                "OCR fallback did not recover enough text for reliable reading.".to_string()
            } else {
                "Native PDF text was unavailable or too weak for reliable reading. Try OCR fallback.".to_string()
            }),
        )
    };

    PageTextExtractionDto {
        page_index,
        source_kind: source_kind.to_string(),
        reliability,
        warning,
        spans,
    }
}

fn text_to_spans(text: &str) -> Vec<crate::contracts::dto::TextSpanDto> {
    if text.is_empty() {
        return Vec::new();
    }

    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                return None;
            }

            Some(crate::contracts::dto::TextSpanDto {
                text: trimmed.to_string(),
                x: 0.0,
                y: (index as f32) * 18.0,
                width: trimmed.len() as f32 * 8.0,
                height: 16.0,
            })
        })
        .collect()
}
