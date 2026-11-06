//! Host del Transformer experimental. No cambia el protocolo ni pesos Tiny 1.0.
use boddy_neural::Rng;
use boddy_transformer::{encode, initialize, sgd, Config, Graph};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::Path,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
const C: Config = Config {
    context: 16,
    dimension: 16,
    feed_forward: 32,
};
const CORPUS: &str = include_str!("../../../training/corpora/boddy-attention-v1.txt");
const NAME: &str = "Boddy Attention 0.1";
fn valid(id: &str) -> bool {
    id.strip_prefix("attn-")
        .is_some_and(|s| !s.is_empty() && s.len() <= 30 && s.bytes().all(|b| b.is_ascii_digit()))
}
fn read(root: &Path, id: &str, file: &str, limit: u64) -> Result<Vec<u8>, String> {
    if !valid(id) {
        return Err("Identificador Attention invalido".into());
    }
    let relative = format!("data/attention/{id}/{file}");
    boddy_reader::plain_path(root, Path::new(&relative))?;
    let mut bytes = Vec::new();
    fs::File::open(root.join(relative))
        .map_err(|e| e.to_string())?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("Archivo Attention demasiado grande".into());
    }
    Ok(bytes)
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn load(root: &Path, id: &str) -> Result<(Value, Vec<f32>), String> {
    let m: Value = serde_json::from_slice(&read(root, id, "checkpoint.json", 4096)?)
        .map_err(|e| e.to_string())?;
    if m["schema"] != 1
        || m["architecture"] != "boddy-causal-byte-transformer-v1"
        || m["context"] != C.context
        || m["dimension"] != C.dimension
        || m["feed_forward"] != C.feed_forward
        || m["tokenizer"] != "utf8-bytes-256-v1"
        || m["weights_bytes"] != C.parameters() * 4
    {
        return Err("Modelo Attention incompatible".into());
    }
    let bytes = read(root, id, "weights.f32le", (C.parameters() * 4) as u64)?;
    if bytes.len() != C.parameters() * 4 || m["weights_sha256"] != hash(&bytes) {
        return Err("Pesos Attention corruptos".into());
    }
    let w: Vec<_> = bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    if w.iter().any(|v| !v.is_finite()) {
        return Err("Pesos Attention no finitos".into());
    }
    Ok((m, w))
}
fn evaluate(graph: &Graph, w: &[f32], tokens: &[u32]) -> Result<Value, String> {
    let count = (tokens.len() - C.context).min(64);
    let mut loss = 0.0;
    let mut correct = 0;
    for i in 0..count {
        let p = i * (tokens.len() - C.context) / count;
        let e = graph
            .evaluate(w, &tokens[p..p + C.context], tokens[p + C.context], false)
            .map_err(str::to_string)?;
        loss += e.loss;
        let best = e
            .probabilities
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .ok_or("Probabilidades ausentes")?
            .0;
        if best == tokens[p + C.context] as usize {
            correct += 1;
        }
    }
    Ok(
        json!({"loss":loss/count as f32,"next_byte_accuracy":correct as f32/count as f32,"contexts":count}),
    )
}
fn utf8_allowed(bytes: &[u8], b: u8) -> bool {
    match std::str::from_utf8(bytes) {
        Ok(_) => matches!(b,9|10|32..=126|194..=244),
        Err(e) => {
            let tail = &bytes[e.valid_up_to()..];
            if e.error_len().is_some() || tail.is_empty() {
                return false;
            }
            if tail.len() > 1 {
                return (128..=191).contains(&b);
            }
            match tail[0] {
                224 => (160..=191).contains(&b),
                237 => (128..=159).contains(&b),
                240 => (144..=191).contains(&b),
                244 => (128..=143).contains(&b),
                _ => (128..=191).contains(&b),
            }
        }
    }
}
fn generate(w: &[f32], prefix: &str, count: usize) -> Result<String, String> {
    if prefix.len() > 512 || !(1..=128).contains(&count) {
        return Err("Attention: prefijo hasta 512 bytes; salida 1..128 bytes".into());
    }
    let graph = Graph::build(C).map_err(str::to_string)?;
    let mut bytes = prefix.as_bytes().to_vec();
    let mut tokens = vec![32; C.context.saturating_sub(bytes.len())];
    tokens.extend(encode(prefix));
    for _ in 0..count {
        let e = graph
            .evaluate(w, &tokens[tokens.len() - C.context..], 0, false)
            .map_err(str::to_string)?;
        let best = e
            .probabilities
            .iter()
            .enumerate()
            .filter(|(i, _)| utf8_allowed(&bytes, *i as u8))
            .max_by(|a, b| a.1.total_cmp(b.1))
            .ok_or("No hay continuacion UTF8")?
            .0;
        bytes.push(best as u8);
        tokens.push(best as u32);
    }
    let length = match std::str::from_utf8(&bytes) {
        Ok(_) => bytes.len(),
        Err(e) if e.error_len().is_none() => e.valid_up_to(),
        Err(_) => return Err("Generacion UTF8 invalida".into()),
    };
    bytes.truncate(length);
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
pub fn train(root: &Path, steps: u64, parent: Option<&str>) -> Result<Value, String> {
    let profile = if let Some(id) = parent {
        let (m, _) = load(root, id)?;
        m["training_profile"]
            .as_str()
            .unwrap_or("general-v1")
            .to_string()
    } else {
        "general-v1".to_string()
    };
    train_profile(root, steps, parent, &profile)
}
pub fn train_profile(
    root: &Path,
    steps: u64,
    parent: Option<&str>,
    profile: &str,
) -> Result<Value, String> {
    if !(1..=1000).contains(&steps) {
        return Err("Attention: 1 a 1000 pasos".into());
    }
    let started = Instant::now();
    let (training_text, validation_text, corpus_hash) = match profile {
        "general-v1" => {
            let lines: Vec<_> = CORPUS.split_inclusive('\n').collect();
            let cut = lines.len() * 4 / 5;
            (
                lines[..cut].concat(),
                lines[cut..].concat(),
                hash(CORPUS.as_bytes()),
            )
        }
        "rust-v1" => (
            boddy_rust_tasks::TRAIN.to_string(),
            boddy_rust_tasks::VALIDATION.to_string(),
            hash(boddy_rust_tasks::TRAIN.as_bytes()),
        ),
        _ => return Err("Perfil de aprendizaje desconocido".into()),
    };
    let training = encode(&training_text);
    let validation = encode(&validation_text);
    let graph = Graph::build(C).map_err(str::to_string)?;
    let mut sampler = Rng(42);
    let mut completed = 0;
    let mut resume_kind = "fresh";
    let initial = if let Some(id) = parent {
        let (m, w) = load(root, id)?;
        let same_profile = m["training_profile"].as_str().unwrap_or("general-v1") == profile;
        if (same_profile && m["corpus_sha256"] != corpus_hash)
            || m["training_protocol"] != "attention-sgd-clip1-v1"
        {
            return Err("Corpus o protocolo Attention incompatible".into());
        }
        if same_profile {
            sampler.0 = m["sampler_state"]
                .as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .ok_or("Estado de muestreo invalido")?;
            resume_kind = "full_state";
        } else {
            resume_kind = "transfer_profile";
        }
        completed = m["completed_steps"].as_u64().ok_or("Pasos invalidos")?;
        w
    } else {
        initialize(C, 1337).map_err(str::to_string)?
    };
    let total = completed.checked_add(steps).ok_or("Pasos fuera de rango")?;
    let before_train = evaluate(&graph, &initial, &training)?;
    let before_valid = evaluate(&graph, &initial, &validation)?;
    let start = sampler.next_u32() as usize % (training.len() - C.context);
    let input = &training[start..start + C.context];
    let target = training[start + C.context];
    let reference = graph
        .evaluate(&initial, input, target, true)
        .map_err(str::to_string)?;
    let nodes = graph.instructions().len();
    let gpu =
        boddy_wgpu::transformer::Trainer::new(Graph::build(C).map_err(str::to_string)?, &initial)?;
    let (gpu_loss, gpu_gradient) = gpu.probe(input, target)?;
    let gradient_error = reference
        .gradient
        .iter()
        .zip(&gpu_gradient)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f32::max);
    let loss_error = (reference.loss - gpu_loss).abs();
    if gradient_error > 0.001 || loss_error > 0.0001 {
        return Err(format!(
            "Paridad Attention fallida: loss={loss_error}, gradient={gradient_error}"
        ));
    }
    let mut expected = initial.clone();
    sgd(&mut expected, &reference.gradient, 0.03).map_err(str::to_string)?;
    gpu.step(input, target, 0.03)?;
    let first = gpu.weights()?;
    let update_error = first
        .iter()
        .zip(&expected)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f32::max);
    if update_error > 0.0001 {
        return Err("SGD Attention no coincide con CPU".into());
    }
    let mut curve = vec![json!({"step":0,"train_loss":before_train["loss"]})];
    for step in 1..=steps {
        if step > 1 {
            let p = sampler.next_u32() as usize % (training.len() - C.context);
            gpu.step(&training[p..p + C.context], training[p + C.context], 0.03)?;
        }
        if step % 100 == 0 || step == steps {
            let w = gpu.weights()?;
            curve.push(json!({"step":step,"train_loss":evaluate(&graph,&w,&training)?["loss"]}));
        }
    }
    let w = gpu.weights()?;
    let after_train = evaluate(&graph, &w, &training)?;
    let after_valid = evaluate(&graph, &w, &validation)?;
    let sample_prefix = if profile == "rust-v1" {
        "fn completar(a: i32, b: i32) -> i32 {\n// objetivo: suma\n"
    } else {
        "Boddy: "
    };
    let text = generate(&w, sample_prefix, 96)?;
    let id = format!(
        "attn-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
    );
    let relative = format!("data/attention/{id}");
    boddy_reader::plain_path(root, Path::new(&relative))?;
    let dir = root.join(&relative);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let bytes: Vec<u8> = w.iter().flat_map(|v| v.to_le_bytes()).collect();
    let manifest = json!({"schema":1,"model":NAME,"architecture":"boddy-causal-byte-transformer-v1","context":C.context,"dimension":C.dimension,"feed_forward":C.feed_forward,"tokenizer":"utf8-bytes-256-v1","weights_bytes":bytes.len(),"weights_sha256":hash(&bytes),"seed":1337,"sampler_state":sampler.0,"completed_steps":total,"parent_id":parent,"corpus_sha256":corpus_hash,"validation_sha256":hash(validation_text.as_bytes()),"training_profile":profile,"training_protocol":"attention-sgd-clip1-v1","pretrained_weights":null});
    let metadata = serde_json::to_vec(&manifest).map_err(|e| e.to_string())?;
    super::write_new(&dir.join("weights.f32le"), &bytes)?;
    super::write_new(&dir.join("checkpoint.json"), &metadata)?;
    let (_, restored) = load(root, &id)?;
    if restored
        .iter()
        .zip(&w)
        .any(|(a, b)| a.to_bits() != b.to_bits())
        || generate(&restored, sample_prefix, 96)? != text
    {
        return Err("Checkpoint Attention no reproduce pesos o muestra".into());
    }
    let report = json!({"schema":1,"id":id,"model":NAME,"architecture":"one_block_causal_transformer","parameters":C.parameters(),"context_tokens":C.context,"embedding_dimension":C.dimension,"feed_forward":C.feed_forward,"attention_heads":1,"tokenizer":"UTF8 bytes: 256 tokens","device":gpu.device_name,"backend":"WGPU/Vulkan scalar tape reference","nodes":nodes,"tensor_buffer_bytes":gpu.tensor_bytes,"steps":steps,"completed_steps":total,"parent_id":parent,"learning_rate":0.03,"gradient_clip_norm":1.0,"loss":{"train_before":before_train["loss"],"train_after":after_train["loss"],"validation_before":before_valid["loss"],"validation_after":after_valid["loss"]},"validation":{"before":before_valid,"after":after_valid},"validation_improved":after_valid["loss"].as_f64()<before_valid["loss"].as_f64(),"parity":{"loss_max_abs":loss_error,"gradient_max_abs":gradient_error,"sgd_max_abs":update_error},"curve":curve,"sample":text,"sample_backend":"CPU reference; greedy with UTF8 validity mask","elapsed_ms":started.elapsed().as_millis(),"checkpoint_roundtrip":true,"checkpoint_path":format!("{relative}/checkpoint.json"),"storage":{"manifest_bytes":metadata.len(),"weights_bytes":bytes.len(),"new_weight_bytes":bytes.len(),"blob_reused":false},"corpus_sha256":hash(CORPUS.as_bytes()),"provenance":{"pretrained_weights":null,"teacher_model":null},"limitations":["Experimental: no chat ni herramientas","16 bytes de contexto, no 16 caracteres","Una cabeza; FFN tanh; sin RoPE ni SwiGLU","Interprete escalar GPU: no se afirma mejora de rendimiento","Validacion pequena y relacionada; no evalua razonamiento","Formato separado de Tiny 1.0; pesos no intercambiables"]});
    let mut report = report;
    report["corpus_sha256"] = json!(corpus_hash);
    report["validation_sha256"] = json!(hash(validation_text.as_bytes()));
    report["training_profile"] = json!(profile);
    report["resume_kind"] = json!(resume_kind);
    super::write_new(
        &dir.join("metrics.json"),
        &serde_json::to_vec(&report).map_err(|e| e.to_string())?,
    )?;
    Ok(report)
}
pub fn command(root: &Path, request: &Value) -> Result<Value, String> {
    match request["action"].as_str().ok_or("Falta accion Attention")? {
        "train" => {
            let parent = match request.get("parent_id") {
                None | Some(Value::Null) => None,
                Some(v) => Some(v.as_str().ok_or("Padre invalido")?),
            };
            let steps = request["steps"].as_u64().ok_or("Falta steps")?;
            match request.get("profile") {
                None => train(root, steps, parent),
                Some(v) => {
                    train_profile(root, steps, parent, v.as_str().ok_or("Perfil no textual")?)
                }
            }
        }
        "generate" => {
            let id = request["id"].as_str().ok_or("Falta identificador")?;
            let (_, w) = load(root, id)?;
            let count = request["max_tokens"]
                .as_u64()
                .filter(|n| (1..=128).contains(n))
                .ok_or("Attention: 1..128 bytes")? as usize;
            let prefix = request["prefix"].as_str().ok_or("Falta prefijo")?;
            let content = generate(&w, prefix, count)?;
            let completed = content.len() - prefix.len();
            Ok(
                json!({"content":content,"model":NAME,"id":id,"context_tokens":C.context,"completion_tokens":completed,"sampled_tokens":count,"discarded_utf8_tail_bytes":count-completed,"backend":"CPU reference"}),
            )
        }
        "inspect" => serde_json::from_slice(&read(
            root,
            request["id"].as_str().ok_or("Falta identificador")?,
            "metrics.json",
            65536,
        )?)
        .map_err(|e| e.to_string()),
        "list" => {
            boddy_reader::plain_path(root, Path::new("data/attention"))?;
            let mut models = Vec::new();
            let mut invalid = 0;
            let mut truncated = false;
            match fs::read_dir(root.join("data/attention")) {
                Ok(entries) => {
                    let mut ids = Vec::new();
                    for (i, entry) in entries.take(513).enumerate() {
                        if i == 512 {
                            truncated = true;
                            break;
                        }
                        let entry = entry.map_err(|e| e.to_string())?;
                        let id = entry.file_name().to_string_lossy().into_owned();
                        if valid(&id) {
                            ids.push(id);
                        }
                    }
                    ids.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| b.cmp(a)));
                    truncated |= ids.len() > 64;
                    for id in ids.iter().take(64) {
                        if let Ok((m, _)) = load(root, id) {
                            models.push(json!({"id":id,"model":NAME,"profile":m["training_profile"].as_str().unwrap_or("general-v1")}));
                        } else {
                            invalid += 1;
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.to_string()),
            }
            Ok(json!({"models":models,"invalid":invalid,"truncated":truncated}))
        }
        _ => Err("Accion Attention no permitida".into()),
    }
}

