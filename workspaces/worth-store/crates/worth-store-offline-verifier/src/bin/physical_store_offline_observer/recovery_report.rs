#[path = "recovery_report/arguments.rs"]
mod arguments;
#[path = "recovery_report/limits.rs"]
mod limits;
#[path = "recovery_report/observation.rs"]
mod observation;
#[path = "recovery_report/persistence.rs"]
mod persistence;
#[path = "recovery_report/rendering.rs"]
mod rendering;

pub(super) fn run(arguments: impl Iterator<Item = std::ffi::OsString>) -> Result<(), String> {
    let invocation = arguments::parse(arguments)?;
    let limits = limits::build(&invocation)?;
    let report = observation::execute(&invocation.root, limits)?;
    persistence::write(&invocation.output, &report)?;
    rendering::emit(&report);
    Ok(())
}
