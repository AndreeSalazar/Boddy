//! Acciones manuales de escritorio enumeradas; no ejecuta texto del modelo.
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct Action {
    pub program: PathBuf,
    pub args: Vec<String>,
}

pub fn plan(root: &Path, action: &str, source: Option<&str>) -> Result<Action, String> {
    let windows =
        PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into()));
    let text = |p: &Path| p.to_string_lossy().to_string();
    let (program, args) = match action {
        "open_project" => (windows.join("explorer.exe"), vec![text(root)]),
        "open_reports" => {
            boddy_reader::plain_path(root, Path::new("data/tasks"))?;
            if !root.join("data/tasks").is_dir() {
                return Err("No hay registros todavia".into());
            }
            (
                windows.join("explorer.exe"),
                vec![text(&root.join("data/tasks"))],
            )
        }
        "open_task_manager" => (windows.join("System32/taskmgr.exe"), vec![]),
        "open_graph" => (
            root.parent()
                .ok_or("Proyecto sin padre")?
                .join("SUPER GRAFO/ultra-carpetas/target/release/ultra-carpetas.exe"),
            vec!["--grafo".into(), text(root)],
        ),
        "open_editor" => {
            let source = source.ok_or("Selecciona una fuente")?;
            boddy_reader::read(root, source, &[])?;
            (
                windows.join("System32/notepad.exe"),
                vec![text(&root.join(source))],
            )
        }
        _ => return Err("Accion de escritorio no registrada".into()),
    };
    Ok(Action { program, args })
}

fn audit(root: &Path, event: Value) -> Result<(), String> {
    let path = Path::new("runtime/desktop-actions.jsonl");
    boddy_reader::plain_path(root, path)?;
    std::fs::create_dir_all(root.join("runtime")).map_err(|e| e.to_string())?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join(path))
        .map_err(|e| e.to_string())?;
    writeln!(file, "{event}").map_err(|e| e.to_string())
}

pub fn run(root: &Path, request: &Value) -> Result<Value, String> {
    let name = request["action"].as_str().ok_or("Falta accion")?;
    let action = plan(root, name, request["path"].as_str())?;
    if request["preview"].as_bool() == Some(true) {
        return Ok(
            json!({"action":name,"program":action.program,"args":action.args,"launched":false}),
        );
    }
    if !action.program.is_file() {
        return Err("Aplicacion no disponible en la ruta registrada".into());
    }
    audit(
        root,
        json!({"origin":"manual_desktop_command","action":name,"path":request["path"],"status":"requested"}),
    )?;
    match Command::new(&action.program)
        .args(&action.args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => {
            audit(
                root,
                json!({"action":name,"status":"launched","pid":child.id()}),
            )?;
            Ok(json!({"action":name,"launched":true,"pid":child.id()}))
        }
        Err(error) => {
            audit(
                root,
                json!({"action":name,"status":"failed","error":error.to_string()}),
            )?;
            Err(error.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_shell_execution_and_unscoped_editor_paths() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        for action in [
            "shell",
            "powershell",
            "delete",
            "shutdown",
            "open_project;calc.exe",
        ] {
            assert!(plan(root, action, None).is_err());
        }
        assert!(plan(root, "open_editor", Some("runtime/access.key")).is_err());
        assert!(plan(root, "open_editor", Some("../secret.rs")).is_err());
        let project = plan(root, "open_project", None).unwrap();
        assert_eq!(project.args.len(), 1);
        assert_eq!(project.args[0], root.to_string_lossy());
        let script = plan(root, "open_editor", Some("window/Boddy.vbs")).unwrap();
        assert!(script.program.ends_with("notepad.exe"));
        assert_eq!(script.args.len(), 1);
    }
}
