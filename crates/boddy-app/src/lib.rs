use boddy_catalog as catalog;
use boddy_supergrafo as graph;

use boddy_core::{Mission, Policy, Task, Tool};
use std::io::{self, Read, Write};
use std::path::Path;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn demo() -> Result<(), String> {
    println!("DEMO DETERMINISTA del controlador. Sin LLM, GPU ni código generado.");
    let result = (|| -> Result<(), boddy_core::Error> {
        let mission = Mission::new(
            "Mantener una investigación trazable",
            "Preparar el plan inicial de Boddy",
            "Plan con referencias y estado explícito",
        )?;
        let mut task = Task::new(mission, Policy::local_research(), 4)?;
        task.start()?;
        task.request_tool(Tool::ReadDocuments)?;
        task.record_evidence("knowledge/sources.tsv#A01")?;
        task.set_plan("docs/PLAN.md")?;
        println!(
            "Estado: {:?}; fuentes: {}; llamadas reservadas: {}",
            task.stage(),
            task.evidence_count(),
            task.tool_calls()
        );
        println!(
            "Ejecución de código: {:?}",
            task.request_tool(Tool::ExecuteCode)
        );
        task.begin_verification()?;
        println!("Completar sin verificador: {:?}", task.complete());
        println!("La demo termina pendiente de verificación; no fabrica un resultado aprobado.");
        Ok(())
    })();
    result.map_err(|e| format!("{e:?}"))
}

