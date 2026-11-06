pub const DATA: &str = include_str!("../../../knowledge/sources.tsv");

/// Busca metadatos locales; no descarga páginas ni hace recuperación semántica.
pub fn search(query: &str) -> Vec<&'static str> {
    let query = query.to_lowercase();
    DATA.lines()
        .skip(1)
        .filter(|line| line.to_lowercase().contains(&query))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn catalog_has_unique_ids_provenance_and_explicit_ingestion_status() {
        let mut ids = BTreeSet::new();
        for line in DATA.lines().skip(1) {
            let fields: Vec<_> = line.split('\t').collect();
            assert_eq!(fields.len(), 7, "{line}");
            assert!(ids.insert(fields[0]), "ID duplicado");
            assert!(fields[3].starts_with("https://"));
            assert_eq!(fields[5], "2026-11-05");
            assert_eq!(fields[6], "metadata_only");
        }
        assert!(ids.len() >= 15);
    }

    #[test]
    fn search_is_case_insensitive_and_reports_no_match() {
        assert_eq!(search("PYTHON"), search("python"));
        assert!(!search("python").is_empty());
        assert!(search("no-such-source-xyz").is_empty());
    }
}
