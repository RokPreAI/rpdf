use serde::{Deserialize, Serialize};

use crate::domain::canvas::DocumentVersion;
use crate::domain::reading::ReadingReliabilityState;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfStudyDocument {
    pub version: DocumentVersion,
    pub id: String,
    pub source_pdf_path: String,
    pub page_count: Option<u32>,
    pub current_page_index: u32,
    pub annotations: Vec<PdfPageAnnotationLayer>,
    pub recolor: PdfRecolorSettings,
    pub reading_cache: Vec<PdfPageReadingCache>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPageAnnotationLayer {
    pub page_index: u32,
    pub strokes: Vec<PdfStrokeAnnotation>,
    pub notes: Vec<PdfTextNote>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PdfStrokeAnnotation {
    pub color: String,
    pub width: f32,
    pub points: Vec<PdfPoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PdfTextNote {
    pub text: String,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PdfPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PdfRecolorSettings {
    pub enabled: bool,
    pub foreground: String,
    pub background: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPageReadingCache {
    pub page_index: u32,
    pub reliability: ReadingReliabilityState,
    pub source_kind: ReadingSourceKind,
    pub cache_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadingSourceKind {
    Native,
    Ocr,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdf_study_document_deserializes_camel_case_payload() {
        let raw = r##"{
          "version": { "major": 1, "minor": 0 },
          "id": "pdf-1",
          "sourcePdfPath": "/tmp/example.pdf",
          "pageCount": 42,
          "currentPageIndex": 3,
          "annotations": [
            {
              "pageIndex": 3,
              "strokes": [
                {
                  "color": "#ffffff",
                  "width": 2.0,
                  "points": [
                    { "x": 1.0, "y": 2.0 }
                  ]
                }
              ],
              "notes": []
            }
          ],
          "recolor": {
            "enabled": true,
            "foreground": "#111111",
            "background": "#eeeeee"
          },
          "readingCache": [
            {
              "pageIndex": 3,
              "reliability": "native_reliable",
              "sourceKind": "native",
              "cacheKey": null
            }
          ]
        }"##;

        let parsed: PdfStudyDocument = serde_json::from_str(raw).expect("pdf study document should deserialize");

        assert_eq!(parsed.source_pdf_path, "/tmp/example.pdf");
        assert_eq!(parsed.page_count, Some(42));
        assert_eq!(parsed.current_page_index, 3);
        assert_eq!(parsed.reading_cache[0].source_kind, ReadingSourceKind::Native);
    }
}
