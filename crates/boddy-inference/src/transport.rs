use super::{ADDRESS, RESPONSE_LIMIT};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;
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

pub(crate) fn post(route: &str, request: &Value) -> Result<Value, String> {
    let payload = serde_json::to_vec(request).map_err(|e| e.to_string())?;
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
    write!(stream,"POST {route} HTTP/1.1\r\nHost: {ADDRESS}\r\nAuthorization: Bearer {key}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",payload.len()).map_err(|e| e.to_string())?;
    stream.write_all(&payload).map_err(|e| e.to_string())?;
    let mut raw = Vec::new();
    stream
        .take(RESPONSE_LIMIT + 1)
        .read_to_end(&mut raw)
        .map_err(|e| format!("Error leyendo el modelo: {e}"))?;
    if raw.len() as u64 > RESPONSE_LIMIT {
        return Err("Respuesta del modelo demasiado grande".into());
    }
    decode(&raw)
}

#[cfg(test)]
mod tests {
    use super::*;
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
