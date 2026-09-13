//! Focused real-process journal lane. Full C9 acceptance still runs every family.
use super::{
    process_execution::{fresh_identity, run_subject},
    process_protocol::{executable_sha256, ProcessReportPayload, ProcessSubjectRequest},
    RootWireRole,
};

#[test]
#[ignore = "requires the independently built C9 observer"]
fn c9_journal_process_courtroom() {
    let observer = std::env::var_os(super::OBSERVER_EXECUTABLE_ENV).expect("Cargo-built observer");
    let mut world = tempfile::tempdir().unwrap();
    if std::env::var_os("WORTH_C9_RETAIN_WORLD").is_some() {
        world.disable_cleanup(true);
    }
    println!("C9 journal world={}", world.path().display());
    let stores = world.path().join("stores");
    let reports = world.path().join("reports");
    std::fs::create_dir(&stores).unwrap();
    std::fs::create_dir(&reports).unwrap();
    let root = stores.join("production-root");
    let executable = std::env::current_exe().unwrap();
    let scenario = fresh_identity("journal-producer-scenario", world.path());
    let run = fresh_identity("journal-producer-run", world.path());
    let producer = run_subject(
        &executable,
        &reports,
        "producer",
        ProcessSubjectRequest::producer(
            scenario,
            run,
            root.clone(),
            reports.join("producer.report"),
        ),
    );
    let ProcessReportPayload::Produced(manifest) = producer.report.payload() else {
        panic!("actual producer")
    };
    producer
        .report
        .require(
            RootWireRole::Producer,
            scenario,
            run,
            manifest.store_identity(),
            producer.process_id,
            executable_sha256(&executable).unwrap(),
        )
        .unwrap();
    manifest.require_unchanged(&root).unwrap();
    super::artifact_courtroom::run_journals(
        std::path::Path::new(&observer),
        &root,
        &stores,
        &reports,
        manifest,
    );
}
