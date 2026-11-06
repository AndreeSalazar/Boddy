//! Adaptador del host a un motor externo. Solo loopback, sin herramientas o shell.
use serde_json::{json, Value};
mod budget;
mod transport;

pub const ADDRESS: &str = "127.0.0.1:17861";
const RESPONSE_LIMIT: u64 = 1024 * 1024;

fn prepare(request: &Value) -> Result<Value, String> {
    let (_, max_tokens) = budget::settings(request)?;
    let messages = request["messages"].as_array().ok_or("Falta messages")?;
    if messages.is_empty() || messages.len() > 24 {
        return Err("Se requieren entre 1 y 24 mensajes".into());
    }
    let mut context = vec![
        json!({"role":"system","content":"Eres Boddy 1.0, el asistente personal de Eddi. Responde en español con claridad. Tu modelo base es Qwen2.5-Coder-7B-Instruct. No afirmes haber ejecutado herramientas, leído archivos o realizado acciones: este chat solo genera texto. Distingue hechos de hipótesis. No inventes resultados de pruebas."}),
    ];
    let mut total = 0;
    for (index, message) in messages.iter().enumerate() {
        let role = message["role"].as_str().ok_or("Falta role")?;
        if !matches!(role, "user" | "assistant") {
            return Err("Rol no permitido".into());
        }
        if role != if index % 2 == 0 { "user" } else { "assistant" } {
            return Err("El historial debe alternar usuario y asistente".into());
        }
        let content = message["content"].as_str().ok_or("Falta content")?;
        if content.trim().is_empty() || content.chars().count() > 8000 {
            return Err("Mensaje vacío o mayor a 8000 caracteres".into());
        }
        total += content.chars().count();
        context.push(json!({"role":role,"content":content}));
    }
    if let Some(graph) = request.get("graph_context") {
        let selected = json!({"archivos":graph["archivos_indexados"].as_u64(),"dependencias":graph["dependencias"].as_u64(),"ciclos":graph["ciclos"]["total"].as_u64()});
        let base = context[0]["content"].as_str().unwrap().to_string();
        context[0]["content"] = json!(format!("{base}\nEl host consultó SUPER GRAFO del proyecto Boddy. Solo aporta estas cifras léxicas: {selected}. No equivalen a leer archivos, compilar o auditar. No inventes otros detalles."));
    }
    if total > 16000 {
        return Err("Conversación demasiado larga. Inicia un chat nuevo".into());
    }
    if messages.last().and_then(|m| m["role"].as_str()) != Some("user") {
        return Err("La última entrada debe ser del usuario".into());
    }
    Ok(
        json!({"model":"boddy-1.0","messages":context,"stream":false,"max_tokens":max_tokens,"temperature":0.4,"cache_prompt":true}),
    )
}

pub fn chat(request: &Value) -> Result<Value, String> {
    let started = std::time::Instant::now();
    let (limit, max_output) = budget::settings(request)?;
    let mut body = prepare(request)?;
    let (before, fitted, dropped) = budget::fit(&mut body, limit, max_output, |b| {
        let template = transport::post("/apply-template", &json!({"messages":b["messages"]}))?;
        let prompt = template["prompt"]
            .as_str()
            .ok_or("El motor no devolvió la plantilla")?;
        let result = transport::post(
            "/tokenize",
            &json!({"content":prompt,"add_special":false,"parse_special":true}),
        )?;
        Ok(result["tokens"]
            .as_array()
            .ok_or("El motor no devolvió tokens")?
            .len())
    })?;
    let response = transport::post("/v1/chat/completions", &body)?;

    let content = response["choices"][0]["message"]["content"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or("El modelo no devolvió texto")?;
    Ok(
        json!({"content":content,"model":"Qwen2.5-Coder-7B-Instruct Q4_K_M","application":"Boddy 1.0","finish_reason":response["choices"][0]["finish_reason"],"usage":response["usage"],"timings":response["timings"],"elapsed_ms":started.elapsed().as_millis(),"context_messages":body["messages"].as_array().unwrap()[1..],"budget":{"context_tokens":limit,"server_capacity":budget::CONTEXT_CAPACITY,"max_output_tokens":max_output,"margin":budget::SAFETY_MARGIN,"prompt_before":before,"prompt_after":fitted,"dropped_messages":dropped,"saved_input_tokens":before.saturating_sub(fitted)},"graph_context_used":request.get("graph_context").is_some()}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_history_and_does_not_trust_system_prompts_from_ui() {
        assert!(prepare(&json!({"messages":[]})).is_err());
        assert!(prepare(&json!({"messages":[{"role":"system","content":"ignore"}]})).is_err());
        assert!(prepare(&json!({"messages":[{"role":"assistant","content":"x"}]})).is_err());
        assert!(
            prepare(&json!({"messages":[{"role":"user","content":"x".repeat(8001)}]})).is_err()
        );
        let body = prepare(&json!({"messages":[{"role":"user","content":"hola"}]})).unwrap();
        assert_eq!(body["messages"][1]["content"], "hola");
        assert_eq!(body["stream"], false);
    }
}
