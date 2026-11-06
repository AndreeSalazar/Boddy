use boddy_reader::Source;
use serde_json::Value;

/// Anclas canónicas: el modelo elige IDs, el host conserva texto y localización.
pub fn anchors(sources: &[Source]) -> Value {
    let mut anchors = Vec::new();
    for (source_index, source) in sources.iter().enumerate() {
        let mut lines: Vec<_> = source
            .lines
            .iter()
            .enumerate()
            .filter(|(_, line)| {
                let t = line.trim();
                (12..=180).contains(&t.chars().count())
                    && !(t.contains("fn ") && t.ends_with('{'))
                    && !t.starts_with("pub struct ")
                    && !t.starts_with("use ")
            })
            .map(|(offset, line)| {
                let t = line.trim();
                let score = usize::from(t.contains("false") || t.contains("true")) * 5
                    + usize::from(t.starts_with("if ") || t.starts_with("pub const ")) * 4
                    + usize::from(t.contains("=>") || t.contains("return Err")) * 3;
                (offset, line, score)
            })
            .collect();
        lines.sort_by_key(|(offset, _, score)| (std::cmp::Reverse(*score), *offset));
        lines.truncate(6);
        lines.sort_by_key(|(offset, _, _)| *offset);
        for (offset, line, _) in lines {
            let number = source.start + offset;
            anchors.push(serde_json::json!({"citation_id":format!("S{}:L{number}",source_index+1),"source_id":format!("S{}",source_index+1),"path":source.path,"line":number,"quote":line}));
        }
    }
    Value::Array(anchors)
}

pub fn materialize(raw: &Value, sources: &[Source]) -> Result<Value, String> {
    let catalog = anchors(sources);
    let mut findings = Vec::new();
    for finding in raw["findings"].as_array().ok_or("Falta findings")? {
        let object = finding.as_object().ok_or("Hallazgo no es objeto")?;
        if object
            .keys()
            .any(|key| !matches!(key.as_str(), "claim" | "citation_id"))
        {
            return Err(
                "Solo se permiten claim y citation_id; las citas las resuelve el host".into(),
            );
        }
        let id = finding["citation_id"].as_str().ok_or("Falta citation_id")?;
        let anchor = catalog
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["citation_id"].as_str() == Some(id))
            .ok_or_else(|| format!("Ancla {id} no disponible; elige un citation_id de la lista"))?;
        findings.push(serde_json::json!({"claim":finding["claim"],"citation_id":id,"source_id":anchor["source_id"],"line_start":anchor["line"],"line_end":anchor["line"],"quote":anchor["quote"]}));
    }
    Ok(serde_json::json!({"findings":findings}))
}

pub fn validate(report: &Value, sources: &[Source]) -> Result<(), String> {
    let findings = report["findings"]
        .as_array()
        .ok_or("Faltan hallazgos estructurados")?;
    if findings.is_empty() || findings.len() > 5 {
        return Err("Se requieren 1 a 5 hallazgos".into());
    }
    for (number, finding) in findings.iter().enumerate() {
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
        let literal_false = quote
            .trim()
            .trim_end_matches(',')
            .split_once(':')
            .is_some_and(|(_, value)| value.trim() == "false");
        let words: Vec<_> = claim
            .split(|c: char| !c.is_alphanumeric())
            .map(str::to_lowercase)
            .collect();
        let affirmative = words.iter().any(|w| {
            matches!(
                w.as_str(),
                "permite" | "habilita" | "activa" | "allows" | "enables"
            )
        });
        let negative = words.iter().any(|w| {
            matches!(
                w.as_str(),
                "no" | "deniega"
                    | "desactiva"
                    | "desactivado"
                    | "desactivada"
                    | "deshabilita"
                    | "impide"
                    | "prohibe"
                    | "false"
                    | "not"
                    | "denies"
                    | "disabled"
            )
        });
        if literal_false && affirmative && !negative {
            return Err(format!("Hallazgo {}: afirmacion de permiso afirmativo contradice un inicializador false. Describe el valor observado, sin atribuir permisos globales: {quote}",number+1));
        }
        if !source.check_quote(start, end, quote) {
            let matches: Vec<_> = source
                .lines
                .iter()
                .enumerate()
                .filter(|(_, line)| quote.chars().count() >= 12 && line.contains(quote))
                .take(3)
                .map(|(i, _)| source.start + i)
                .collect();
            return Err(format!("Hallazgo {}: la cita no coincide con {id}:{start}-{end}. Coincidencias textuales dentro del fragmento de {id}: {matches:?}. Corrige usando solo evidencia disponible; no inventes una cita.",number+1));
        }
    }
    Ok(())
}

