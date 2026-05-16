use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentVersion {
    pub major: u16,
    pub minor: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasDocument {
    pub version: DocumentVersion,
    pub id: String,
    pub background_pattern: BackgroundPattern,
    pub strokes: Vec<CanvasStroke>,
    #[serde(default)]
    pub shapes: Vec<CanvasShape>,
    #[serde(default)]
    pub texts: Vec<CanvasText>,
    pub images: Vec<CanvasImagePlacement>,
    #[serde(default)]
    pub pdf_pages: Vec<CanvasPdfPagePlacement>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundPattern {
    Dots,
    Lines,
    Squares,
    None,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasStroke {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub color: String,
    pub width: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u32>,
    pub points: Vec<CanvasPoint>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanvasShapeKind {
    Line,
    Arrow,
    Rectangle,
    Ellipse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasShapePoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasShape {
    pub id: String,
    pub kind: CanvasShapeKind,
    pub color: String,
    pub width: f32,
    #[serde(default)]
    pub order: Option<u32>,
    pub start: CanvasShapePoint,
    pub end: CanvasShapePoint,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasText {
    pub id: String,
    pub text: String,
    pub color: String,
    pub font_size: f32,
    #[serde(default)]
    pub order: Option<u32>,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasPoint {
    pub x: f32,
    pub y: f32,
    #[serde(
        default = "default_pressure_value",
        skip_serializing_if = "is_default_pressure_value"
    )]
    pub pressure: f32,
}

fn default_pressure_value() -> f32 {
    1.0
}

fn is_default_pressure_value(value: &f32) -> bool {
    (*value - 1.0).abs() < f32::EPSILON
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasImagePlacement {
    pub id: String,
    pub asset_path: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasPdfPagePlacement {
    pub id: String,
    pub source_pdf_path: String,
    pub page_index: u32,
    pub asset_path: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub recolor: CanvasPdfPageRecolor,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasPdfPageRecolor {
    pub enabled: bool,
    pub foreground: String,
    pub background: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_document_deserializes_camel_case_payload() {
        let raw = r##"{
          "version": { "major": 1, "minor": 0 },
          "id": "canvas-1",
          "backgroundPattern": "dots",
          "strokes": [
            {
              "id": "stroke-1",
              "color": "#ffffff",
              "width": 4.5,
              "order": 1,
              "points": [
                { "x": 10.0, "y": 20.0, "pressure": 0.8 }
              ]
            }
          ],
          "shapes": [],
          "texts": [
            {
              "id": "text-1",
              "text": "hello",
              "color": "#ffffff",
              "fontSize": 16.0,
              "order": 2,
              "x": 12.0,
              "y": 24.0
            }
          ],
          "images": [
            {
              "id": "image-1",
              "assetPath": "/tmp/example.png",
              "x": 0.0,
              "y": 0.0,
              "width": 100.0,
              "height": 50.0
            }
          ],
          "pdfPages": [
            {
              "id": "pdf-1",
              "sourcePdfPath": "/tmp/example.pdf",
              "pageIndex": 0,
              "assetPath": "/tmp/example-page.png",
              "x": 5.0,
              "y": 6.0,
              "width": 300.0,
              "height": 200.0,
              "recolor": {
                "enabled": true,
                "foreground": "#111111",
                "background": "#eeeeee"
              }
            }
          ]
        }"##;

        let parsed: CanvasDocument = serde_json::from_str(raw).expect("canvas document should deserialize");

        assert_eq!(parsed.background_pattern, BackgroundPattern::Dots);
        assert_eq!(parsed.texts[0].font_size, 16.0);
        assert_eq!(parsed.images[0].asset_path, "/tmp/example.png");
        assert_eq!(parsed.pdf_pages[0].source_pdf_path, "/tmp/example.pdf");
    }

    #[test]
    fn canvas_document_serialization_omits_default_pressure_and_empty_stroke_metadata() {
        let document = CanvasDocument {
            version: DocumentVersion { major: 1, minor: 0 },
            id: "canvas-1".to_string(),
            background_pattern: BackgroundPattern::Dots,
            strokes: vec![CanvasStroke {
                id: None,
                color: "#ffffff".to_string(),
                width: 3.0,
                order: None,
                points: vec![
                    CanvasPoint {
                        x: 10.0,
                        y: 20.0,
                        pressure: 1.0,
                    },
                    CanvasPoint {
                        x: 12.0,
                        y: 24.0,
                        pressure: 0.5,
                    },
                ],
            }],
            shapes: vec![],
            texts: vec![],
            images: vec![],
            pdf_pages: vec![],
        };

        let serialized =
            serde_json::to_string(&document).expect("canvas document should serialize compactly");

        assert!(!serialized.contains("\"id\":null"));
        assert!(!serialized.contains("\"order\":null"));
        assert!(!serialized.contains("\"pressure\":1.0"));
        assert!(serialized.contains("\"pressure\":0.5"));
    }
}