/// Puntua continuaciones cerradas; no recibe los casos ni respuestas del verificador.
pub struct Ranker {
    graph: Graph,
    weights: Vec<f32>,
    metadata: Value,
}
impl Ranker {
    pub fn load(root: &Path, id: &str) -> Result<Self, String> {
        let (metadata, weights) = load(root, id)?;
        Ok(Self {
            graph: Graph::build(C).map_err(str::to_string)?,
            weights,
            metadata,
        })
    }
    pub fn provenance(&self) -> Value {
        self.metadata.clone()
    }
    pub fn scores(&self, prefix: &str, candidates: &[&str]) -> Result<Vec<f32>, String> {
        if prefix.len() > 512
            || candidates.is_empty()
            || candidates.len() > 16
            || candidates.iter().any(|s| s.is_empty() || s.len() > 64)
        {
            return Err("Ranking fuera de limites".into());
        }
        let mut context = vec![32; C.context.saturating_sub(prefix.len())];
        context.extend(encode(prefix));
        candidates
            .iter()
            .map(|text| {
                let mut tokens = context.clone();
                let mut loss = 0.0;
                for b in text.bytes() {
                    loss += self
                        .graph
                        .evaluate(
                            &self.weights,
                            &tokens[tokens.len() - C.context..],
                            u32::from(b),
                            false,
                        )
                        .map_err(str::to_string)?
                        .loss;
                    tokens.push(u32::from(b));
                }
                Ok(loss / text.len() as f32)
            })
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn utf8_mask_rejects_overlong_and_surrogates() {
        assert!(!utf8_allowed(&[224], 128));
        assert!(!utf8_allowed(&[237], 160));
        assert!(!utf8_allowed(&[244], 144));
        assert!(utf8_allowed(&[240, 159], 140));
        assert!(!valid("attn-1/../other"));
    }
    #[test]
    fn generation_preserves_unicode_and_bounds() {
        let w = initialize(C, 1337).unwrap();
        let text = generate(&w, "¿Qué tal? 🧠", 8).unwrap();
        assert!(text.starts_with("¿Qué tal? 🧠"));
        assert!(generate(&w, "hola", 129).is_err());
        assert!(train(Path::new("."), 0, None).is_err());
    }
    #[test]
    fn ranking_and_profiles_are_bounded() {
        assert!(train_profile(Path::new("."), 1, None, "unknown").is_err());
        let ranker = Ranker {
            graph: Graph::build(C).unwrap(),
            weights: initialize(C, 1337).unwrap(),
            metadata: json!({}),
        };
        assert!(ranker.scores("", &[]).is_err());
        assert!(ranker.scores("", &[""]).is_err());
        let scores = ranker.scores("// suma\n", &["a + b", "a - b"]).unwrap();
        assert_eq!(scores.len(), 2);
        assert!(scores.iter().all(|s| s.is_finite()));
    }
}
