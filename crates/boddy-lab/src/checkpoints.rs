//! Carga acotada y validacion de modelos locales; no acepta rutas del solicitante.
use super::{CONFIG, CORPUS};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path};

pub(crate) struct Saved {
    pub weights: Vec<f32>,
    pub sampler: u32,
    pub completed: Option<u64>,
    pub legacy: bool,
}
pub(crate) fn valid_id(id: &str) -> bool {
    id.strip_prefix("tiny-").is_some_and(|suffix| {
        !suffix.is_empty() && suffix.len() <= 30 && suffix.bytes().all(|b| b.is_ascii_digit())
    })
}
pub(crate) fn read(root: &Path, id: &str, file: &str, limit: u64) -> Result<Value, String> {
    if !valid_id(id) {
        return Err("Identificador Tiny invalido".into());
    }
    let relative = format!("data/lab/{id}/{file}");
    boddy_reader::plain_path(root, Path::new(&relative))?;
    let mut bytes = Vec::new();
    fs::File::open(root.join(relative))
        .map_err(|e| e.to_string())?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("Archivo de modelo demasiado grande".into());
    }
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
fn parse(value: &Value) -> Result<Saved, String> {
    let schema = value["schema"]
        .as_u64()
        .ok_or("Falta version de checkpoint")?;
    if !matches!(schema, 1 | 2)
        || value["architecture"] != "one_hot_tanh_softmax_mlp"
        || value["context"] != CONFIG.context
        || value["hidden"] != CONFIG.hidden
        || value["vocab"] != CONFIG.vocab
        || value["tokenizer"] != "newline=0; printable ASCII 32..126=1..95"
    {
        return Err("Arquitectura o tokenizer no compatibles con Tiny 1.0".into());
    }
    let raw = value["weights"]
        .as_array()
        .filter(|v| v.len() == CONFIG.parameters())
        .ok_or("Forma de pesos incorrecta")?;
    let weights: Vec<_> = raw
        .iter()
        .map(|v| {
            v.as_f64()
                .map(|v| v as f32)
                .filter(|v| v.is_finite())
                .ok_or("Peso no finito o invalido")
        })
        .collect::<Result<_, _>>()?;
    let legacy = schema == 1;
    let sampler = if legacy {
        42
    } else {
        value["sampler_state"]
            .as_u64()
            .and_then(|v| u32::try_from(v).ok())
            .ok_or("Estado de muestreo invalido")?
    };
    let completed = if legacy || value["completed_steps"].is_null() {
        None
    } else {
        Some(
            value["completed_steps"]
                .as_u64()
                .ok_or("Pasos acumulados invalidos")?,
        )
    };
    Ok(Saved {
        weights,
        sampler,
        completed,
        legacy,
    })
}
pub(crate) fn load(root: &Path, id: &str, resume: bool) -> Result<Saved, String> {
    let value = read(root, id, "checkpoint.json", 2 * 1024 * 1024)?;
    let saved = parse(&value)?;
    if resume {
        let hash = format!("{:x}", Sha256::digest(CORPUS.as_bytes()));
        let recorded = if saved.legacy {
            read(root, id, "metrics.json", 65536)?["corpus_sha256"].clone()
        } else {
            value["corpus_sha256"].clone()
        };
        if recorded != hash {
            return Err("Corpus distinto: no se puede reanudar con el protocolo Tiny v1".into());
        }
        if !saved.legacy && value["training_protocol"] != "tiny-sgd-v1" {
            return Err("Protocolo de entrenamiento incompatible".into());
        }
    }
    Ok(saved)
}

pub fn list(root: &Path) -> Result<Value, String> {
    boddy_reader::plain_path(root, Path::new("data/lab"))?;
    let entries = match fs::read_dir(root.join("data/lab")) {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(json!({"models":[],"invalid":0,"truncated":false}))
        }
        Err(e) => return Err(e.to_string()),
    };
    let mut ids = Vec::new();
    let mut scanned = 0;
    for entry in entries.take(513) {
        scanned += 1;
        if scanned > 512 {
            break;
        }
        let entry = entry.map_err(|e| e.to_string())?;
        let id = entry.file_name().to_string_lossy().into_owned();
        if valid_id(&id) {
            ids.push(id);
        }
    }
    ids.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| b.cmp(a)));
    let truncated = scanned > 512 || ids.len() > 64;
    let mut models = Vec::new();
    let mut invalid = 0;
    for id in ids.iter().take(64) {
        match load(root,id,false).and_then(|saved|Ok((saved,read(root,id,"metrics.json",65536)?))) {
            Ok((saved,metrics))=>models.push(json!({"id":id,"model":"Boddy Tiny 1.0","parameters":CONFIG.parameters(),"completed_steps":saved.completed,"legacy":saved.legacy,"loss":metrics["loss"],"parent_id":metrics["parent_id"],"checkpoint_path":format!("data/lab/{id}/checkpoint.json")})),
            Err(_)=>invalid+=1,
        }
    }
    Ok(
        json!({"models":models,"invalid":invalid,"truncated":truncated,"chat_model":{"id":"qwen-local","name":"Qwen2.5-Coder 7B","purpose":"Chat e investigacion","availability":"No comprobada por esta consulta"}}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn value() -> Value {
        json!({"schema":2,"architecture":"one_hot_tanh_softmax_mlp","context":8,"hidden":32,"vocab":96,"tokenizer":"newline=0; printable ASCII 32..126=1..95","weights":vec![0.0;CONFIG.parameters()],"sampler_state":42,"completed_steps":100})
    }
    #[test]
    fn rejects_paths_and_incompatible_weights() {
        for id in [
            "../tiny-1",
            "tiny-1/other",
            "tiny-",
            "tiny-abc",
            "C:\\tiny-1",
        ] {
            assert!(!valid_id(id));
        }
        let mut v = value();
        assert!(parse(&v).is_ok());
        v["hidden"] = json!(64);
        assert!(parse(&v).is_err());
        v = value();
        v["weights"][0] = json!(1e100);
        assert!(parse(&v).is_err());
        v = value();
        v["schema"] = json!(3);
        assert!(parse(&v).is_err());
        v = value();
        v["weights"] = json!([0.0]);
        assert!(parse(&v).is_err());
    }
    #[test]
    fn old_checkpoints_do_not_invent_sampler_history() {
        let mut v = value();
        v["schema"] = json!(1);
        let s = parse(&v).unwrap();
        assert!(s.legacy);
        assert_eq!(s.completed, None);
        assert_eq!(s.sampler, 42);
    }
}
