//! Biblioteca documental del host: indice primero, fragmentos despues. Sin red ni inferencia.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Component, Path},
    time::{SystemTime, UNIX_EPOCH},
};
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn slug(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 48
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn relative(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', ':', '\0'])
        && path
            .split('/')
            .all(|s| !s.is_empty() && !s.starts_with('.'))
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}
fn read(root: &Path, path: &str, limit: u64) -> Result<Vec<u8>, String> {
    if !relative(path) {
        return Err("Ruta documental fuera de alcance".into());
    }
    boddy_reader::plain_path(root, Path::new(path))?;
    let mut bytes = Vec::new();
    fs::File::open(root.join(path))
        .map_err(|e| format!("{path}: {e}"))?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("Documento demasiado grande".into());
    }
    Ok(bytes)
}
fn json_file(root: &Path, path: &str, limit: u64) -> Result<Value, String> {
    serde_json::from_slice(&read(root, path, limit)?).map_err(|e| e.to_string())
}
fn normalize(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' => 'a',
            'é' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'ï' => 'i',
            'ó' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            'ñ' => 'n',
            _ => c,
        })
        .collect()
}
fn terms(query: &str) -> Vec<String> {
    let n = normalize(query);
    let mut v = Vec::new();
    for s in n.split(|c: char| !c.is_alphanumeric()) {
        if s.len() > 2
            && !matches!(
                s,
                "que"
                    | "como"
                    | "para"
                    | "por"
                    | "los"
                    | "las"
                    | "una"
                    | "del"
                    | "con"
                    | "son"
                    | "sus"
                    | "este"
                    | "esta"
                    | "porque"
                    | "the"
                    | "and"
                    | "what"
                    | "why"
                    | "how"
            )
            && !v.iter().any(|x| x == s)
        {
            v.push(s.to_string());
        }
    }
    v
}
struct Catalog {
    cards: Vec<Value>,
    categories: Vec<Value>,
    hash: String,
}
impl Catalog {
    fn load(root: &Path) -> Result<Self, String> {
        let raw = read(root, "biblioteca/registro.json", 131072)?;
        let registry: Value = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
        let raw_categories = read(root, "biblioteca/categorias.json", 16384)?;
        let cats: Value = serde_json::from_slice(&raw_categories).map_err(|e| e.to_string())?;
        if registry["schema"] != 1 || cats["schema"] != 1 {
            return Err("Version de biblioteca incompatible".into());
        }
        let categories = cats["categories"]
            .as_array()
            .filter(|a| !a.is_empty() && a.len() <= 16)
            .ok_or("Categorias invalidas")?
            .clone();
        let cards = registry["cards"]
            .as_array()
            .filter(|a| a.len() <= 256)
            .ok_or("Registro demasiado grande")?
            .clone();
        let mut category_ids = std::collections::BTreeSet::new();
        for cat in &categories {
            let id = cat["id"]
                .as_str()
                .filter(|s| slug(s))
                .ok_or("Categoria invalida")?;
            let path = cat["path"]
                .as_str()
                .filter(|s| relative(s) && !s.contains('/'))
                .ok_or("Carpeta de categoria invalida")?;
            if !category_ids.insert(id) {
                return Err("Categoria duplicada".into());
            }
            boddy_reader::plain_path(root, Path::new(&format!("biblioteca/{path}")))?;
        }
        let mut ids = std::collections::BTreeSet::new();
        for card in &cards {
            let id = card["id"]
                .as_str()
                .filter(|s| slug(s))
                .ok_or("ID documental invalido")?;
            if !ids.insert(id) {
                return Err("ID documental duplicado".into());
            }
            let cat = categories
                .iter()
                .find(|c| c["id"] == card["category"])
                .ok_or("Categoria no registrada")?;
            let path = card["path"]
                .as_str()
                .filter(|p| {
                    relative(p)
                        && p.ends_with(".md")
                        && p.starts_with(&format!("{}/", cat["path"].as_str().unwrap()))
                })
                .ok_or("Ruta documental invalida")?;
            boddy_reader::plain_path(root, Path::new(&format!("biblioteca/{path}")))?;
            for (key, max) in [("title", 200), ("summary", 512)] {
                if card[key]
                    .as_str()
                    .map_or(true, |s| s.is_empty() || s.len() > max)
                {
                    return Err(format!("Campo {key} invalido"));
                }
            }
            if !matches!(
                card["status"].as_str(),
                Some("authored_note" | "authored_protocol" | "outline")
            ) {
                return Err("Estado documental desconocido".into());
            }
            for (key, max) in [("tags", 16), ("related", 16), ("sources", 8)] {
                let values = card[key]
                    .as_array()
                    .filter(|a| a.len() <= max)
                    .ok_or("Metadatos demasiado grandes")?;
                if values
                    .iter()
                    .any(|v| v.as_str().map_or(true, |s| s.len() > 512))
                {
                    return Err("Metadato invalido".into());
                }
            }
        }
        for card in &cards {
            for id in card["related"].as_array().unwrap() {
                if !ids.contains(id.as_str().unwrap()) {
                    return Err("Relacion documental desconocida".into());
                }
            }
        }
        let mut combined = raw;
        combined.push(0);
        combined.extend(raw_categories);
        Ok(Self {
            cards,
            categories,
            hash: sha(&combined),
        })
    }
    fn card(&self, id: &str) -> Result<&Value, String> {
        self.cards
            .iter()
            .find(|c| c["id"] == id)
            .ok_or_else(|| format!("Documento no registrado: {id}"))
    }
    fn search(&self, query: &str, category: Option<&str>) -> Result<Vec<Value>, String> {
        if query.is_empty() || query.len() > 256 {
            return Err("Consulta documental: 1 a 256 bytes".into());
        }
        if let Some(id) = category {
            if !self.categories.iter().any(|c| c["id"] == id) {
                return Err("Categoria desconocida".into());
            }
        }
        let words = terms(query);
        let exact = normalize(query);
        let mut found = Vec::new();
        for c in &self.cards {
            if category.is_some_and(|id| c["category"] != id) {
                continue;
            }
            let title = normalize(c["title"].as_str().unwrap());
            let summary = normalize(c["summary"].as_str().unwrap());
            let tags = normalize(
                &c["tags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            let mut score = if c["id"] == query { 1000 } else { 0 };
            for word in &words {
                if title.contains(word) {
                    score += 5;
                }
                if tags.contains(word) {
                    score += 8;
                }
                if summary.contains(word) {
                    score += 2;
                }
            }
            if title.contains(&exact) {
                score += 10;
            }
            if score > 0 {
                found.push((score, c.clone()));
            }
        }
        found.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| a.1["id"].as_str().cmp(&b.1["id"].as_str()))
        });
        Ok(found
            .into_iter()
            .map(|(score, mut c)| {
                c["score"] = json!(score);
                c
            })
            .collect())
    }
}
fn query(request: &Value) -> Result<&str, String> {
    request["query"]
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= 256)
        .ok_or_else(|| "Falta query (1..256 bytes)".into())
}
fn budget(request: &Value) -> Result<usize, String> {
    match request.get("budget_bytes") {
        None => Ok(1024),
        Some(v) => v
            .as_u64()
            .filter(|n| (256..=4096).contains(n))
            .map(|n| n as usize)
            .ok_or_else(|| "Presupuesto: 256 a 4096 bytes UTF8".into()),
    }
}
fn category(request: &Value) -> Result<Option<&str>, String> {
    match request.get("category") {
        None | Some(Value::Null) => Ok(None),
        Some(v) => Ok(Some(v.as_str().ok_or("Categoria no textual")?)),
    }
}
fn floor(text: &str, max: usize) -> &str {
    let mut end = text.len().min(max);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}
