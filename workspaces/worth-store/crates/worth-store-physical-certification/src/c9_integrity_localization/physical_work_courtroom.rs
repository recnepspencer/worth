mod artifact_manifest;
mod corruption;
mod expectations;
mod open_store;
mod pending_producer;
mod process_protocol;
mod runner;
mod runtime_observation;

pub(super) use runner::run;

#[test]
#[ignore = "requires the Cargo-built independent observer"]
fn pending_obligation_process_matrix() {
    let executable = std::env::var_os(super::OBSERVER_EXECUTABLE_ENV).unwrap();
    run(std::path::Path::new(&executable));
}

pub(super) fn run_subject_if_requested() -> bool {
    let Some(path) = std::env::var_os(process_protocol::REQUEST_ENV) else {
        return false;
    };
    let request: process_protocol::Request =
        super::process_protocol::read_wire(path.as_ref()).expect("bounded PW process request");
    assert_eq!(request.version, 1);
    match request.role {
        process_protocol::Role::PendingProducer => pending_producer::run(&request),
        process_protocol::Role::Editor(operator) => corruption::run_editor(&request, operator),
        process_protocol::Role::RuntimeObserver => runtime_observation::run(&request),
    }
    true
}
