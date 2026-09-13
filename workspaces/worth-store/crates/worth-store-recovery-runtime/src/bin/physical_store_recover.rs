#[path = "physical_store_recover/admission.rs"]
mod admission;
#[path = "physical_store_recover/arguments.rs"]
mod arguments;
#[path = "physical_store_recover/fate_marker.rs"]
mod fate_marker;
#[path = "physical_store_recover/outcome.rs"]
mod outcome;
#[path = "physical_store_recover/report.rs"]
mod report;
#[path = "physical_store_recover/terminal.rs"]
mod terminal;

use std::process::ExitCode;

fn main() -> ExitCode {
    match run(std::env::args_os().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("physical_store_recover: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: Vec<std::ffi::OsString>) -> Result<(), String> {
    let invocation = arguments::parse(arguments)?;
    let request = admission::open_request(&invocation.root, invocation.profile)?;
    let outcome = terminal::execute(request, invocation.profile, invocation.yieldpoint)?;
    report::persist(invocation.report_path.as_deref(), &outcome)?;
    outcome::render(outcome)
}
