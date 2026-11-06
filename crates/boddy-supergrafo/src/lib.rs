use std::io::{self, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

const OUTPUT_LIMIT: usize = 64 * 1024;
const TIME_LIMIT: Duration = Duration::from_secs(30);

fn read_bounded(reader: impl Read) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader
        .take(OUTPUT_LIMIT as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > OUTPUT_LIMIT {
        return Err(io::Error::other("SUPER GRAFO superó 64 KiB de salida"));
    }
    Ok(bytes)
}

fn poll(
    receiver: &Receiver<io::Result<Vec<u8>>>,
    target: &mut Option<Vec<u8>>,
) -> Result<(), String> {
    if target.is_some() {
        return Ok(());
    }
    match receiver.try_recv() {
        Ok(result) => *target = Some(result.map_err(|e| e.to_string())?),
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            return Err("Se interrumpió el lector de SUPER GRAFO".into())
        }
    }
    Ok(())
}

pub fn query(root: &Path, name: &str) -> Result<Vec<u8>, String> {
    if !matches!(name, "resumen" | "fallos" | "ciclos") {
        return Err("Consulta permitida: resumen, fallos o ciclos".into());
    }
    invoke(root, name, None)
}

pub fn search(root: &Path, text: &str) -> Result<Vec<u8>, String> {
    if text.trim().is_empty() || text.chars().count() > 64 || text.chars().any(char::is_control) {
        return Err("Busqueda de grafo invalida".into());
    }
    let args = serde_json::json!({"texto":text,"limite":16}).to_string();
    invoke(root, "buscar", Some(&args))
}

fn invoke(root: &Path, name: &str, args: Option<&str>) -> Result<Vec<u8>, String> {
    let binary = match std::env::var_os("BODDY_SG_BIN") {
        Some(path) => path.into(),
        None => root
            .parent()
            .ok_or("Boddy no tiene carpeta padre")?
            .join("SUPER GRAFO/ultra-carpetas/target/release/ultra-carpetas.exe"),
    };
    if !binary.is_file() {
        return Err(
            "No se encontró SUPER GRAFO. Define BODDY_SG_BIN con la ruta del ejecutable".into(),
        );
    }
    let mut command = Command::new(binary);
    command.arg("--consulta").arg(name).arg(root);
    if let Some(args) = args {
        command.arg(args);
    }
    let mut child = command
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("No se pudo iniciar SUPER GRAFO: {e}"))?;
    let result = (|| {
        let stdout = child.stdout.take().ok_or("Sin stdout de SUPER GRAFO")?;
        let stderr = child.stderr.take().ok_or("Sin stderr de SUPER GRAFO")?;
        let (out_tx, out_rx) = mpsc::channel();
        let (err_tx, err_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = out_tx.send(read_bounded(stdout));
        });
        std::thread::spawn(move || {
            let _ = err_tx.send(read_bounded(stderr));
        });
        let started = Instant::now();
        let (mut out, mut err, mut status) = (None, None, None);
        loop {
            poll(&out_rx, &mut out)?;
            poll(&err_rx, &mut err)?;
            if status.is_none() {
                status = child.try_wait().map_err(|e| e.to_string())?;
            }
            if let (Some(status), Some(out), Some(err)) = (&status, &out, &err) {
                if !status.success() {
                    return Err(format!(
                        "SUPER GRAFO terminó con {status}: {}",
                        String::from_utf8_lossy(err)
                    ));
                }
                return Ok(out.clone());
            }
            if started.elapsed() >= TIME_LIMIT {
                return Err("SUPER GRAFO excedió 30 segundos".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    })();
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unapproved_queries_are_rejected_before_launching_any_process() {
        for name in ["guardar", "gcc_diagnosticos", "nota", "resumen; calc.exe"] {
            assert!(query(Path::new("."), name)
                .unwrap_err()
                .starts_with("Consulta permitida"));
        }
    }

    #[test]
    fn output_limit_is_exact_and_excess_is_rejected() {
        assert_eq!(read_bounded(&b"ok"[..]).unwrap(), b"ok");
        assert_eq!(
            read_bounded(&vec![0; OUTPUT_LIMIT][..]).unwrap().len(),
            OUTPUT_LIMIT
        );
        assert!(read_bounded(&vec![0; OUTPUT_LIMIT + 1][..]).is_err());
    }
}
