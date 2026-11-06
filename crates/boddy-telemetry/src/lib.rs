//! Muestras del dispositivo completo, no del proceso ni del pico de VRAM.
use serde_json::{json, Value};
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn parse(text: &str) -> Option<Value> {
    let line = text.lines().next()?;
    let parts: Vec<_> = line.split(',').map(str::trim).collect();
    if parts.len() != 4 {
        return None;
    }
    let index = parts[0].parse::<u32>().ok()?;
    let used = parts[1].parse::<u64>().ok()?;
    let total = parts[2].parse::<u64>().ok()?;
    let utilization = parts[3].parse::<u32>().ok()?;
    if used > total || utilization > 100 {
        return None;
    }
    Some(
        json!({"available":true,"gpu_index":index,"used_mib":used,"total_mib":total,"free_mib":total-used,"utilization_percent":utilization,"scope":"device_snapshot_not_process_or_peak"}),
    )
}

fn sample() -> Option<Value> {
    let binary =
        std::path::PathBuf::from(std::env::var_os("SystemRoot")?).join("System32/nvidia-smi.exe");
    let mut command = Command::new(binary);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command
        .args([
            "--query-gpu=index,memory.used,memory.total,utilization.gpu",
            "--format=csv,noheader,nounits",
            "--id=0",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return None;
                }
                break;
            }
            Ok(None) if started.elapsed() < Duration::from_secs(3) => {
                std::thread::sleep(Duration::from_millis(30))
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
    let mut text = String::new();
    child
        .stdout
        .take()?
        .take(4097)
        .read_to_string(&mut text)
        .ok()?;
    if text.len() > 4096 {
        return None;
    }
    parse(&text)
}

pub fn snapshot() -> Value {
    sample()
        .unwrap_or_else(|| json!({"available":false,"scope":"device_snapshot_not_process_or_peak"}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unknown_readings_and_labels_scope() {
        assert!(parse("0,N/A,12288,5").is_none());
        assert!(parse("0,13000,12288,5").is_none());
        let result = parse("0, 5600, 12288, 12").unwrap();
        assert_eq!(result["free_mib"], 6688);
        assert_eq!(result["scope"], "device_snapshot_not_process_or_peak");
    }
}
