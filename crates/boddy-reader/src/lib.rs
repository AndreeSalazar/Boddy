//! Lecturas limitadas de fuentes del proyecto. No es un sandbox del SO.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Component, Path};

pub fn allowed(path: &str) -> bool {
    if path.is_empty()
        || path.contains(['\\', ':', '\0'])
        || path.split('/').any(|p| p.is_empty() || p.starts_with('.'))
    {
        return false;
    }
    if Path::new(path)
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return false;
    }
    let scope = path.starts_with("crates/")
        || path.starts_with("docs/")
        || path.starts_with("window/")
        || matches!(path, "README.md" | "Cargo.toml");
    scope
        && matches!(
            Path::new(path).extension().and_then(|x| x.to_str()),
            Some("rs" | "md" | "toml" | "ps1" | "vbs")
        )
}

pub fn plain_path(root: &Path, relative: &Path) -> Result<(), String> {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err("Ruta fuera de alcance".into());
        }
        current.push(component);
        if let Ok(metadata) = std::fs::symlink_metadata(&current) {
            let mut linked = metadata.file_type().is_symlink();
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                linked |= metadata.file_attributes() & 0x400 != 0;
            }
            if linked {
                return Err("Enlaces y puntos de reanalisis no permitidos".into());
            }
        }
    }
    Ok(())
}

pub fn list(root: &Path) -> Result<Value, String> {
    let mut pending = vec![
        "docs".to_string(),
        "crates".to_string(),
        "window".to_string(),
    ];
    let mut files = vec!["README.md".to_string(), "Cargo.toml".to_string()];
    let mut visited = 0;
    while let Some(relative) = pending.pop() {
        if visited >= 2048 || files.len() >= 200 {
            return Ok(json!({"files":files,"truncated":true}));
        }
        if plain_path(root, Path::new(&relative)).is_err() {
            continue;
        }
        for entry in std::fs::read_dir(root.join(&relative)).map_err(|e| e.to_string())? {
            visited += 1;
            if visited > 2048 || files.len() >= 200 {
                return Ok(json!({"files":files,"truncated":true}));
            }
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.')
                || matches!(name.as_str(), "target" | "node_modules" | "vendor")
            {
                continue;
            }
            let path = format!("{relative}/{name}");
            if plain_path(root, Path::new(&path)).is_err() {
                continue;
            }
            let metadata = entry.metadata().map_err(|e| e.to_string())?;
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file() && allowed(&path) {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(json!({"files":files,"truncated":false}))
}

#[derive(Clone, Debug)]
pub struct Source {
    pub sha256: String,
    pub path: String,
    pub start: usize,
    pub lines: Vec<String>,
    pub truncated: bool,
}

impl Source {
    pub fn as_json(&self, id: &str) -> Value {
        json!({"id":id,"path":self.path,"sha256":self.sha256,"start":self.start,"end":self.start+self.lines.len()-1,"truncated":self.truncated,"lines":self.lines.iter().enumerate().map(|(i,l)|format!("{}: {}",self.start+i,l)).collect::<Vec<_>>()})
    }
    pub fn check_quote(&self, start: usize, end: usize, quote: &str) -> bool {
        if start < self.start
            || end < start
            || end >= self.start + self.lines.len()
            || quote.trim().chars().count() < 12
        {
            return false;
        }
        self.lines[start - self.start..=end - self.start]
            .join("\n")
            .contains(quote)
    }
}

pub fn read(root: &Path, path: &str, terms: &[String]) -> Result<Source, String> {
    if !allowed(path) {
        return Err("Archivo fuera del alcance de lectura".into());
    }
    plain_path(root, Path::new(path))?;
    let canonical = root.canonicalize().map_err(|e| e.to_string())?;
    let target = canonical
        .join(path)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !target.starts_with(&canonical) {
        return Err("Archivo externo al proyecto".into());
    }
    let file = std::fs::File::open(&target).map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("No es archivo regular".into());
    }
    let mut text = String::new();
    file.take(262145)
        .read_to_string(&mut text)
        .map_err(|_| "Fuente no es texto UTF8".to_string())?;
    if text.len() > 262144 || text.contains('\0') {
        return Err("Fuente binaria o demasiado grande".into());
    }
    let all: Vec<_> = text.lines().collect();
    if all.is_empty() {
        return Err("Archivo vacio".into());
    }
    let hit = all
        .iter()
        .position(|l| {
            terms
                .iter()
                .any(|t| !t.is_empty() && l.to_lowercase().contains(&t.to_lowercase()))
        })
        .unwrap_or(0);
    let start = hit.saturating_sub(4);
    let mut lines = Vec::new();
    let mut size = 0;
    for line in all.iter().skip(start).take(35) {
        size += line.chars().count();
        if size > 1600 {
            break;
        }
        lines.push((*line).to_string());
    }
    if lines.is_empty() {
        return Err("Primera linea supera el presupuesto de lectura".into());
    }
    let truncated = start > 0 || start + lines.len() < all.len();
    Ok(Source {
        sha256: format!("{:x}", Sha256::digest(text.as_bytes())),
        path: path.into(),
        start: start + 1,
        lines,
        truncated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn denies_external_private_and_runtime_paths() {
        for p in [
            "../secret.rs",
            "crates/../README.md",
            "C:/secret.rs",
            "crates\\x.rs",
            "runtime/access.key",
            ".git/config",
            "docs/.env",
            "models/x.md",
            "/docs/x.md",
        ] {
            assert!(!allowed(p), "{p}");
        }
        assert!(allowed("crates/boddy-core/src/task.rs"));
        assert!(allowed("window/components/Workbench.ps1"));
    }
    #[test]
    fn validates_quotes_against_exact_line_ranges() {
        let s = Source {
            sha256: String::new(),
            path: "docs/test.md".into(),
            start: 10,
            lines: vec![
                "una evidencia comprobable".into(),
                "segunda linea de prueba".into(),
            ],
            truncated: true,
        };
        assert!(s.check_quote(10, 10, "evidencia comprobable"));
        assert!(!s.check_quote(11, 11, "evidencia comprobable"));
        assert!(!s.check_quote(9, 10, "evidencia comprobable"));
        assert!(!s.check_quote(10, 100, "evidencia comprobable"));
    }

    #[test]
    fn reads_bounded_real_sources_with_a_fingerprint() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let source = read(
            root,
            "crates/boddy-reader/src/lib.rs",
            &["pub fn allowed".into()],
        )
        .unwrap();
        assert!(source.lines.len() <= 35);
        assert_eq!(source.sha256.len(), 64);
        assert!(source.truncated);
        assert!(source.lines.iter().any(|l| l.contains("pub fn allowed")));
        assert!(read(root, "runtime/access.key", &[]).is_err());
        assert!(read(root, "crates/../README.md", &[]).is_err());
    }
}
