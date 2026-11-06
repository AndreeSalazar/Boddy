use std::process::ExitCode;
fn main() -> ExitCode {
    match boddy_app::run(&std::env::args().skip(1).collect::<Vec<_>>()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Boddy: {error}");
            ExitCode::FAILURE
        }
    }
}
