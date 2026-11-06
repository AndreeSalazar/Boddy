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
fn bytes(root: &Path, relative: &str, limit: u64) -> Result<Vec<u8>, String> {
    boddy_reader::plain_path(root, Path::new(relative))?;
    let mut bytes = Vec::new();
    fs::File::open(root.join(relative))
        .map_err(|e| e.to_string())?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("Archivo de modelo demasiado grande".into());
    }
    Ok(bytes)
}
pub(crate) fn read(root: &Path, id: &str, file: &str, limit: u64) -> Result<Value, String> {
    if !valid_id(id) {
        return Err("Identificador Tiny invalido".into());
    }
    serde_json::from_slice(&bytes(root, &format!("data/lab/{id}/{file}"), limit)?)
        .map_err(|e| e.to_string())
}
fn parse(value: &Value, binary: Option<Vec<f32>>) -> Result<Saved, String> {
    let schema = value["schema"]
        .as_u64()
        .ok_or("Falta version de checkpoint")?;
    if !matches!(schema, 1..=3)
        || value["architecture"] != "one_hot_tanh_softmax_mlp"
        || value["context"] != CONFIG.context
        || value["hidden"] != CONFIG.hidden
        || value["vocab"] != CONFIG.vocab
        || value["tokenizer"] != "newline=0; printable ASCII 32..126=1..95"
    {
        return Err("Arquitectura o tokenizer no compatibles con Tiny 1.0".into());
    }
    let weights: Vec<_> = if schema == 3 {
        binary.ok_or("Faltan pesos binarios")?
    } else {
        let raw = value["weights"]
            .as_array()
            .filter(|v| v.len() == CONFIG.parameters())
            .ok_or("Forma de pesos incorrecta")?;
        raw.iter()
            .map(|v| {
                v.as_f64()
                    .map(|v| v as f32)
                    .filter(|v| v.is_finite())
                    .ok_or("Peso no finito o invalido")
            })
            .collect::<Result<_, _>>()?
    };
    if weights.len() != CONFIG.parameters() || weights.iter().any(|w| !w.is_finite()) {
        return Err("Pesos binarios incompatibles".into());
    }
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
    let binary = if value["schema"] == 3 {
        Some(load_blob(root, &value["weights"])?)
    } else {
        None
    };
    let saved = parse(&value, binary)?;
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

fn blob_path(descriptor: &Value) -> Result<String, String> {
    let hash = descriptor["sha256"].as_str().ok_or("Falta hash de pesos")?;
    if descriptor["encoding"] != "boddy-f32le-v1"
        || descriptor["count"] != CONFIG.parameters()
        || descriptor["bytes"] != CONFIG.parameters() * 4
        || hash.len() != 64
        || !hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("Descriptor binario invalido".into());
    }
    Ok(format!("data/lab/blobs/{hash}.f32le"))
}
fn decode_blob(descriptor: &Value, raw: &[u8]) -> Result<Vec<f32>, String> {
    blob_path(descriptor)?;
    if raw.len() != CONFIG.parameters() * 4
        || descriptor["sha256"] != format!("{:x}", Sha256::digest(raw))
    {
        return Err("Pesos truncados o SHA256 incorrecto".into());
    }
    let weights: Vec<_> = raw
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    if weights.iter().any(|v| !v.is_finite()) {
        return Err("Pesos binarios no finitos".into());
    }
    Ok(weights)
}
fn load_blob(root: &Path, descriptor: &Value) -> Result<Vec<f32>, String> {
    let path = blob_path(descriptor)?;
    decode_blob(
        descriptor,
        &bytes(root, &path, (CONFIG.parameters() * 4) as u64)?,
    )
}
pub(crate) fn store(
    root: &Path,
    id: &str,
    mut metadata: Value,
    weights: &[f32],
) -> Result<Value, String> {
    if !valid_id(id)
        || weights.len() != CONFIG.parameters()
        || weights.iter().any(|v| !v.is_finite())
    {
        return Err("Modelo incompatible para guardar".into());
    }
    let raw: Vec<u8> = weights.iter().flat_map(|v| v.to_le_bytes()).collect();
    let descriptor = json!({"encoding":"boddy-f32le-v1","count":weights.len(),"bytes":raw.len(),"sha256":format!("{:x}",Sha256::digest(&raw))});
    let relative = blob_path(&descriptor)?;
    boddy_reader::plain_path(root, Path::new(&relative))?;
    fs::create_dir_all(root.join("data/lab/blobs")).map_err(|e| e.to_string())?;
    let reused = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(&relative))
    {
        Ok(mut file) => {
            use std::io::Write;
            file.write_all(&raw).map_err(|e| e.to_string())?;
            false
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            if bytes(root, &relative, raw.len() as u64)? != raw {
                return Err("Blob existente incompatible; no se sobrescribe".into());
            }
            true
        }
        Err(e) => return Err(e.to_string()),
    };
    metadata["schema"] = json!(3);
    metadata["weights"] = descriptor;
    let encoded = serde_json::to_vec(&metadata).map_err(|e| e.to_string())?;
    let relative_manifest = format!("data/lab/{id}/checkpoint.json");
    boddy_reader::plain_path(root, Path::new(&relative_manifest))?;
    super::write_new(&root.join(&relative_manifest), &encoded)?;
    // Lectura real: manifiesto + blob deben restaurar cada bit de los pesos.
    let restored = load(root, id, false)?;
    if restored
        .weights
        .iter()
        .zip(weights)
        .any(|(a, b)| a.to_bits() != b.to_bits())
    {
        return Err("La restauracion binaria cambio los pesos".into());
    }
    Ok(
        json!({"format":"boddy-checkpoint-v3","manifest_bytes":encoded.len(),"weights_bytes":raw.len(),"weights_path":relative,"blob_reused":reused,"new_weight_bytes":if reused {0}else{raw.len()},"precision":"f32 sin cuantizacion; restauracion exacta","note":"Bytes de contenido; excluye asignacion del sistema de archivos. Otros modelos/versiones tambien ocupan disco."}),
    )
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
        assert!(parse(&v, None).is_ok());
        v["hidden"] = json!(64);
        assert!(parse(&v, None).is_err());
        v = value();
        v["weights"][0] = json!(1e100);
        assert!(parse(&v, None).is_err());
        v = value();
        v["schema"] = json!(4);
        assert!(parse(&v, None).is_err());
        v = value();
        v["weights"] = json!([0.0]);
        assert!(parse(&v, None).is_err());
    }
    #[test]
    fn old_checkpoints_do_not_invent_sampler_history() {
        let mut v = value();
        v["schema"] = json!(1);
        let s = parse(&v, None).unwrap();
        assert!(s.legacy);
        assert_eq!(s.completed, None);
        assert_eq!(s.sampler, 42);
    }
    #[test]
    fn binary_preserves_float_bits_and_rejects_corruption() {
        let mut weights = super::super::initialize(CONFIG, 1337).unwrap();
        weights[0] = -0.0;
        let raw: Vec<u8> = weights.iter().flat_map(|v| v.to_le_bytes()).collect();
        let d = json!({"encoding":"boddy-f32le-v1","count":weights.len(),"bytes":raw.len(),"sha256":format!("{:x}",Sha256::digest(&raw))});
        let loaded = decode_blob(&d, &raw).unwrap();
        assert!(weights
            .iter()
            .zip(loaded)
            .all(|(a, b)| a.to_bits() == b.to_bits()));
        let mut broken = raw.clone();
        broken[10] ^= 1;
        assert!(decode_blob(&d, &broken).is_err());
        assert!(decode_blob(&d, &raw[..raw.len() - 1]).is_err());
        let mut escaped = d.clone();
        escaped["sha256"] = json!("../checkpoint");
        assert!(blob_path(&escaped).is_err());
        let mut bad = d.clone();
        bad["count"] = json!(1);
        assert!(decode_blob(&bad, &raw).is_err());
    }
    #[test]
    fn verified_binary_still_rejects_nan() {
        let raw: Vec<u8> = vec![f32::NAN; CONFIG.parameters()]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let d = json!({"encoding":"boddy-f32le-v1","count":CONFIG.parameters(),"bytes":raw.len(),"sha256":format!("{:x}",Sha256::digest(&raw))});
        assert!(decode_blob(&d, &raw).is_err());
    }
}
