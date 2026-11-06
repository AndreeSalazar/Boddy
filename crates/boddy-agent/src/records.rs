use serde_json::{json, Value};
use std::io::Read;
use std::path::Path;

fn valid_id(id: &str) -> bool {
    id.strip_prefix("read-")
        .is_some_and(|s| !s.is_empty() && s.len() <= 32 && s.bytes().all(|c| c.is_ascii_digit()))
}

pub fn record(root: &Path, id: &str) -> Result<Value, String> {
    if !valid_id(id) {
        return Err("Identificador de tarea invalido".into());
    }
    let relative = format!("data/tasks/{id}.json");
    boddy_reader::plain_path(root, Path::new(&relative))?;
    let file = std::fs::File::open(root.join(relative)).map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("Registro no es archivo".into());
    }
    let mut text = String::new();
    file.take(131073)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    if text.len() > 131072 {
        return Err("Registro demasiado grande".into());
    }
    let value: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    if value["id"].as_str() != Some(id) {
        return Err("Identidad del registro no coincide".into());
    }
    Ok(value)
}

pub fn list(root: &Path) -> Result<Value, String> {
    let folder = Path::new("data/tasks");
    boddy_reader::plain_path(root, folder)?;
    if !root.join(folder).exists() {
        return Ok(json!({"tasks":[]}));
    }
    let mut ids = Vec::new();
    for entry in std::fs::read_dir(root.join(folder))
        .map_err(|e| e.to_string())?
        .take(2048)
    {
        let entry = entry.map_err(|e| e.to_string())?;
        if let Some(id) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.strip_suffix(".json"))
            .filter(|s| valid_id(s))
        {
            ids.push(id.to_string());
        }
    }
    ids.sort();
    ids.reverse();
    let tasks:Vec<_>=ids.iter().take(20).filter_map(|id|record(root,id).ok()).map(|v|json!({"id":v["id"],"objective":v["objective"],"state":v["state"],"model_calls":v["model_calls"]})).collect();
    Ok(json!({"tasks":tasks,"limit":20}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn record_ids_cannot_be_paths() {
        for id in [
            "../runtime/access.key",
            "read-../x",
            "read-1.json",
            "read-",
            "read-1/2",
        ] {
            assert!(!valid_id(id));
        }
        assert!(valid_id("read-1793945240659335300"));
    }
}