fn passage(text: &str, query: &str, max: usize) -> (String, usize, usize, bool) {
    let words = terms(query);
    let mut best = None;
    let mut offset = 0;
    let separator = if text.contains("\r\n") {
        "\r\n\r\n"
    } else {
        "\n\n"
    };
    for part in text.split_inclusive(separator) {
        let trimmed = part.trim_end();
        if !trimmed.is_empty() && !(trimmed.starts_with("# ") && !trimmed.contains('\n')) {
            let n = normalize(trimmed);
            let score = words.iter().filter(|w| n.contains(w.as_str())).count();
            if best.map_or(true, |(s, _, _)| score > s) {
                best = Some((score, offset, trimmed.len()));
            }
        }
        offset += part.len();
    }
    let (_, start, length) = best.unwrap_or((0, 0, text.len()));
    let selected = &text[start..start + length];
    let excerpt = floor(selected, max);
    let from = text[..start].bytes().filter(|b| *b == b'\n').count() + 1;
    let to = from + excerpt.bytes().filter(|b| *b == b'\n').count();
    (
        excerpt.to_string(),
        from,
        to,
        excerpt.len() < selected.len(),
    )
}
fn packet(
    root: &Path,
    catalog: &Catalog,
    ids: &[String],
    query: &str,
    budget: usize,
) -> Result<Value, String> {
    if ids.len() > 3 {
        return Err("Maximo tres documentos por contexto".into());
    }
    let mut context = String::new();
    let mut refs = Vec::new();
    let mut source_bytes = 0;
    let mut seen = std::collections::BTreeSet::new();
    for id in ids {
        let remaining = budget.saturating_sub(context.len());
        if remaining < 160 {
            break;
        }
        if !seen.insert(id) {
            continue;
        }
        let card = catalog.card(id)?;
        let path = format!("biblioteca/{}", card["path"].as_str().unwrap());
        let bytes = read(root, &path, 16384)?;
        source_bytes += bytes.len();
        let hash = sha(&bytes);
        let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
        let (snippet, from, to, truncated) = passage(text, query, remaining.saturating_sub(144));
        if snippet.is_empty() {
            continue;
        }
        let header = format!("[{id} L{from}-L{to} SHA256:{hash}]\n");
        let item = format!("{header}{snippet}\n\n");
        if context.len() + item.len() > budget {
            return Err("Contexto excedio presupuesto".into());
        }
        context.push_str(&item);
        refs.push(json!({"id":id,"title":card["title"],"path":path,"sha256":hash,"from_line":from,"to_line":to,"truncated":truncated,"sources":card["sources"],"status":card["status"],"related":card["related"]}));
    }
    Ok(
        json!({"schema":1,"query":query,"catalog_sha256":catalog.hash,"context":context,"references":refs,"budget_bytes":budget,"context_bytes":context.len(),"source_bytes_read":source_bytes,"documents_read":seen.len(),"token_count":null,"note":"Bytes del texto de contexto, no tokens ni todo el prompt. El tokenizer de destino debe contar tokens. Notas propias con enlaces; hash no verifica verdad semantica.","training_effect":"none"}),
    )
}
fn retrieve(root: &Path, catalog: &Catalog, request: &Value) -> Result<Value, String> {
    let q = query(request)?;
    let found = catalog.search(q, category(request)?)?;
    let ids: Vec<_> = found
        .iter()
        .take(3)
        .map(|c| c["id"].as_str().unwrap().to_string())
        .collect();
    let mut p = packet(root, catalog, &ids, q, budget(request)?)?;
    p["metadata_matches"] = json!(found.len());
    Ok(p)
}
fn new_id(prefix: &str) -> Result<String, String> {
    Ok(format!(
        "{prefix}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
    ))
}
fn write(root: &Path, path: &str, bytes: &[u8]) -> Result<(), String> {
    if !relative(path) || !path.starts_with("data/library/") {
        return Err("Destino fuera de biblioteca".into());
    }
    boddy_reader::plain_path(root, Path::new(path))?;
    let target = root.join(path);
    fs::create_dir_all(target.parent().ok_or("Sin carpeta destino")?).map_err(|e| e.to_string())?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())
}
fn memory_id(request: &Value) -> Result<&str, String> {
    let id = request["memory_id"].as_str().ok_or("Falta memory_id")?;
    if !id
        .strip_prefix("mem-")
        .is_some_and(|s| !s.is_empty() && s.len() <= 30 && s.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err("Recuerdo invalido".into());
    }
    Ok(id)
}
fn recall(root: &Path, catalog: &Catalog, request: &Value) -> Result<Value, String> {
    let id = memory_id(request)?;
    let memory = json_file(root, &format!("data/library/memories/{id}.json"), 8192)?;
    if memory["schema"] != 1 || memory["id"] != id {
        return Err("Version o identidad de recuerdo invalida".into());
    }
    let refs = memory["references"]
        .as_array()
        .filter(|r| r.len() <= 3)
        .ok_or("Referencias de recuerdo invalidas")?;
    let ids: Vec<String> = refs
        .iter()
        .map(|r| {
            r["id"]
                .as_str()
                .filter(|s| slug(s))
                .map(str::to_string)
                .ok_or("Referencia invalida")
        })
        .collect::<Result<_, _>>()?;
    let original_query = memory["query"]
        .as_str()
        .filter(|s| s.len() <= 256)
        .ok_or("Consulta guardada invalida")?;
    let b = memory["budget_bytes"]
        .as_u64()
        .filter(|n| (256..=4096).contains(n))
        .ok_or("Presupuesto guardado invalido")? as usize;
    let q = if request.get("query").is_some() {
        query(request)?
    } else {
        original_query
    };
    let b = if request.get("budget_bytes").is_some() {
        budget(request)?
    } else {
        b
    };
    let mut p = packet(root, catalog, &ids, q, b)?;
    p["original_query"] = json!(original_query);
    let changes:Vec<_>=p["references"].as_array().unwrap().iter().map(|current|{let old=refs.iter().find(|r|r["id"]==current["id"]).unwrap();json!({"id":current["id"],"content_changed":old["sha256"]!=current["sha256"],"path_changed":old["path"]!=current["path"]})}).collect();
    p["memory_id"] = json!(id);
    p["catalog_changed"] = json!(memory["catalog_sha256"] != catalog.hash);
    p["changes"] = json!(changes);
    p["reread"] = json!(true);
    Ok(p)
}
fn languages(root: &Path, catalog: &Catalog, request: &Value) -> Result<Value, String> {
    let q = normalize(request["query"].as_str().unwrap_or(""));
    if q.len() > 128 {
        return Err("Consulta de idiomas demasiado larga".into());
    }
    let q = match q.as_str() {
        "espanol" => "spanish",
        "ingles" => "english",
        "frances" => "french",
        "aleman" => "german",
        "japones" => "japanese",
        s => s,
    };
    let meta = json_file(root, "biblioteca/02-babel/procedencia.json", 4096)?;
    let path = "biblioteca/02-babel/idiomas.tsv";
    boddy_reader::plain_path(root, Path::new(path))?;
    let f = fs::File::open(root.join(path)).map_err(|e| e.to_string())?;
    if f.metadata().map_err(|e| e.to_string())?.len() > 524288 {
        return Err("Catalogo de idiomas demasiado grande".into());
    }
    let mut reader = BufReader::new(f);
    let mut line = Vec::new();
    let mut digest = Sha256::new();
    let mut rows = 0;
    let mut found = Vec::new();
    let mut matches = 0;
    let mut first = true;
    loop {
        line.clear();
        let n = reader
            .by_ref()
            .take(2049)
            .read_until(b'\n', &mut line)
            .map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        if n > 2048 {
            return Err("Linea de idioma demasiado larga".into());
        }
        digest.update(&line);
        let text = std::str::from_utf8(&line)
            .map_err(|e| e.to_string())?
            .trim_end_matches(['\n', '\r']);
        if first {
            first = false;
            if text != "code\tname\tdeprecated\tpreferred\tscope" {
                return Err("Cabecera de idiomas invalida".into());
            }
            continue;
        }
        rows += 1;
        if rows > 10000 {
            return Err("Demasiados idiomas".into());
        }
        let fields: Vec<_> = text.split('\t').collect();
        if fields.len() != 5 {
            return Err("Registro de idioma invalido".into());
        }
        if q.is_empty() || normalize(fields[1]).contains(q) || fields[0].contains(q) {
            matches += 1;
            let score = if fields[0] == q {
                1000
            } else if fields[0].starts_with(q) {
                100
            } else {
                1
            };
            let cards: Vec<_> = catalog
                .cards
                .iter()
                .filter(|c| c["language"] == fields[0])
                .map(|c| json!({"id":c["id"],"status":c["status"]}))
                .collect();
            found.push((score,json!({"code":fields[0],"name":fields[1],"deprecated":fields[2],"preferred":fields[3],"scope":fields[4],"school_status":if cards.is_empty(){"catalog_only"}else{"outline_only"},"local_documents":cards})));
            found.sort_by(|a, b| {
                b.0.cmp(&a.0)
                    .then_with(|| a.1["code"].as_str().cmp(&b.1["code"].as_str()))
            });
            found.truncate(12);
        }
    }
    if meta["catalog_sha256"] != format!("{:x}", digest.finalize()) || meta["records"] != rows {
        return Err("Catalogo IANA cambiado: regenerar procedencia".into());
    }
    Ok(
        json!({"records":rows,"matched":matches,"results":found.into_iter().map(|(_,v)|v).collect::<Vec<_>>(),"provenance":meta,"note":"Metadatos IANA, no cursos ni dominio de idiomas por el modelo."}),
    )
}
pub fn command(root: &Path, request: &Value) -> Result<Value, String> {
    let catalog = Catalog::load(root)?;
    match request["action"]
        .as_str()
        .ok_or("Falta accion de biblioteca")?
    {
        "memories" => {
            let folder = "data/library/memories";
            boddy_reader::plain_path(root, Path::new(folder))?;
            let mut ids = Vec::new();
            let mut truncated = false;
            match fs::read_dir(root.join(folder)) {
                Ok(entries) => {
                    for (i, e) in entries.take(513).enumerate() {
                        if i == 512 {
                            truncated = true;
                            break;
                        }
                        let e = e.map_err(|e| e.to_string())?;
                        let name = e.file_name().to_string_lossy().into_owned();
                        if let Some(id) = name.strip_suffix(".json") {
                            if memory_id(&json!({"memory_id":id})).is_ok() {
                                ids.push(id.to_string());
                            }
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.to_string()),
            }
            ids.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| b.cmp(a)));
            truncated |= ids.len() > 64;
            let mut memories = Vec::new();
            let mut invalid = 0;
            for id in ids.iter().take(64) {
                match json_file(root,&format!("{folder}/{id}.json"),8192){Ok(m)if m["schema"]==1&&m["id"]==id.as_str()=>memories.push(json!({"memory_id":id,"query":m["query"],"references":m["references"].as_array().map_or(0,Vec::len)})),_=>invalid+=1}
            }
            Ok(json!({"memories":memories,"invalid":invalid,"truncated":truncated}))
        }
        "categories" => Ok(
            json!({"categories":catalog.categories,"cards":catalog.cards.len(),"catalog_sha256":catalog.hash}),
        ),
        "search" => {
            let found = catalog.search(query(request)?, category(request)?)?;
            Ok(
                json!({"matches":found.len(),"cards":found.into_iter().take(12).collect::<Vec<_>>(),"documents_read":0,"scope":"Solo metadatos del registro"}),
            )
        }
        "read" => {
            let id = request["id"].as_str().ok_or("Falta id documental")?;
            packet(
                root,
                &catalog,
                &[id.to_string()],
                request["query"].as_str().unwrap_or(id),
                budget(request)?,
            )
        }
        "context" => retrieve(root, &catalog, request),
        "remember" => {
            let p = retrieve(root, &catalog, request)?;
            if p["references"].as_array().unwrap().is_empty() {
                return Err("No hay referencias que recordar".into());
            }
            let id = new_id("mem")?;
            let refs: Vec<_> = p["references"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| json!({"id":r["id"],"path":r["path"],"sha256":r["sha256"]}))
                .collect();
            let memory = json!({"schema":1,"id":id,"query":p["query"],"budget_bytes":p["budget_bytes"],"catalog_sha256":p["catalog_sha256"],"references":refs});
            let bytes = serde_json::to_vec(&memory).map_err(|e| e.to_string())?;
            let path = format!("data/library/memories/{id}.json");
            write(root, &path, &bytes)?;
            Ok(
                json!({"memory_id":id,"path":path,"bytes":bytes.len(),"references":memory["references"],"note":"Referencias persistentes; no cambia pesos neuronales."}),
            )
        }
        "recall" => recall(root, &catalog, request),
        "export" => {
            let p = if request.get("memory_id").is_some() {
                recall(root, &catalog, request)?
            } else {
                retrieve(root, &catalog, request)?
            };
            let id = new_id("packet")?;
            let j = format!("data/library/exports/{id}.json");
            let md = format!("data/library/exports/{id}.md");
            let bytes = serde_json::to_vec(&p).map_err(|e| e.to_string())?;
            write(root, &j, &bytes)?;
            let mut markdown = format!(
                "# Contexto documental\n\n{}\n\n{}",
                p["query"].as_str().unwrap(),
                p["context"].as_str().unwrap()
            );
            markdown.push_str("\n## Procedencia\n\n");
            for r in p["references"].as_array().unwrap() {
                markdown.push_str(&format!(
                    "- {}: {} ({}). Fuentes: {}\n",
                    r["id"].as_str().unwrap(),
                    r["path"].as_str().unwrap(),
                    r["sha256"].as_str().unwrap(),
                    r["sources"]
                ));
            }
            write(root, &md, markdown.as_bytes())?;
            Ok(
                json!({"json_path":j,"markdown_path":md,"json_bytes":bytes.len(),"context_bytes":p["context_bytes"],"references":p["references"]}),
            )
        }
        "languages" => languages(root, &catalog, request),
        _ => Err("Accion documental no permitida".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(document: bool) -> std::path::PathBuf {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("data/library/fixtures")
            .join(new_id("test").unwrap());
        fs::create_dir_all(root.join("biblioteca/01-forja")).unwrap();
        let categories =
            json!({"schema":1,"categories":[{"id":"forja","name":"Forja","path":"01-forja"}]});
        let registry = json!({"schema":1,"cards":[{"id":"rust-propiedad","category":"forja","path":"01-forja/propiedad.md","title":"Rust propiedad","summary":"Ownership y memoria","tags":["rust","propiedad","ownership"],"related":[],"sources":["nota de prueba"],"status":"authored_note"}]});
        fs::write(
            root.join("biblioteca/categorias.json"),
            serde_json::to_vec(&categories).unwrap(),
        )
        .unwrap();
        fs::write(
            root.join("biblioteca/registro.json"),
            serde_json::to_vec(&registry).unwrap(),
        )
        .unwrap();
        if document {
            fs::write(root.join("biblioteca/01-forja/propiedad.md"),format!("# Rust\n\n## Propiedad\nLa propiedad conserva información: {}\n\n## Otro panorama\nUna perspectiva distinta.\n","áéíóúñ🧠".repeat(80))).unwrap();
        }
        root
    }
    #[test]
    fn search_reads_metadata_without_opening_documents() {
        let root = fixture(false);
        let r = command(&root, &json!({"action":"search","query":"ownership"})).unwrap();
        assert_eq!(r["documents_read"], 0);
        assert_eq!(r["cards"][0]["id"], "rust-propiedad");
        assert!(command(&root, &json!({"action":"context","query":"ownership"})).is_err());
    }
    #[test]
    fn budget_preserves_unicode_and_true_source_references() {
        let root = fixture(true);
        let file = root.join("biblioteca/01-forja/propiedad.md");
        let windows = fs::read_to_string(&file).unwrap().replace('\n', "\r\n");
        fs::write(&file, windows).unwrap();
        let p = command(
            &root,
            &json!({"action":"context","query":"propiedad","budget_bytes":256}),
        )
        .unwrap();
        let context = p["context"].as_str().unwrap();
        assert!(context.len() <= 256);
        assert_eq!(p["context_bytes"], context.len());
        let raw = fs::read(root.join("biblioteca/01-forja/propiedad.md")).unwrap();
        assert_eq!(p["references"][0]["sha256"], sha(&raw));
        let snippet = context.split_once('\n').unwrap().1.trim_end();
        assert!(std::str::from_utf8(&raw).unwrap().contains(snippet));
        assert_eq!(p["token_count"], Value::Null);
    }
    #[test]
    fn recall_rereads_and_detects_changed_content() {
        let root = fixture(true);
        let remembered = command(
            &root,
            &json!({"action":"remember","query":"propiedad","budget_bytes":512}),
        )
        .unwrap();
        let id = remembered["memory_id"].as_str().unwrap();
        let before = command(&root, &json!({"action":"recall","memory_id":id})).unwrap();
        assert_eq!(before["changes"][0]["content_changed"], false);
        fs::write(
            root.join("biblioteca/01-forja/propiedad.md"),
            "# Revisado\n\n## Otro panorama\nPropiedad revisada con otra perspectiva.\n",
        )
        .unwrap();
        let after = command(
            &root,
            &json!({"action":"recall","memory_id":id,"query":"otro panorama"}),
        )
        .unwrap();
        assert_eq!(after["changes"][0]["content_changed"], true);
        assert_eq!(after["reread"], true);
        assert!(after["context"].as_str().unwrap().contains("perspectiva"));
        assert_eq!(after["training_effect"], "none");
        let record = json_file(&root, remembered["path"].as_str().unwrap(), 8192).unwrap();
        assert!(record.get("context").is_none());
    }
    #[test]
    fn rejects_escape_routes_and_excessive_budget() {
        let root = fixture(true);
        assert!(command(&root, &json!({"action":"read","id":"../private"})).is_err());
        assert!(command(&root, &json!({"action":"recall","memory_id":"../private"})).is_err());
        assert!(command(
            &root,
            &json!({"action":"context","query":"rust","budget_bytes":4097})
        )
        .is_err());
        let mut registry = json_file(&root, "biblioteca/registro.json", 131072).unwrap();
        registry["cards"][0]["path"] = json!("01-forja/../../private.md");
        fs::write(
            root.join("biblioteca/registro.json"),
            serde_json::to_vec(&registry).unwrap(),
        )
        .unwrap();
        assert!(Catalog::load(&root).is_err());
    }
    #[test]
    fn exports_only_the_selected_context_and_provenance() {
        let root = fixture(true);
        let e = command(
            &root,
            &json!({"action":"export","query":"propiedad","budget_bytes":512}),
        )
        .unwrap();
        let json = json_file(&root, e["json_path"].as_str().unwrap(), 16384).unwrap();
        let md = fs::read_to_string(root.join(e["markdown_path"].as_str().unwrap())).unwrap();
        assert!(json["context"].as_str().unwrap().len() <= 512);
        assert!(md.contains("rust-propiedad"));
        assert!(md.contains(json["references"][0]["sha256"].as_str().unwrap()));
    }
    #[test]
    fn language_registry_distinguishes_catalog_from_schools() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let en = command(root, &json!({"action":"languages","query":"en"})).unwrap();
        assert!(en["records"].as_u64().unwrap() > 7000);
        assert_eq!(en["results"][0]["code"], "en");
        assert_eq!(en["results"][0]["school_status"], "outline_only");
        let fr = command(root, &json!({"action":"languages","query":"fr"})).unwrap();
        assert_eq!(fr["results"][0]["code"], "fr");
        assert_eq!(fr["results"][0]["school_status"], "catalog_only");
    }
}
