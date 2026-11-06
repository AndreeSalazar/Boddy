//! Experimentos acotados con pesos propios. No interviene en el chat Qwen.
use boddy_neural::{forward, initialize, loss_and_gradient, Config, Rng};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::Path,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

const CORPUS: &str = include_str!("../../../training/corpora/boddy-tiny-v1.txt");
const CONFIG: Config = Config {
    context: 8,
    vocab: 96,
    hidden: 32,
};
const BATCH: usize = 32;
pub mod attention;
pub mod checkpoints;
const CONTROL: &str = include_str!("../../../training/evals/tiny-language-v1.txt");

fn encode(text: &str) -> Result<Vec<u32>, String> {
    text.bytes()
        .map(|b| match b {
            b'\n' => Ok(0),
            32..=126 => Ok((b - 31) as u32),
            _ => Err("Boddy Tiny v1 solo acepta ASCII imprimible y salto de linea".into()),
        })
        .collect()
}
fn examples(tokens: &[u32], starts: &[usize]) -> (Vec<u32>, Vec<u32>) {
    let mut inputs = Vec::with_capacity(starts.len() * CONFIG.context);
    let mut targets = Vec::with_capacity(starts.len());
    for start in starts {
        inputs.extend_from_slice(&tokens[*start..*start + CONFIG.context]);
        targets.push(tokens[*start + CONFIG.context]);
    }
    (inputs, targets)
}
fn eval(weights: &[f32], tokens: &[u32]) -> Result<f32, String> {
    let count = tokens.len() - CONFIG.context;
    let starts: Vec<_> = (0..count.min(256))
        .map(|i| i * count / count.min(256))
        .collect();
    let mut total = 0.0;
    for start in starts.iter() {
        let (_, p) = forward(CONFIG, weights, &tokens[*start..*start + CONFIG.context])
            .map_err(str::to_string)?;
        total -= p[tokens[*start + CONFIG.context] as usize].max(1e-9).ln();
    }
    Ok(total / starts.len() as f32)
}
fn sample(weights: &[f32]) -> Result<String, String> {
    sample_from(weights, "  boddy ", 256)
}
fn sample_from(weights: &[f32], prefix: &str, count: usize) -> Result<String, String> {
    if prefix.len() > 256 || !(1..=256).contains(&count) {
        return Err("Prefijo maximo 256 bytes; salida entre 1 y 256 caracteres".into());
    }
    let mut tokens = encode(prefix)?;
    if tokens.len() < CONFIG.context {
        let mut padded = vec![1; CONFIG.context - tokens.len()];
        padded.extend(tokens);
        tokens = padded;
    }
    let mut rng = Rng(2026);
    for _ in 0..count {
        let (_, p) = forward(CONFIG, weights, &tokens[tokens.len() - CONFIG.context..])
            .map_err(str::to_string)?;
        let mut draw = rng.unit();
        let mut next = 95;
        for (i, value) in p.iter().enumerate() {
            draw -= value;
            if draw <= 0.0 {
                next = i as u32;
                break;
            }
        }
        tokens.push(next);
    }
    Ok(tokens
        .into_iter()
        .map(|t| if t == 0 { '\n' } else { (t as u8 + 31) as char })
        .collect())
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    f.write_all(bytes).map_err(|e| e.to_string())
}

