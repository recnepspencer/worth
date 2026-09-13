use std::path::Path;

use super::process_courtroom_assertions::{
    assert_addressed_root_poison_preserves_current_selector, assert_offline_expectation,
    assert_recovery_expectation, DecoderCounters,
};
use super::process_execution::{fresh_identity, hex, run_offline_observer, run_subject};
use super::process_identity_substitution::assert_recovery_store_substitution_is_denied;
use super::process_manifest::ProcessTreeSnapshot;
use super::process_protocol::{executable_sha256, ProcessReportPayload, ProcessSubjectRequest};
use super::{DeclaredProcessPoison, ProcessRootCase, RootWireRole};

pub(super) fn run(observer_executable: &Path) {
    assert!(
        observer_executable.is_file(),
        "Cargo-owned observer executable is unavailable"
    );
    let mut world = tempfile::tempdir().expect("courtroom directory");
    if std::env::var_os("WORTH_C9_RETAIN_WORLD").is_some() {
        world.disable_cleanup(true);
        println!("C9 retained evidence world={}", world.path().display());
    }
    let stores = world.path().join("stores");
    let reports = world.path().join("reports");
    std::fs::create_dir_all(&stores).expect("Store rows directory");
    std::fs::create_dir_all(&reports).expect("external reports directory");
    let parent_executable = std::env::current_exe().expect("courtroom executable");
    let parent_digest = executable_sha256(&parent_executable).expect("courtroom digest");
    let producer_scenario = fresh_identity("producer-scenario", world.path());
    let producer_run = fresh_identity("producer-run", world.path());
    let baseline = stores.join("closed-production-baseline");
    let producer = run_subject(
        &parent_executable,
        &reports,
        "producer",
        ProcessSubjectRequest::producer(
            producer_scenario,
            producer_run,
            baseline.clone(),
            reports.join("producer.report"),
        ),
    );
    let manifest = match producer.report.payload() {
        ProcessReportPayload::Produced(manifest) => manifest.clone(),
        _ => panic!("producer report payload substitution"),
    };
    producer
        .report
        .require(
            RootWireRole::Producer,
            producer_scenario,
            producer_run,
            manifest.store_identity(),
            producer.process_id,
            parent_digest,
        )
        .expect("producer report binding");
    assert!(manifest.file_count() > 0);
    assert!(manifest.byte_count() > 0);
    manifest
        .require_unchanged(&baseline)
        .expect("producer baseline remains exact after producer exit");
    let inventory = super::artifact_inventory::ArtifactInventory::observe(&baseline);
    assert!(
        inventory.granules.len() > 200,
        "the primary courtroom covers the actual published topology"
    );

    let observer_digest = executable_sha256(observer_executable).expect("observer digest");
    assert_ne!(
        observer_digest, parent_digest,
        "recovery/verifier binary reuse"
    );
    let mut clean_counters = None;
    for case in [
        ProcessRootCase::CleanControl,
        ProcessRootCase::PoisonCurrentSelector,
        ProcessRootCase::PoisonAddressedRoot,
    ] {
        let row = stores.join(case.label());
        manifest.copy_to(&baseline, &row).expect("fresh exact row");
        let scenario = fresh_identity(&format!("{}-scenario", case.label()), world.path());
        let declared_poison = case.role().map(|_| {
            DeclaredProcessPoison::for_case(&manifest, case)
                .expect("independent poison declaration")
        });
        if let Some(declaration) = declared_poison.as_ref() {
            let editor_snapshot =
                ProcessTreeSnapshot::observe(&row).expect("snapshot row before editor process");
            let editor_run = fresh_identity(&format!("{}-editor", case.label()), world.path());
            let editor = run_subject(
                &parent_executable,
                &reports,
                &format!("{}-editor", case.label()),
                ProcessSubjectRequest::editor(
                    scenario,
                    editor_run,
                    row.clone(),
                    reports.join(format!("{}-editor.report", case.label())),
                    manifest.clone(),
                    declaration.clone(),
                ),
            );
            editor
                .report
                .require(
                    RootWireRole::ArtifactEditor,
                    scenario,
                    editor_run,
                    manifest.store_identity(),
                    editor.process_id,
                    parent_digest,
                )
                .expect("editor report binding");
            let audit = match editor.report.payload() {
                ProcessReportPayload::Edited(audit) => audit,
                _ => panic!("editor report payload substitution"),
            };
            let artifact = manifest
                .artifact(declaration.role())
                .expect("declared editor target");
            let (actual_before_sha256, actual_after_sha256) = editor_snapshot
                .require_exact_one_byte_delta(
                    &row,
                    artifact.relative_path(),
                    declaration.offset(),
                    declaration.xor_mask(),
                )
                .expect("editor changes exactly the declared byte and nothing else");
            assert_eq!(audit.declaration_identity(), declaration.identity());
            assert_eq!(audit.changed_offset(), declaration.offset());
            assert_eq!(audit.before_sha256(), actual_before_sha256);
            assert_eq!(audit.after_sha256(), actual_after_sha256);
            if case == ProcessRootCase::PoisonAddressedRoot {
                assert_addressed_root_poison_preserves_current_selector(&row, &manifest);
            }
        }
        let observer_snapshot =
            ProcessTreeSnapshot::observe(&row).expect("snapshot row before offline observation");
        let observer_run = fresh_identity(&format!("{}-observer", case.label()), world.path());
        let offline_report = reports.join(format!("{}-offline.json", case.label()));
        let observed = run_offline_observer(
            observer_executable,
            &row,
            &offline_report,
            observer_run,
            scenario,
        );
        assert_eq!(
            observed.report["protocol"],
            "store.physical.integrity-observation"
        );
        assert_eq!(observed.report["version"], 1);
        assert_eq!(observed.report["role"], "offline-root-observer");
        assert_eq!(
            observed.report["executable"],
            "physical_store_integrity_observer"
        );
        assert_eq!(observed.report["process"], observed.process_id.to_string());
        assert_eq!(observed.report["run"], hex(observer_run));
        assert_eq!(observed.report["scenario"], hex(scenario));
        assert_eq!(observed.report["store"], hex(manifest.store_identity()));
        assert_eq!(observed.executable_sha256, observer_digest);
        assert_eq!(
            observed.report["completeness"], "complete",
            "offline report: {}",
            observed.report
        );
        assert_offline_expectation(&observed.report, &manifest, declared_poison.as_ref());
        observer_snapshot
            .require_unchanged(&row)
            .expect("offline observer leaves isolated Store row byte-exact");
        let counters = DecoderCounters::from_report(&observed.report);
        match case {
            ProcessRootCase::CleanControl => {
                assert!(counters.checksum_calculations > 0);
                assert!(counters.checksum_validated_frames > 0);
                assert!(counters.selector_payload_entries > 0);
                assert!(counters.root_manifest_payload_entries > 0);
                clean_counters = Some(counters);
            }
            ProcessRootCase::PoisonCurrentSelector => {
                let clean = clean_counters.expect("clean row precedes poison rows");
                // Lost current reachability intentionally stops descendant traversal. Global
                // checksum totals therefore cannot serve as a root decoder-entry oracle.
                assert_eq!(
                    counters.selector_payload_entries + 1,
                    clean.selector_payload_entries
                );
                assert_eq!(
                    counters.root_manifest_payload_entries,
                    clean.root_manifest_payload_entries
                );
            }
            ProcessRootCase::PoisonAddressedRoot => {
                let clean = clean_counters.expect("clean row precedes poison rows");
                assert_eq!(
                    counters.selector_payload_entries,
                    clean.selector_payload_entries
                );
                assert_eq!(
                    counters.root_manifest_payload_entries + 1,
                    clean.root_manifest_payload_entries
                );
            }
        }
        let recovery_run = fresh_identity(&format!("{}-recovery", case.label()), world.path());
        let recovery = run_subject(
            &parent_executable,
            &reports,
            &format!("{}-recovery", case.label()),
            ProcessSubjectRequest::recovery(
                scenario,
                recovery_run,
                row.clone(),
                reports.join(format!("{}-recovery.report", case.label())),
                manifest.store_identity(),
            ),
        );
        recovery
            .report
            .require(
                RootWireRole::Recovery,
                scenario,
                recovery_run,
                manifest.store_identity(),
                recovery.process_id,
                parent_digest,
            )
            .expect("recovery report binding");
        let recovery_observation = match recovery.report.payload() {
            ProcessReportPayload::Recovered(observation) => observation,
            _ => panic!("recovery report payload substitution"),
        };
        println!(
            "C9 row={} recovery={:?} effects={} discovery={:?}",
            case.label(),
            recovery_observation.posture,
            recovery_observation.recovery_effects,
            recovery_observation.discovery
        );
        assert_recovery_expectation(recovery_observation, &manifest, case);
    }
    assert_recovery_store_substitution_is_denied(
        &parent_executable,
        &stores,
        &reports,
        &baseline,
        &manifest,
        fresh_identity("recovery-store-substitution-scenario", world.path()),
        fresh_identity("recovery-store-substitution-run", world.path()),
    );
    manifest
        .require_unchanged(&baseline)
        .expect("courtroom never edits the production baseline");
    super::artifact_courtroom::run(
        observer_executable,
        &baseline,
        &stores,
        &reports,
        super::production_profile::ProductionWorldProfile::Primary16KiB,
        &manifest,
    );
    run_page_profiles(observer_executable, &parent_executable, &stores, &reports);
    super::physical_work_courtroom::run(observer_executable);
    super::namespace_courtroom::run(observer_executable);
}