/// Revisión acotada: ni el modelo ni la corrección pueden relajar el verificador.
pub fn verify_with_retry(
    mut response: Result<Value, String>,
    sources: &[Source],
    repairs: u8,
    attempts: &mut Vec<Value>,
    mut retry: impl FnMut(&str, &[Value]) -> Result<Value, String>,
) -> Result<Value, String> {
    if repairs > 1 {
        return Err("Solo se permite una correccion".into());
    }
    for attempt in 0..=repairs {
        let validation = match &response {
            Ok(report) => validate(report, sources),
            Err(error) => Err(error.clone()),
        };
        attempts.push(serde_json::json!({"attempt":attempt+1,"report":response.as_ref().ok(),"validation_error":validation.as_ref().err(),"verified":validation.is_ok()}));
        match validation {
            Ok(()) => return response,
            Err(error) if attempt < repairs => response = retry(&error, attempts),
            Err(error) => return Err(error),
        }
    }
    unreachable!()
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

    #[test]
    fn revision_can_fix_line_numbers_but_cannot_bypass_validation() {
        let source = Source {
            path: "docs/x.md".into(),
            sha256: String::new(),
            start: 20,
            lines: vec!["una evidencia verificable".into()],
            truncated: false,
        };
        let bad = serde_json::json!({"findings":[{"claim":"Resultado","source_id":"S1","line_start":12,"line_end":12,"quote":"una evidencia verificable"}]});
        let mut attempts = Vec::new();
        let mut calls = 0;
        let good = verify_with_retry(
            Ok(bad.clone()),
            std::slice::from_ref(&source),
            1,
            &mut attempts,
            |error, _| {
                calls += 1;
                assert!(error.contains("20"));
                let mut corrected = bad.clone();
                corrected["findings"][0]["line_start"] = 20.into();
                corrected["findings"][0]["line_end"] = 20.into();
                Ok(corrected)
            },
        )
        .unwrap();
        assert!(validate(&good, std::slice::from_ref(&source)).is_ok());
        assert_eq!(calls, 1);
        assert_eq!(attempts.len(), 2);
        assert_eq!(attempts[0]["verified"], false);
        let mut rejected = Vec::new();
        assert!(
            verify_with_retry(Ok(bad.clone()), &[source], 1, &mut rejected, |_, _| Ok(
                bad.clone()
            ))
            .is_err()
        );
        assert_eq!(rejected.len(), 2);
    }

    #[test]
    fn cannot_retry_without_budget_or_approve_invalid_json() {
        let mut attempts = Vec::new();
        assert!(verify_with_retry(
            Err("JSON invalido".into()),
            &[],
            0,
            &mut attempts,
            |_, _| panic!("No debe reintentar")
        )
        .is_err());
        assert_eq!(attempts.len(), 1);
        let mut attempts = Vec::new();
        assert!(verify_with_retry(
            Err("JSON invalido".into()),
            &[],
            1,
            &mut attempts,
            |_, _| Ok(serde_json::json!({"findings":[]}))
        )
        .is_err());
        assert_eq!(attempts.len(), 2);
        assert!(attempts.iter().all(|a| a["verified"] == false));
    }

    #[test]
    fn host_resolves_only_known_anchors_and_rejects_text_overrides() {
        let sources = [Source {
            path: "docs/x.md".into(),
            sha256: String::new(),
            start: 20,
            lines: vec!["una evidencia verificable".into()],
            truncated: false,
        }];
        let raw = serde_json::json!({"findings":[{"claim":"Conclusion pendiente","citation_id":"S1:L20"}]});
        let resolved = materialize(&raw, &sources).unwrap();
        assert!(validate(&resolved, &sources).is_ok());
        assert_eq!(
            resolved["findings"][0]["quote"],
            "una evidencia verificable"
        );
        let mut invalid = raw.clone();
        invalid["findings"][0]["citation_id"] = "S1:L12".into();
        assert!(materialize(&invalid, &sources).is_err());
        let mut invalid = raw;
        invalid["findings"][0]["quote"] = "texto inventado".into();
        assert!(materialize(&invalid, &sources).is_err());
    }
    #[test]
    fn function_signatures_are_not_behavior_anchors() {
        let sources = [Source {
            path: "docs/x.md".into(),
            sha256: String::new(),
            start: 1,
            lines: vec![
                "pub fn permits_execution() -> bool {".into(),
                "    execute_code: false,".into(),
            ],
            truncated: false,
        }];
        let anchors = anchors(&sources);
        assert_eq!(anchors.as_array().unwrap().len(), 1);
        assert_eq!(anchors[0]["line"], 2);
    }
    #[test]
    fn flags_obvious_permission_claims_against_a_false_initializer() {
        let sources = [Source {
            path: "docs/x.md".into(),
            sha256: String::new(),
            start: 1,
            lines: vec!["execute_code: false,".into()],
            truncated: false,
        }];
        let raw = serde_json::json!({"findings":[{"claim":"Permite ejecutar codigo","citation_id":"S1:L1"}]});
        let mut report = materialize(&raw, &sources).unwrap();
        assert!(validate(&report, &sources).unwrap_err().contains("false"));
        report["findings"][0]["claim"] = "No permite ejecutar codigo en este inicializador".into();
        assert!(validate(&report, &sources).is_ok());
    }
}