pub fn train(root: &Path, steps: u64) -> Result<Value, String> {
    train_from(root, steps, None)
}
pub fn train_from(root: &Path, steps: u64, parent: Option<&str>) -> Result<Value, String> {
    if !(1..=2000).contains(&steps) {
        return Err("Pasos permitidos: 1 a 2000".into());
    }
    let started = Instant::now();
    // Separacion por lineas antes de construir ventanas: ningun contexto cruza el corte.
    let lines: Vec<_> = CORPUS.split_inclusive('\n').collect();
    let split = lines.len() * 4 / 5;
    let train_tokens = encode(&lines[..split].concat())?;
    let valid_tokens = encode(&lines[split..].concat())?;
    let saved = parent
        .map(|id| checkpoints::load(root, id, true))
        .transpose()?;
    let sampler = saved.as_ref().map_or(42, |s| s.sampler);
    let previous_steps = saved.as_ref().map_or(Some(0), |s| s.completed);
    let legacy = saved.as_ref().is_some_and(|s| s.legacy);
    let initial = match saved {
        Some(s) => s.weights,
        None => initialize(CONFIG, 1337).map_err(str::to_string)?,
    };
    let before_train = eval(&initial, &train_tokens)?;
    let before_valid = eval(&initial, &valid_tokens)?;
    let gpu = boddy_wgpu::Trainer::new(CONFIG, &initial, BATCH)?;
    let mut rng = Rng(sampler);
    let mut batch = || {
        let starts: Vec<_> = (0..BATCH)
            .map(|_| (rng.next_u32() as usize) % (train_tokens.len() - CONFIG.context))
            .collect();
        examples(&train_tokens, &starts)
    };
    let (inputs, targets) = batch();
    let (cpu_loss, cpu_grad) =
        loss_and_gradient(CONFIG, &initial, &inputs, &targets).map_err(str::to_string)?;
    let (gpu_loss, gpu_grad) = gpu.probe(&inputs, &targets)?;
    let grad_error = cpu_grad
        .iter()
        .zip(&gpu_grad)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f32::max);
    let loss_error = (cpu_loss - gpu_loss).abs();
    if loss_error > 0.0001 || grad_error > 0.0005 {
        return Err(format!(
            "Paridad CPU/GPU fallida: perdida {loss_error}, gradiente {grad_error}"
        ));
    }
    // Verifica tambien la actualizacion SGD real antes de continuar.
    gpu.step(&inputs, &targets, 0.2)?;
    let updated = gpu.weights()?;
    let update_error = updated
        .iter()
        .zip(initial.iter().zip(&cpu_grad))
        .map(|(v, (w, g))| (v - (w - 0.2 * g)).abs())
        .fold(0.0, f32::max);
    if update_error > 0.0005 {
        return Err(format!("Actualizacion GPU incorrecta: {update_error}"));
    }
    let mut curve = vec![json!({"step":0,"train_loss":before_train})];
    for step in 1..=steps {
        if step > 1 {
            let (x, y) = batch();
            gpu.step(&x, &y, 0.2)?;
        }
        if step % 100 == 0 || step == steps {
            let weights = gpu.weights()?;
            curve.push(json!({"step":step,"train_loss":eval(&weights,&train_tokens)?}));
        }
    }
    let weights = gpu.weights()?;
    let after_train = eval(&weights, &train_tokens)?;
    let after_valid = eval(&weights, &valid_tokens)?;
    let total_steps = previous_steps
        .map(|v| {
            v.checked_add(steps)
                .ok_or("Contador acumulado fuera de rango")
        })
        .transpose()?;
    let text = sample(&weights)?;
    let corpus_hash = format!("{:x}", Sha256::digest(CORPUS.as_bytes()));
    let checkpoint = json!({"schema":3,"model":"Boddy Tiny 1.0","architecture":"one_hot_tanh_softmax_mlp","context":CONFIG.context,"vocab":CONFIG.vocab,"hidden":CONFIG.hidden,"tokenizer":"newline=0; printable ASCII 32..126=1..95","seed":1337,"pretrained_weights":null,"teacher_model":null,"sampler_state":rng.0,"completed_steps":total_steps,"parent_id":parent,"corpus_sha256":corpus_hash,"training_protocol":"tiny-sgd-v1"});
    let id = format!(
        "tiny-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
    );
    let relative = format!("data/lab/{id}");
    boddy_reader::plain_path(root, Path::new(&relative))?;
    let dir = root.join(&relative);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let report = json!({"schema":1,"id":id,"model":"Boddy Tiny 1.0","purpose":"Experimento educativo de siguiente caracter; no es asistente general","device":gpu.device_name,"backend":"WGPU/Vulkan","parameters":CONFIG.parameters(),"context_tokens":CONFIG.context,"vocab_tokens":CONFIG.vocab,"batch_tokens":BATCH*CONFIG.context,"training_target_tokens":steps*BATCH as u64,"tensor_buffer_bytes":gpu.tensor_bytes,"weight_f32_bytes":CONFIG.parameters()*4,"memory_note":"Buffers declarados; excluye driver, compilador, staging y otros procesos. No es VRAM total ni pico.","steps":steps,"learning_rate":0.2,"seed":1337,"corpus_sha256":format!("{:x}",Sha256::digest(CORPUS.as_bytes())),"corpus_bytes":CORPUS.len(),"train_characters":train_tokens.len(),"validation_characters":valid_tokens.len(),"loss":{"train_before":before_train,"train_after":after_train,"validation_before":before_valid,"validation_after":after_valid},"parity":{"loss_max_abs":loss_error,"gradient_max_abs":grad_error,"sgd_max_abs":update_error},"checkpoint_roundtrip":true,"elapsed_ms":started.elapsed().as_millis(),"curve":curve,"sample":text,"sample_backend":"CPU reference","checkpoint_path":format!("{relative}/checkpoint.json"),"metrics_path":format!("{relative}/metrics.json"),"provenance":{"initialization":"seeded_random","pretrained_weights":null,"teacher_model":null},"limitations":["ASCII solamente","Contexto de ocho caracteres","Corpus pequeno y validacion relacionada: no mide capacidades generales","Sin aprendizaje automatico de conversaciones"]});
    let mut report = report;
    report["parent_id"] = json!(parent);
    report["completed_steps"] = json!(total_steps);
    report["resume_kind"] = json!(if parent.is_none() {
        "fresh"
    } else if legacy {
        "weights_only_legacy"
    } else {
        "full_state"
    });
    report["validation_improved"] = json!(after_valid < before_valid);
    report["external_control"] = control(&initial, &weights)?;
    report["storage"] = checkpoints::store(root, &id, checkpoint, &weights)?;
    if sample(&checkpoints::load(root, &id, false)?.weights)? != text {
        return Err("Checkpoint no reproduce la muestra".into());
    }
    write_new(&dir.join("sample.txt"), text.as_bytes())?;
    write_new(
        &dir.join("metrics.json"),
        &serde_json::to_vec(&report).map_err(|e| e.to_string())?,
    )?;
    Ok(report)
}