#[test]
#[ignore = "requires the independently built C9 observer"]
fn c9_page_profiles_process_courtroom() {
    let observer = std::env::var_os(super::OBSERVER_EXECUTABLE_ENV).expect("Cargo-built observer");
    let world = tempfile::tempdir().unwrap();
    let stores = world.path().join("stores");
    let reports = world.path().join("reports");
    std::fs::create_dir(&stores).unwrap();
    std::fs::create_dir(&reports).unwrap();
    run_page_profiles(
        Path::new(&observer),
        &std::env::current_exe().unwrap(),
        &stores,
        &reports,
    );
}

fn run_page_profiles(observer: &Path, executable: &Path, stores: &Path, reports: &Path) {
    use super::production_profile::ProductionWorldProfile::{Pages32KiB, Pages64KiB};
    for profile in [Pages32KiB, Pages64KiB] {
        let label = format!("{}-producer", profile.label());
        let baseline = stores.join(format!("{}-baseline", profile.label()));
        let scenario = fresh_identity(&format!("{label}-scenario"), stores);
        let run = fresh_identity(&format!("{label}-run"), reports);
        let producer = run_subject(
            executable,
            reports,
            &label,
            ProcessSubjectRequest::producer(
                scenario,
                run,
                baseline.clone(),
                reports.join(format!("{label}.report")),
            )
            .with_profile(profile),
        );
        let ProcessReportPayload::Produced(manifest) = producer.report.payload() else {
            panic!("page world producer role");
        };
        producer
            .report
            .require(
                RootWireRole::Producer,
                scenario,
                run,
                manifest.store_identity(),
                producer.process_id,
                executable_sha256(executable).unwrap(),
            )
            .unwrap();
        let recovery_scenario = fresh_identity(&format!("{label}-recovery-scenario"), stores);
        let recovery_run = fresh_identity(&format!("{label}-recovery-run"), reports);
        let row = stores.join(format!("{}-recovery", profile.label()));
        manifest.copy_to(&baseline, &row).unwrap();
        let recovered = run_subject(
            executable,
            reports,
            &format!("{label}-recovery"),
            ProcessSubjectRequest::recovery(
                recovery_scenario,
                recovery_run,
                row,
                reports.join(format!("{label}-recovery.report")),
                manifest.store_identity(),
            )
            .with_profile(profile),
        );
        recovered
            .report
            .require(
                RootWireRole::Recovery,
                recovery_scenario,
                recovery_run,
                manifest.store_identity(),
                recovered.process_id,
                executable_sha256(executable).unwrap(),
            )
            .unwrap();
        let ProcessReportPayload::Recovered(observation) = recovered.report.payload() else {
            panic!("page world recovery role");
        };
        assert_recovery_expectation(observation, manifest, ProcessRootCase::CleanControl);
        super::artifact_courtroom::run(observer, &baseline, stores, reports, profile, manifest);
    }
}