pub fn run(args: &[String]) -> Result<(), String> {
    match args {
        [command]
            if command == "entrenar"
                || command == "modelo"
                || command == "atencion"
                || command == "escuela-rust" =>
        {
            let mut input = String::new();
            io::stdin()
                .take(4097)
                .read_to_string(&mut input)
                .map_err(|e| e.to_string())?;
            if input.len() > 4096 {
                return Err("Peticion de laboratorio demasiado grande".into());
            }
            let request: serde_json::Value =
                serde_json::from_str(&input).map_err(|e| e.to_string())?;
            let result = if command == "escuela-rust" {
                match request["action"].as_str().ok_or("Falta accion Rust")? {
                    "tasks" => boddy_agent::rust_school::catalog(),
                    "run" => {
                        let id = request["id"].as_str().ok_or("Falta modelo Attention")?;
                        let ranker = boddy_lab::attention::Ranker::load(root(), id)?;
                        boddy_agent::rust_school::run(
                            root(),
                            id,
                            request["max_attempts"]
                                .as_u64()
                                .ok_or("Falta max_attempts")?,
                            ranker.provenance(),
                            |prefix, bodies| ranker.scores(prefix, bodies),
                        )?
                    }
                    _ => return Err("Accion Rust no permitida".into()),
                }
            } else if command == "atencion" {
                boddy_lab::attention::command(root(), &request)?
            } else if command == "modelo" {
                boddy_lab::model_command(root(), &request)?
            } else {
                boddy_lab::train_from(
                    root(),
                    request["steps"].as_u64().ok_or("Falta steps entero")?,
                    match request.get("parent_id") {
                        None | Some(serde_json::Value::Null) => None,
                        Some(value) => Some(value.as_str().ok_or("parent_id debe ser texto")?),
                    },
                )?
            };
            println!("{result}");
            Ok(())
        }
        [command] if command == "pc" => {
            let mut input = String::new();
            io::stdin()
                .take(4097)
                .read_to_string(&mut input)
                .map_err(|e| e.to_string())?;
            if input.len() > 4096 {
                return Err("Peticion de escritorio demasiado grande".into());
            }
            let request: serde_json::Value =
                serde_json::from_str(&input).map_err(|e| e.to_string())?;
            let response = match request["action"].as_str().ok_or("Falta accion")? {
                "resources" => serde_json::json!({"gpu":boddy_telemetry::snapshot()}),
                "source_list" => boddy_reader::list(root())?,
                "source_preview" => {
                    boddy_reader::read(root(), request["path"].as_str().ok_or("Falta ruta")?, &[])?
                        .as_json("preview")
                }
                "task_list" => boddy_agent::records::list(root())?,
                "task_record" => boddy_agent::records::record(
                    root(),
                    request["id"].as_str().ok_or("Falta identificador")?,
                )?,
                _ => boddy_desktop::run(root(), &request)?,
            };
            println!("{response}");
            Ok(())
        }
        [command] if command == "investigar" => {
            let started = std::time::Instant::now();
            let mut input = String::new();
            io::stdin()
                .take(4097)
                .read_to_string(&mut input)
                .map_err(|e| e.to_string())?;
            if input.len() > 4096 {
                return Err("Peticion de investigacion demasiado grande".into());
            }
            let request: serde_json::Value =
                serde_json::from_str(&input).map_err(|e| e.to_string())?;
            let objective = request["objective"].as_str().ok_or("Falta objetivo")?;
            let repairs = match request.get("repair_attempts") {
                None => 1,
                Some(v) => v
                    .as_u64()
                    .filter(|v| *v <= 1)
                    .ok_or("repair_attempts debe ser 0 o 1")? as u8,
            };
            let mut response = boddy_agent::investigate_with_options(root(), objective, repairs)?;
            response["usage"] = serde_json::json!({"prompt_tokens":response["task"]["usage"]["prompt_tokens"],"completion_tokens":response["task"]["usage"]["completion_tokens"],"prompt_tokens_details":{"cached_tokens":response["task"]["usage"]["cached_tokens"]}});
            response["elapsed_ms"] = serde_json::json!(started.elapsed().as_millis());
            response["gpu"] = boddy_telemetry::snapshot();
            println!("{response}");
            Ok(())
        }
        [command] if command == "chat" => {
            let mut input = String::new();
            io::stdin()
                .take(65537)
                .read_to_string(&mut input)
                .map_err(|e| e.to_string())?;
            if input.len() > 65536 {
                return Err("Entrada de chat demasiado grande".into());
            }
            let mut request: serde_json::Value =
                serde_json::from_str(&input).map_err(|e| format!("Entrada JSON inválida: {e}"))?;
            if !request.is_object() {
                return Err("Se requiere un objeto JSON".into());
            }
            request.as_object_mut().unwrap().remove("graph_context");
            if request["use_graph"].as_bool() == Some(true) {
                let bytes = graph::query(root(), "resumen")?;
                request["graph_context"] =
                    serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            }
            let mut response = boddy_inference::chat(&request)?;
            response["gpu"] = boddy_telemetry::snapshot();
            println!("{response}");
            Ok(())
        }
        [] => {
            println!("Boddy — IA personal local\n\nComandos:\n  chat (JSON por stdin)\n  investigar (JSON por stdin)\n  pc (JSON por stdin)\n  escuela-rust (JSON por stdin: tasks, run)\n  atencion (JSON por stdin: train, list, inspect, generate)\n  modelo (JSON por stdin: list, inspect, generate)\n  entrenar (JSON por stdin: steps 1..2000, parent_id opcional)\n  proposito\n  fuentes [texto]\n  demo\n  grafo [resumen|fallos|ciclos]\n\nChat local y laboratorio Tiny reanudable; ejecucion autonoma y sandbox pendientes.");
            Ok(())
        }
        [command] if command == "proposito" => {
            print!("{}", include_str!("../../../docs/PROPOSITO.md"));
            Ok(())
        }
        [command, rest @ ..] if command == "fuentes" && rest.len() <= 1 => {
            let query = rest.first().map(String::as_str).unwrap_or("");
            let matches = catalog::search(query);
            println!(
                "{} fuentes; mostrando hasta 10. Metadatos, sin documentos ingeridos.",
                matches.len()
            );
            println!("{}", catalog::DATA.lines().next().unwrap());
            for line in matches.iter().take(10) {
                println!("{line}");
            }
            Ok(())
        }
        [command] if command == "demo" => demo(),
        [command, rest @ ..] if command == "grafo" && rest.len() <= 1 => {
            let mission = Mission::new(
                "Orientarse sin cargar el repositorio completo",
                "Consultar el mapa de Boddy",
                "Respuesta acotada de SUPER GRAFO",
            )
            .map_err(|e| format!("{e:?}"))?;
            let mut task =
                Task::new(mission, Policy::local_research(), 1).map_err(|e| format!("{e:?}"))?;
            task.start().map_err(|e| format!("{e:?}"))?;
            task.request_tool(Tool::ReadGraph)
                .map_err(|e| format!("{e:?}"))?;
            let bytes = graph::query(
                root(),
                rest.first().map(String::as_str).unwrap_or("resumen"),
            )?;
            io::stdout().write_all(&bytes).map_err(|e| e.to_string())
        }
        _ => Err("Comando inválido. Ejecuta cargo run -p boddy-cli -- para ver la ayuda".into()),
    }
}