fn quality(weights: &[f32], tokens: &[u32]) -> Result<Value, String> {
    let mut hits = 0;
    let count = (tokens.len() - CONFIG.context).min(256);
    for i in 0..count {
        let start = i * (tokens.len() - CONFIG.context) / count;
        let (_, p) = forward(CONFIG, weights, &tokens[start..start + CONFIG.context])
            .map_err(str::to_string)?;
        let best = p
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .ok_or("Salida vacia")?
            .0;
        if best == tokens[start + CONFIG.context] as usize {
            hits += 1;
        }
    }
    Ok(
        json!({"loss":eval(weights,tokens)?,"next_character_accuracy":hits as f32/count as f32,"contexts":count}),
    )
}
fn control(before: &[f32], after: &[f32]) -> Result<Value, String> {
    let tokens = encode(CONTROL)?;
    Ok(
        json!({"dataset":"tiny-language-v1","sha256":format!("{:x}",Sha256::digest(CONTROL.as_bytes())),"before":quality(before,&tokens)?,"after":quality(after,&tokens)?,"note":"Frases de control nunca usadas en SGD; no mide razonamiento ni precision de respuestas"}),
    )
}
pub fn model_command(root: &Path, request: &Value) -> Result<Value, String> {
    match request["action"].as_str().ok_or("Falta accion de modelo")? {
        "list" => checkpoints::list(root),
        "inspect" => checkpoints::read(
            root,
            request["id"].as_str().ok_or("Falta identificador")?,
            "metrics.json",
            65536,
        ),
        "generate" => {
            let started = Instant::now();
            let id = request["id"].as_str().ok_or("Falta identificador")?;
            let saved = checkpoints::load(root, id, false)?;
            let prefix = request["prefix"].as_str().ok_or("Falta prefijo")?;
            let count = request["max_tokens"]
                .as_u64()
                .filter(|v| (1..=256).contains(v))
                .ok_or("Salida Tiny: 1 a 256 caracteres")? as usize;
            let text = sample_from(&saved.weights, prefix, count)?;
            Ok(
                json!({"content":text,"model":"Boddy Tiny 1.0","id":id,"backend":"CPU reference","context_tokens":CONFIG.context,"completion_tokens":count,"elapsed_ms":started.elapsed().as_millis(),"purpose":"Continuacion experimental de caracteres; no es respuesta a instrucciones"}),
            )
        }
        _ => Err("Accion de modelo no permitida".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tokenizer_is_fixed_and_rejects_unicode() {
        assert_eq!(encode("\n ~").unwrap(), vec![0, 1, 95]);
        assert!(encode("ñ").is_err());
    }
    #[test]
    fn corpus_and_context_are_valid() {
        assert!(encode(CORPUS).unwrap().len() > CONFIG.context * 2);
        assert_eq!(CONFIG.parameters(), 27776);
        assert_eq!(
            sample(&initialize(CONFIG, 1337).unwrap()).unwrap().len(),
            264
        );
    }
    #[test]
    fn rejects_unbounded_training() {
        assert!(train(Path::new("."), 2001).is_err());
        assert!(train(Path::new("."), 0).is_err());
    }
}
