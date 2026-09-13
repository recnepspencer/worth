mod corruption;
mod expectations;
mod process_roles;
mod runner;

pub(super) use runner::run;

#[test]
#[ignore = "requires the Cargo-built independent observer"]
fn namespace_identity_process_matrix() {
    let executable = std::env::var_os(super::OBSERVER_EXECUTABLE_ENV).unwrap();
    run(std::path::Path::new(&executable));
}

pub(super) fn run_subject_if_requested() -> bool {
    let Some(path) = std::env::var_os(process_roles::REQUEST_ENV) else {
        return false;
    };
    let request = super::process_protocol::read_wire(path.as_ref()).unwrap();
    process_roles::run_subject(request);
    true
}
