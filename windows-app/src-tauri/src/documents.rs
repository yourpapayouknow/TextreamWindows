use crate::models::SaveDocumentRequest;
use crate::pptx;
use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub fn open_document(path: String) -> Result<Vec<String>, String> {
    let path = PathBuf::from(path);
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or_default().to_ascii_lowercase().as_str() {
        "textream" => {
            let data = fs::read_to_string(&path)
                .map_err(|err| format!("Failed to read document: {err}"))?;
            let pages: Vec<String> =
                serde_json::from_str(&data).map_err(|err| format!("Invalid .textream document: {err}"))?;
            if pages.is_empty() {
                Err("Document has no pages".to_string())
            } else {
                Ok(pages)
            }
        }
        "pptx" => extract_pptx_notes(path.to_string_lossy().to_string()),
        "key" => Err("Keynote files cannot be imported directly. Export to PowerPoint (.pptx) first.".to_string()),
        _ => Err("Unsupported file. Use .textream or .pptx.".to_string()),
    }
}

#[tauri::command]
pub fn save_document(request: SaveDocumentRequest) -> Result<(), String> {
    let path = PathBuf::from(request.path);
    let data = serde_json::to_string_pretty(&request.pages)
        .map_err(|err| format!("Could not encode document: {err}"))?;
    fs::write(path, data).map_err(|err| format!("Could not save document: {err}"))
}

#[tauri::command]
pub fn extract_pptx_notes(path: String) -> Result<Vec<String>, String> {
    pptx::extract_notes(&PathBuf::from(path)).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn textream_document_shape_is_string_array_json() {
        let pages = vec!["one".to_string(), "two".to_string()];
        let data = serde_json::to_string(&pages).unwrap();
        let decoded: Vec<String> = serde_json::from_str(&data).unwrap();
        assert_eq!(decoded, pages);
    }
}

