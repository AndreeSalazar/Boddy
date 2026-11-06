use serde_json::Value;

pub const CONTEXT_CAPACITY: usize = 8192;
pub const SAFETY_MARGIN: usize = 64;

pub fn settings(request: &Value) -> Result<(usize, usize), String> {
    fn option(
        request: &Value,
        key: &str,
        default: usize,
        choices: &[usize],
    ) -> Result<usize, String> {
        let value = match request.get(key) {
            None => default,
            Some(v) => v
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .ok_or_else(|| format!("{key} inválido"))?,
        };
        if !choices.contains(&value) {
            return Err(format!("{key} fuera de las opciones permitidas"));
        }
        Ok(value)
    }
    Ok((
        option(
            request,
            "context_tokens",
            2048,
            &[512, 1024, 2048, 4096, 8192],
        )?,
        option(request, "max_tokens", 256, &[64, 128, 256, 512, 1024])?,
    ))
}

/// Cuenta con el tokenizer del motor; elimina pares antiguos, nunca la última petición.
pub fn fit(
    body: &mut Value,
    limit: usize,
    output: usize,
    mut count: impl FnMut(&Value) -> Result<usize, String>,
) -> Result<(usize, usize, usize), String> {
    let before = count(body)?;
    let mut current = before;
    let mut dropped = 0;
    while current.saturating_add(output).saturating_add(SAFETY_MARGIN) > limit {
        let messages = body["messages"].as_array_mut().ok_or("Faltan mensajes")?;
        if messages.len() <= 3 {
            return Err("Tu mensaje y la respuesta reservada superan el presupuesto. Aumenta el contexto o acorta el mensaje".into());
        }
        messages.drain(1..3);
        dropped += 2;
        current = count(body)?;
    }
    Ok((before, current, dropped))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn discards_whole_old_turns_preserves_system_and_latest_question() {
        let mut body = json!({"messages":[{"role":"system","content":"rules"},{"role":"user","content":"old"},{"role":"assistant","content":"old reply"},{"role":"user","content":"latest"}]});
        let (_, after, dropped) = fit(&mut body, 300, 64, |b| {
            Ok(b["messages"].as_array().unwrap().len() * 80)
        })
        .unwrap();
        assert_eq!(dropped, 2);
        assert!(after + 64 + SAFETY_MARGIN <= 300);
        assert_eq!(body["messages"][0]["content"], "rules");
        assert_eq!(body["messages"][1]["content"], "latest");
    }
    #[test]
    fn refuses_to_silently_cut_the_current_question_or_accept_bad_settings() {
        let mut body = json!({"messages":[{},{}]});
        assert!(fit(&mut body, 512, 256, |_| Ok(400)).is_err());
        assert!(settings(&json!({"max_tokens":100000})).is_err());
        assert!(settings(&json!({"context_tokens":"2048"})).is_err());
    }
}
