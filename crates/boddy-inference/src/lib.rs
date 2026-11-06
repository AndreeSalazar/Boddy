//! Adaptador del host a un motor externo. Solo loopback, sin herramientas o shell.
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

pub const ADDRESS: &str = "127.0.0.1:17861";
const RESPONSE_LIMIT: u64 = 1024 * 1024;

fn prepare(request: &Value) -> Result<Value, String> {
    let messages = request["messages"].as_array().ok_or("Falta messages")?;
    if messages.is_empty() || messages.len() > 24 {
        return Err("Se requieren entre 1 y 24 mensajes".into());
    }
    let mut context = vec![
        json!({"role":"system","content":"Eres Boddy 1.0, el asistente personal de Eddi. Responde en español con claridad. Tu modelo base es Qwen2.5-Coder-7B-Instruct. No afirmes haber ejecutado herramientas, leído archivos o realizado acciones: este chat solo genera texto. Distingue hechos de hipótesis. No inventes resultados de pruebas."}),
    ];
    let mut total = 0;
    for message in messages {
        let role = message["role"].as_str().ok_or("Falta role")?;
        if !matches!(role, "user" | "assistant") {
            return Err("Rol no permitido".into());
        }
        let content = message["content"].as_str().ok_or("Falta content")?;
        if content.trim().is_empty() || content.chars().count() > 8000 {
            return Err("Mensaje vacío o mayor a 8000 caracteres".into());
        }
        total += content.chars().count();
        context.push(json!({"role":role,"content":content}));
    }
    if total > 16000 {
        return Err("Conversación demasiado larga. Inicia un chat nuevo".into());
    }
    if messages.last().and_then(|m| m["role"].as_str()) != Some("user") {
        return Err("La última entrada debe ser del usuario".into());
    }
    Ok(
        json!({"model":"boddy-1.0","messages":context,"stream":false,"max_tokens":512,"temperature":0.4}),
    )
}

fn decode(raw: &[u8]) -> Result<Value, String> {
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("Respuesta HTTP incompleta")?;
    let headers = std::str::from_utf8(&raw[..split]).map_err(|e| e.to_string())?;
    if !headers
        .lines()
        .next()
        .is_some_and(|l| l.split_whitespace().nth(1) == Some("200"))
    {
        return Err(format!(
            "Motor local: {}",
            String::from_utf8_lossy(raw)
                .chars()
                .take(1000)
                .collect::<String>()
        ));
    }
    let body = &raw[split + 4..];
    let mut decoded = Vec::new();
    if headers.lines().any(|l| {
        l.to_ascii_lowercase()
            .contains("transfer-encoding: chunked")
    }) {
        let mut remaining = body;
        loop {
            let end = remaining
                .windows(2)
                .position(|w| w == b"\r\n")
                .ok_or("Chunk incompleto")?;
            let size = std::str::from_utf8(&remaining[..end])
                .map_err(|e| e.to_string())?
                .split(';')
                .next()
                .unwrap();
            let size = usize::from_str_radix(size, 16).map_err(|e| e.to_string())?;
            remaining = &remaining[end + 2..];
            if size == 0 {
                break;
            }
            if remaining.len() < size
                || remaining.get(size..size.saturating_add(2)) != Some(b"\r\n")
            {
                return Err("Chunk truncado".into());
            }
            decoded.extend_from_slice(&remaining[..size]);
            remaining = &remaining[size + 2..];
        }
    } else {
        decoded.extend_from_slice(body);
    }
    serde_json::from_slice(&decoded).map_err(|e| format!("JSON del motor inválido: {e}"))
}

pub fn chat(request: &Value) -> Result<Value, String> {
    let payload = serde_json::to_vec(&prepare(request)?).map_err(|e| e.to_string())?;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let key = std::fs::read_to_string(root.join("runtime/access.key"))
        .map_err(|_| "Inicia el modelo con window/Start-Boddy.ps1".to_string())?;
    let key = key.trim();
    if key.len() != 64 || !key.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Credencial local inválida".into());
    }
    let address: SocketAddr = ADDRESS
        .parse()
        .map_err(|e: std::net::AddrParseError| e.to_string())?;
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(3))
        .map_err(|_| "El modelo local no está iniciado. Abre window/Start-Boddy.ps1".to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(120)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| e.to_string())?;
    write!(stream,"POST /v1/chat/completions HTTP/1.1\r\nHost: {ADDRESS}\r\nAuthorization: Bearer {key}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",payload.len()).map_err(|e| e.to_string())?;
    stream.write_all(&payload).map_err(|e| e.to_string())?;
    let mut raw = Vec::new();
    stream
        .take(RESPONSE_LIMIT + 1)
        .read_to_end(&mut raw)
        .map_err(|e| format!("Error leyendo el modelo: {e}"))?;
    if raw.len() as u64 > RESPONSE_LIMIT {
        return Err("Respuesta del modelo demasiado grande".into());
    }
    let response = decode(&raw)?;
    let content = response["choices"][0]["message"]["content"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or("El modelo no devolvió texto")?;
    Ok(
        json!({"content":content,"model":"Qwen2.5-Coder-7B-Instruct Q4_K_M","application":"Boddy 1.0","finish_reason":response["choices"][0]["finish_reason"],"usage":response["usage"]}),
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
    #[test]
    fn decodes_http_and_rejects_errors_and_truncated_chunks() {
        assert_eq!(
            decode(b"HTTP/1.1 200 OK\r\n\r\n{\"ok\":true}").unwrap()["ok"],
            true
        );
        assert_eq!(decode(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nb\r\n{\"ok\":true}\r\n0\r\n\r\n").unwrap()["ok"],true);
        assert!(decode(b"HTTP/1.1 500 Bad\r\n\r\n{}").is_err());
        assert!(decode(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nff\r\nx").is_err());
    }
}
