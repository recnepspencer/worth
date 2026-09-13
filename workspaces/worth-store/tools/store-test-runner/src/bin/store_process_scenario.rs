use std::process::ExitCode;

fn main() -> ExitCode {
    match store_test_runner::run_process_scenario_from_environment() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("store-process-scenario: {error}");
            ExitCode::FAILURE
        }
    }
}
