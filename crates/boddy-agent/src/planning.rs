/// Un nombre Rust puede referirse al componente, cuyo archivo real es src/lib.rs.
pub fn query_alias(proposed: &str) -> String {
    let proposed = proposed.trim();
    if !proposed.contains(['/', '\\']) && !matches!(proposed, "lib.rs" | "main.rs") {
        if let Some(stem) = proposed.strip_suffix(".rs").filter(|s| !s.is_empty()) {
            return stem.to_string();
        }
    }
    proposed.to_string()
}

/// Un resultado de carpeta de SG se resuelve a entradas Rust existentes, sin leerlas.
pub fn component_entries(root: &std::path::Path, folder: &str) -> Vec<String> {
    if !folder.starts_with("crates/") || folder.split('/').count() != 2 {
        return Vec::new();
    }
    ["src/lib.rs", "src/main.rs", "Cargo.toml"]
        .iter()
        .map(|entry| format!("{folder}/{entry}"))
        .filter(|p| {
            boddy_reader::allowed(p)
                && boddy_reader::plain_path(root, std::path::Path::new(p)).is_ok()
                && root.join(p).is_file()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolves_component_names_without_rewriting_explicit_paths() {
        assert_eq!(query_alias("policy.rs"), "policy");
        assert_eq!(query_alias("budget.rs"), "budget");
        assert_eq!(query_alias("src/lib.rs"), "src/lib.rs");
        assert_eq!(query_alias("lib.rs"), "lib.rs");
    }
    #[test]
    fn expands_only_existing_project_component_entries() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        assert!(component_entries(root, "crates/boddy-policy")
            .contains(&"crates/boddy-policy/src/lib.rs".into()));
        assert!(component_entries(root, "runtime").is_empty());
        assert!(component_entries(root, "crates/../runtime").is_empty());
    }
}
