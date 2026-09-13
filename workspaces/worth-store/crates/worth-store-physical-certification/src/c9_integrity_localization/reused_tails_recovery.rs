use super::process_courtroom_assertions::assert_recovery_expectation;
use super::process_execution::{fresh_identity, run_subject};
use super::process_protocol::{executable_sha256, ProcessReportPayload, ProcessSubjectRequest};
use super::production_profile::ProductionWorldProfile;
use super::{ProcessRootCase, RootWireRole};

#[test]
fn all_eight_partial_tail_batches_recover_cleanly_in_a_fresh_process() {
    let world = tempfile::tempdir().unwrap();
    let reports = world.path().join("reports");
    std::fs::create_dir(&reports).unwrap();
    let baseline = world.path().join("baseline");
    let executable = std::env::current_exe().unwrap();
    let executable_digest = executable_sha256(&executable).unwrap();
    let profile = ProductionWorldProfile::ReusedTails16KiB;
    let scenario = fresh_identity("reused-tails-producer-scenario", world.path());
    let run = fresh_identity("reused-tails-producer-run", world.path());
    let producer = run_subject(
        &executable,
        &reports,
        "producer",
        ProcessSubjectRequest::producer(
            scenario,
            run,
            baseline.clone(),
            reports.join("producer.report"),
        )
        .with_profile(profile),
    );
    let ProcessReportPayload::Produced(manifest) = producer.report.payload() else {
        panic!("the producer must report its actual closed Store")
    };
    producer
        .report
        .require(
            RootWireRole::Producer,
            scenario,
            run,
            manifest.store_identity(),
            producer.process_id,
            executable_digest,
        )
        .unwrap();
    assert_eq!(manifest.records.len(), 8 * (65 + 1));
    assert_eq!(
        manifest
            .records
            .iter()
            .filter(|record| record.length == 3_000)
            .count(),
        8 * 65
    );
    assert_eq!(
        manifest
            .records
            .iter()
            .filter(|record| record.length == 64 * 1024)
            .count(),
        8
    );

    let recovery_root = world.path().join("recovery");
    manifest.copy_to(&baseline, &recovery_root).unwrap();
    let scenario = fresh_identity("reused-tails-recovery-scenario", world.path());
    let run = fresh_identity("reused-tails-recovery-run", world.path());
    let recovery = run_subject(
        &executable,
        &reports,
        "recovery",
        ProcessSubjectRequest::recovery(
            scenario,
            run,
            recovery_root,
            reports.join("recovery.report"),
            manifest.store_identity(),
        )
        .with_profile(profile),
    );
    assert_ne!(producer.process_id, recovery.process_id);
    recovery
        .report
        .require(
            RootWireRole::Recovery,
            scenario,
            run,
            manifest.store_identity(),
            recovery.process_id,
            executable_digest,
        )
        .unwrap();
    let ProcessReportPayload::Recovered(observation) = recovery.report.payload() else {
        panic!("the recovery process must report its actual outcome")
    };
    assert_recovery_expectation(observation, manifest, ProcessRootCase::CleanControl);
    manifest.require_unchanged(&baseline).unwrap();
}
