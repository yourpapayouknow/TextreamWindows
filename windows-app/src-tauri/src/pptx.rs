use roxmltree::{Document, Node};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug)]
pub enum PptxError {
    UnsupportedFormat,
    ExtractionFailed(String),
    NoNotesFound,
}

impl std::fmt::Display for PptxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedFormat => write!(f, "Unsupported file format. Please drop a .pptx file."),
            Self::ExtractionFailed(detail) => write!(f, "Failed to extract notes: {detail}"),
            Self::NoNotesFound => write!(f, "No presenter notes found in this presentation."),
        }
    }
}

impl std::error::Error for PptxError {}

pub fn extract_notes(path: &Path) -> Result<Vec<String>, PptxError> {
    if path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.eq_ignore_ascii_case("pptx")) != Some(true) {
        return Err(PptxError::UnsupportedFormat);
    }

    let file = File::open(path).map_err(|err| PptxError::ExtractionFailed(err.to_string()))?;
    let mut archive =
        ZipArchive::new(file).map_err(|err| PptxError::ExtractionFailed(err.to_string()))?;

    let mut note_files = Vec::new();
    for index in 0..archive.len() {
        let file = archive
            .by_index(index)
            .map_err(|err| PptxError::ExtractionFailed(err.to_string()))?;
        let name = file.name().to_string();
        if name.starts_with("ppt/notesSlides/notesSlide") && name.ends_with(".xml") {
            note_files.push(name);
        }
    }

    note_files.sort_by_key(|name| extract_number(name).unwrap_or(0));
    if note_files.is_empty() {
        return Err(PptxError::NoNotesFound);
    }

    let mut pages = Vec::new();
    for name in note_files {
        let mut entry = archive
            .by_name(&name)
            .map_err(|err| PptxError::ExtractionFailed(err.to_string()))?;
        let mut xml = String::new();
        entry
            .read_to_string(&mut xml)
            .map_err(|err| PptxError::ExtractionFailed(err.to_string()))?;
        let text = parse_note_xml(&xml);
        let trimmed = text.trim();
        if !trimmed.is_empty() && trimmed.parse::<i64>().is_err() {
            pages.push(trimmed.to_string());
        }
    }

    if pages.is_empty() {
        Err(PptxError::NoNotesFound)
    } else {
        Ok(pages)
    }
}

fn extract_number(name: &str) -> Option<u32> {
    let digits: String = name.chars().filter(|ch| ch.is_ascii_digit()).collect();
    digits.parse().ok()
}

pub(crate) fn parse_note_xml(xml: &str) -> String {
    let Ok(doc) = Document::parse(xml) else {
        return String::new();
    };

    let mut paragraphs = Vec::new();
    for paragraph in doc.descendants().filter(|node| node.is_element() && node.tag_name().name() == "p") {
        if should_skip_shape(paragraph) {
            continue;
        }
        let mut current = String::new();
        for node in paragraph.descendants() {
            if node.is_element() && node.tag_name().name() == "t" {
                if let Some(text) = node.text() {
                    current.push_str(text);
                }
            }
            if node.is_element() && node.tag_name().name() == "br" {
                current.push('\n');
            }
        }
        paragraphs.push(current);
    }

    paragraphs.join("\n")
}

fn should_skip_shape(node: Node<'_, '_>) -> bool {
    node.ancestors()
        .find(|ancestor| ancestor.is_element() && ancestor.tag_name().name() == "sp")
        .map(|shape| {
            shape.descendants().any(|child| {
                child.is_element()
                    && child.tag_name().name() == "ph"
                    && matches!(
                        child.attribute("type").unwrap_or_default(),
                        "sldNum" | "sldImg" | "dt" | "hdr" | "ftr"
                    )
            })
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_note_text_and_skips_slide_number() {
        let xml = r#"
        <p:notes xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
          <p:sp><p:nvSpPr><p:nvPr><p:ph type="sldNum"/></p:nvPr></p:nvSpPr><p:txBody><a:p><a:r><a:t>1</a:t></a:r></a:p></p:txBody></p:sp>
          <p:sp><p:txBody><a:p><a:r><a:t>Hello</a:t></a:r><a:br/><a:r><a:t>World</a:t></a:r></a:p></p:txBody></p:sp>
        </p:notes>
        "#;
        assert_eq!(parse_note_xml(xml).trim(), "Hello\nWorld");
    }
}
