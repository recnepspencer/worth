//! A selected V3 result cannot lose its maintenance obligation
//! while preserving a canonical, same-route root and free-space inventory.

use std::{
    fs,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use worth_store::physical_runtime::PhysicalRecoveryYieldpointStage;
use worth_store_physical_format::{
    DurablePhysicalRootManifest, DurableRootSelector, RecordArtifactFile,
};
use worth_store_recovery_runtime::{PhysicalRecoveryBlockKind, PhysicalRecoveryOutcome};

#[test]
fn selected_released_result_cannot_clear_maintenance() {
    let world = super::pending_wal_world::first();
    let root = world.root();
    let selected_before = fs::read(root.join("families/records/root-current.selector")).unwrap();
    let checkpoint_before = fs::read(root.join("families/checkpoint.current")).unwrap();
    let selected = DurableRootSelector::decode(&selected_before).unwrap();
    let generation = selected.root_generation() + 1;
    let marker = tempfile::tempdir().unwrap();
    let reached = marker.path().join("candidate.reached");
    let release = marker.path().join("candidate.release");
    let mut child = Command::new(env!("CARGO_BIN_EXE_physical_store_recover"))
        .arg(root)
        .arg("--bounded-profile=c8-phase8-fate-coverage-v1")
        .arg(format!(
            "--report={}",
            marker.path().join("report.bin").display()
        ))
        .arg(format!(
            "--yieldpoint-stage={}",
            PhysicalRecoveryYieldpointStage::RootProtocolReplacement.label()
        ))
        .arg(format!("--yieldpoint-reached={}", reached.display()))
        .arg(format!("--yieldpoint-release={}", release.display()))
        .arg(format!(
            "--yieldpoint-cancel={}",
            marker.path().join("cancel").display()
        ))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(240);
    while !reached.is_file() {
        if child.try_wait().unwrap().is_some() || Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "V3 recovery did not publish its selected result: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(20));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let selected_result = fs::read(root.join("families/records/root-current.selector")).unwrap();
    assert_ne!(selected_result, selected_before);
    let selected_result_frame = DurableRootSelector::decode(&selected_result).unwrap();
    assert_eq!(selected_result_frame.root_generation(), generation);

    let artifact = RecordArtifactFile::RootManifest { generation };
    let path = root
        .join("families/records/roots")
        .join(artifact.file_name());
    let bytes = fs::read(&path).unwrap_or_else(|error| {
        let roots = fs::read_dir(root.join("families/records/roots"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        panic!(
            "actual selected V3 result root at {} after source generation {}: {error}; roots={roots:?}",
            path.display(),
            selected.root_generation()
        )
    });
    let (candidate, format) = DurablePhysicalRootManifest::decode(&bytes, u16::MAX).unwrap();
    assert!(candidate.requires_maintenance_protocol());
    assert!(candidate.tier_epoch_anchor().is_none());
    let false_maintenance = DurablePhysicalRootManifest::builder(
        candidate.generation(),
        candidate.tree_identity(),
        candidate.node_capacity(),
        candidate.free_space_checksum(),
    )
    .record_count(candidate.record_count())
    .next_block(candidate.next_block())
    .next_segment_block(candidate.next_segment_block())
    .routing_root(candidate.routing_root())
    .segment_root(candidate.segment_root())
    .free_space_root(candidate.free_space_root())
    .latest_blob_publication(candidate.latest_blob_publication())
    .latest_blob_quarantine(candidate.latest_blob_quarantine())
    .derived_family_directory(candidate.derived_family_directory())
    .last_inline_record(candidate.last_inline_record())
    .last_inline_segment(candidate.last_inline_segment())
    .admit()
    .unwrap();
    assert!(!false_maintenance.requires_maintenance_protocol());
    fs::write(&path, false_maintenance.encode(format)).unwrap();
    let (mutated, mutated_format) =
        DurablePhysicalRootManifest::decode(&fs::read(&path).unwrap(), u16::MAX).unwrap();
    assert_eq!(mutated_format, format);
    assert_eq!(mutated, false_maintenance);

    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        super::certified_release_serving::request(root),
    );
    let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("false-maintenance selected V3 result was admitted: {outcome:?}");
    };
    assert_eq!(blocked.recovery_effects(), 0);
    assert_eq!(blocked.kind, PhysicalRecoveryBlockKind::RedoPlanning);
    assert!(
        blocked
            .evidence()
            .planning_counters
            .as_ref()
            .is_some_and(|counters| counters.redo_skip_historical_drop() > 0),
        "historical V3 path was not reached: {:?}",
        blocked.evidence().planning_counters
    );
    assert_eq!(
        fs::read(root.join("families/records/root-current.selector")).unwrap(),
        selected_result
    );
    assert_eq!(
        fs::read(root.join("families/checkpoint.current")).unwrap(),
        checkpoint_before
    );

    // The same interrupted-publication world must recover once its one
    // substituted semantic bit is restored. This prevents an unrelated
    // interrupted-root defect from satisfying the denial assertion.
    fs::write(&path, bytes).unwrap();
    let restored = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        super::certified_release_serving::request(root),
    );
    assert!(
        matches!(&restored, PhysicalRecoveryOutcome::Recovered(_)),
        "original selected V3 result did not recover: {restored:?}"
    );
}
