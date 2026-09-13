use super::process_protocol::{
    read_wire, write_create_new, ProcessReportPayload, ProcessSubjectReport, ProcessSubjectRequest,
    SUBJECT_REQUEST_ENV,
};
use super::{
    apply_process_poison, production_store::produce_closed_store, RootWireIdentity, RootWireRole,
};

pub(super) fn run() {
    if super::namespace_courtroom::run_subject_if_requested() {
        return;
    }
    if super::physical_work_courtroom::run_subject_if_requested() {
        return;
    }
    if super::artifact_process::run_subject_if_requested() {
        return;
    }
    let Some(request_path) = std::env::var_os(SUBJECT_REQUEST_ENV) else {
        return;
    };
    let request: ProcessSubjectRequest = read_wire(request_path.as_ref())
        .unwrap_or_else(|error| panic!("C9 process request: {error}"));
    request
        .require_version()
        .unwrap_or_else(|error| panic!("C9 process request: {error}"));
    let (store_identity, payload) = match request.role() {
        RootWireRole::Producer => {
            assert!(request.store_identity().is_none());
            assert!(request.manifest().is_none());
            assert!(request.poison().is_none());
            let manifest = produce_closed_store(request.store_root(), request.profile())
                .unwrap_or_else(|error| panic!("produce closed Store: {error}"));
            (
                manifest.store_identity(),
                ProcessReportPayload::Produced(manifest),
            )
        }
        RootWireRole::ArtifactEditor => {
            let manifest = request.manifest().expect("editor manifest");
            assert_eq!(request.store_identity(), Some(manifest.store_identity()));
            let audit = apply_process_poison(
                request.store_root(),
                manifest,
                request.poison().expect("editor declaration"),
            )
            .unwrap_or_else(|error| panic!("edit isolated Store: {error:?}"));
            (
                manifest.store_identity(),
                ProcessReportPayload::Edited(audit),
            )
        }
        RootWireRole::Recovery => {
            let expected_store_identity =
                request.store_identity().expect("expected Store identity");
            assert!(request.manifest().is_none());
            assert!(request.poison().is_none());
            let observation =
                super::recovery_adapter::recover(request.store_root(), request.profile())
                    .unwrap_or_else(|error| panic!("invoke C8 recovery root: {error}"));
            let observed_store_identity = observation
                .require_store_identity(expected_store_identity)
                .unwrap_or_else(|denial| panic!("recovery Store identity denied: {denial:?}"));
            (
                observed_store_identity,
                ProcessReportPayload::Recovered(observation),
            )
        }
        role => panic!("unsupported C9 process subject role: {role:?}"),
    };
    let wire = RootWireIdentity::bind(
        request.role(),
        request.scenario_identity(),
        request.run_identity(),
        store_identity,
    )
    .expect("bind C9 process report");
    let report = ProcessSubjectReport::new(wire, payload).expect("construct C9 process report");
    write_create_new(request.report_path(), &report)
        .unwrap_or_else(|error| panic!("emit C9 process report: {error}"));
}
