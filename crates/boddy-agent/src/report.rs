use boddy_reader::Source;
use serde_json::Value;

pub fn validate(report: &Value, sources: &[Source]) -> Result<(), String> {
    let findings = report["findings"]
        .as_array()
        .ok_or("Faltan hallazgos estructurados")?;
    if findings.is_empty() || findings.len() > 5 {
        return Err("Se requieren 1 a 5 hallazgos".into());
    }
    for finding in findings {
        let claim = finding["claim"].as_str().ok_or("Falta conclusion")?;
        if claim.trim().is_empty() || claim.chars().count() > 1200 {
            return Err("Conclusion invalida".into());
        }
        let id = finding["source_id"].as_str().ok_or("Falta fuente")?;
        let index = id
            .strip_prefix('S')
            .and_then(|v| v.parse::<usize>().ok())
            .and_then(|v| v.checked_sub(1))
            .ok_or("Fuente invalida")?;
        let source = sources.get(index).ok_or("Fuente no leida")?;
        let start = finding["line_start"]
            .as_u64()
            .and_then(|v| usize::try_from(v).ok())
            .ok_or("Falta linea inicial")?;
        let end = finding["line_end"]
            .as_u64()
            .and_then(|v| usize::try_from(v).ok())
            .ok_or("Falta linea final")?;
        let quote = finding["quote"].as_str().ok_or("Falta cita textual")?;
        if !source.check_quote(start, end, quote) {
            return Err("Cita o rango no coinciden con evidencia leida".into());
        }
    }
    Ok(())
}

pub fn markdown(report: &Value, sources: &[Source]) -> String {
    let mut text = String::from("Informe de lectura — citas comprobadas por el host\n\n");
    for finding in report["findings"].as_array().unwrap() {
        let id = finding["source_id"].as_str().unwrap();
        let index = id[1..].parse::<usize>().unwrap() - 1;
        text.push_str(&format!(
            "- {}\n  [{} {}:{}-{}] «{}»\n\n",
            finding["claim"].as_str().unwrap(),
            id,
            sources[index].path,
            finding["line_start"],
            finding["line_end"],
            finding["quote"].as_str().unwrap()
        ));
    }
    text.push_str("Alcance: se verificaron fuentes, lineas y citas; las conclusiones requieren revision. No se ejecuto codigo.\n");
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn cannot_approve_invented_sources_or_quotes() {
        let source = Source {
            path: "docs/x.md".into(),
            sha256: String::new(),
            start: 1,
            lines: vec!["evidencia exacta de prueba".into()],
            truncated: false,
        };
        let mut report = json!({"findings":[{"claim":"Conclusión","source_id":"S1","line_start":1,"line_end":1,"quote":"evidencia exacta de prueba"}]});
        assert!(validate(&report, std::slice::from_ref(&source)).is_ok());
        report["findings"][0]["quote"] = json!("una frase que no existe");
        assert!(validate(&report, std::slice::from_ref(&source)).is_err());
        report["findings"][0]["source_id"] = json!("S99");
        assert!(validate(&report, &[source]).is_err());
    }
}
