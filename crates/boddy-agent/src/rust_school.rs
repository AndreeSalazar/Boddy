//! Busqueda autonoma acotada de expresiones conocidas; no carga codigo Rust.
use boddy_core::{Mission, Policy, Task, Tool};
use boddy_rust_tasks::{verify, Exercise, EXERCISES, EXPRESSIONS};
use serde_json::{json, Value};
use std::{
    fs,
    io::Write,
    path::Path,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
fn core<T>(r: Result<T, boddy_core::Error>) -> Result<T, String> {
    r.map_err(|e| format!("{e:?}"))
}
pub fn catalog() -> Value {
    json!({"curriculum":"rust-expressions-v1","tasks":EXERCISES.iter().map(|e|json!({"id":e.id,"goal":e.goal,"cases":e.cases.len()})).collect::<Vec<_>>(),"candidates":EXPRESSIONS.iter().map(|op|json!({"id":op.id(),"body":op.source()})).collect::<Vec<_>>(),"scope":"Biblioteca cerrada, casos finitos; no compilacion ni ejecucion de proyectos"})
}
fn solve(exercise: &Exercise, limit: usize, scores: &[f32]) -> Result<Value, String> {
    if !(1..=3).contains(&limit)
        || scores.len() != EXPRESSIONS.len()
        || scores.iter().any(|v| !v.is_finite())
    {
        return Err("Ranking o presupuesto Rust invalido".into());
    }
    let mission = core(Mission::new(
        "Aprender un ciclo de propuesta y verificacion Rust",
        exercise.goal,
        "Superar los cinco casos del verificador de expresiones",
    ))?;
    let mut task = core(Task::new(
        mission,
        Policy::local_research(),
        (limit + 1) as u16,
    ))?;
    core(task.start())?;
    core(task.request_tool(Tool::ReadDocuments))?;
    core(task.record_evidence("boddy-rust-tasks:rust-expressions-v1"))?;
    core(task.set_plan("ranking_nll_medio_de_candidatos"))?;
    let mut order: Vec<_> = (0..scores.len()).collect();
    order.sort_by(|a, b| scores[*a].total_cmp(&scores[*b]).then_with(|| a.cmp(b)));
    let ranked: Vec<_> = order
        .iter()
        .map(|i| json!({"expression":EXPRESSIONS[*i].id(),"mean_nll":scores[*i]}))
        .collect();
    let mut attempts = Vec::new();
    let mut chosen = None;
    for (index, candidate) in order.iter().take(limit).enumerate() {
        let op = EXPRESSIONS[*candidate];
        core(task.request_tool(Tool::ProposePatch))?;
        core(task.begin_verification())?;
        let checked = verify(exercise, op);
        let passed = checked.passed == checked.total;
        core(task.record_verification(passed, "rust-expressions-v1:finite_cases"))?;
        attempts.push(json!({"attempt":index+1,"expression":op.id(),"body":op.source(),"cases_passed":checked.passed,"cases_total":checked.total,"first_failed_case":checked.first_failure,"passed":passed}));
        if passed {
            core(task.complete())?;
            chosen = Some(op);
            break;
        } else if index + 1 < limit {
            core(task.revise())?;
        } else {
            core(task.stop())?;
        }
    }
    let source = chosen.map(|op| {
        format!(
            "pub fn {}(a: i32, b: i32) -> i32 {{ {} }}",
            exercise.id,
            op.source()
        )
    });
    Ok(
        json!({"id":exercise.id,"goal":exercise.goal,"state":format!("{:?}",task.stage()),"first_choice_passed":attempts.first().is_some_and(|a|a["passed"]==true),"verified_after_search":chosen.is_some(),"tool_calls":task.tool_calls(),"ranking":ranked,"attempts":attempts,"source":source,"verification_scope":"Cinco casos de una expresion tipada conocida; no prueba general ni compilacion"}),
    )
}
pub fn run<F>(
    root: &Path,
    model_id: &str,
    limit: u64,
    model: Value,
    mut score: F,
) -> Result<Value, String>
where
    F: FnMut(&str, &[&str]) -> Result<Vec<f32>, String>,
{
    if !(1..=3).contains(&limit) {
        return Err("Escuela Rust: 1 a 3 intentos por ejercicio".into());
    }
    let started = Instant::now();
    let bodies: Vec<_> = EXPRESSIONS.iter().map(|op| op.source()).collect();
    let mut results = Vec::new();
    for exercise in &EXERCISES {
        let prefix = format!(
            "fn completar(a: i32, b: i32) -> i32 {{\n// objetivo: {}\n",
            exercise.tag
        );
        let scores = score(&prefix, &bodies)?;
        results.push(solve(exercise, limit as usize, &scores)?);
    }
    let first = results
        .iter()
        .filter(|r| r["first_choice_passed"] == true)
        .count();
    let solved = results
        .iter()
        .filter(|r| r["verified_after_search"] == true)
        .count();
    let id = format!(
        "school-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
    );
    let relative = format!("data/rust-school/{id}");
    boddy_reader::plain_path(root, Path::new(&relative))?;
    let dir = root.join(&relative);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut code=String::from("// Propuestas de una biblioteca cerrada. Verificadas solo en casos finitos.\n// No compiladas ni ejecutadas por este agente. No garantizan ausencia de overflow.\n");
    for r in &results {
        if let Some(source) = r["source"].as_str() {
            code.push_str(source);
            code.push('\n');
        }
    }
    let report = json!({"schema":1,"id":id,"agent":"Rust school bounded candidate search","model_id":model_id,"model_provenance":model,"curriculum":"rust-expressions-v1","state":if solved==EXERCISES.len(){"Completed"}else{"Stopped"},"task_count":EXERCISES.len(),"first_choice_passed":first,"verified_after_search":solved,"max_attempts_per_task":limit,"candidate_count":EXPRESSIONS.len(),"scored_completion_bytes":bodies.iter().map(|s|s.len()).sum::<usize>()*EXERCISES.len(),"policy":{"execute_code":Policy::local_research().allows(Tool::ExecuteCode),"network":Policy::local_research().allows(Tool::Network),"write_scope":"Solo artefactos de esta tarea en data/rust-school"},"results":results,"source_path":format!("{relative}/solutions.rs"),"journal_path":format!("{relative}/report.json"),"elapsed_ms":started.elapsed().as_millis(),"limitations":["Nueve cuerpos de funciones conocidos; no genera programas libres","Completar por busqueda no demuestra que el modelo sepa programar","Aciertos del primer candidato y del agente se informan por separado","Casos conocidos durante desarrollo, con operandos distintos del corpus","Sin rustc, shell, red ni cambios a proyectos; sin autoentrenamiento"]});
    for (name, bytes) in [
        ("solutions.rs", code.into_bytes()),
        (
            "report.json",
            serde_json::to_vec(&report).map_err(|e| e.to_string())?,
        ),
    ] {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(name))
            .map_err(|e| e.to_string())?;
        f.write_all(&bytes).map_err(|e| e.to_string())?;
    }
    Ok(report)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn revisions_are_bounded_and_cannot_fabricate_success() {
        let mut scores = vec![10.0; 9];
        scores[1] = 0.0;
        scores[0] = 1.0;
        let stopped = solve(&EXERCISES[0], 1, &scores).unwrap();
        assert_eq!(stopped["state"], "Stopped");
        assert!(stopped["source"].is_null());
        let complete = solve(&EXERCISES[0], 2, &scores).unwrap();
        assert_eq!(complete["state"], "Completed");
        assert_eq!(complete["first_choice_passed"], false);
        assert_eq!(complete["verified_after_search"], true);
        assert_eq!(complete["attempts"].as_array().unwrap().len(), 2);
    }
    #[test]
    fn rejects_unknown_rankings() {
        assert!(solve(&EXERCISES[0], 4, &[0.0; 9]).is_err());
        assert!(solve(&EXERCISES[0], 1, &[f32::NAN; 9]).is_err());
        assert!(solve(&EXERCISES[0], 1, &[0.0; 8]).is_err());
    }
}
