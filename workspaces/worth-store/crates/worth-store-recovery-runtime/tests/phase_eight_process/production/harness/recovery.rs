use std::collections::BTreeSet;
use std::path::Path;

use worth_store_recovery_runtime::RecoveryReportEnvelope;

use super::super::super::history;
use super::super::fate_markers::{indexed_fate_tags, parse_indexed_recovery_fates};
use super::markers::{parse_recovery_fate_marker, parse_recovery_runtime_marker};
use super::{assert_child_succeeded, run_recovery_with_profile, ProcessWorld, RuntimeProcess};

pub(super) fn recover_root_with_profile(
    world: &ProcessWorld,
    root: &Path,
    name: &str,
    profile: &str,
) -> RuntimeProcess {
    let report_path = world
        .parent
        .path()
        .join(format!("{name}-runtime-report.bin"));
    let persisted_fates = history::classify_persisted_fates(&world.writer.expected, root)
        .unwrap_or_else(|error| panic!("persisted fate oracle failed: {error}"));
    let (process_id, output) =
        run_recovery_with_profile(root, &report_path, world.parent.path(), profile);
    assert_child_succeeded(name, &output);
    let marker = parse_recovery_runtime_marker(&output);
    assert_ne!(
        marker.runtime, world.writer.runtime_identity,
        "recovery must issue a fresh runtime identity rather than reusing the killed writer identity"
    );
    let fates = parse_recovery_fate_marker(&output);
    let indexed_fates = parse_indexed_recovery_fates(&output);
    assert_eq!(
        indexed_fates.len() as u64,
        fates.total(),
        "production recovery fate evidence must be identity-indexed"
    );
    let observed_fates = indexed_fate_tags(&indexed_fates)
        .unwrap_or_else(|error| panic!("indexed fate evidence failed: {error}"));
    assert_eq!(
        observed_fates, persisted_fates,
        "recovery fates must agree with independently parsed persisted evidence"
    );
    let mut identities = BTreeSet::new();
    let mut indexed_counts = [0_u64; 4];
    for indexed in &indexed_fates {
        assert!(
            identities.insert(indexed.idempotency),
            "production recovery emitted a duplicate fate identity"
        );
        indexed_counts[fate_bucket(&indexed.fate)] += 1;
    }
    assert_eq!(indexed_counts[0], fates.acknowledged);
    assert_eq!(indexed_counts[1], fates.durable_unacknowledged);
    assert_eq!(indexed_counts[2], fates.proven_no_effect);
    assert_eq!(indexed_counts[3], fates.indeterminate);
    let encoded = std::fs::read(&report_path).expect("runtime report bytes");
    let report = RecoveryReportEnvelope::decode(&encoded).expect("runtime report decode");
    RuntimeProcess {
        process_id,
        marker,
        fates,
        indexed_fates,
        report,
        encoded,
    }
}

fn fate_bucket(fate: &str) -> usize {
    match fate {
        "AcknowledgedDurable" => 0,
        "DurableUnacknowledged" => 1,
        "ProvenNoEffect" => 2,
        "Indeterminate" => 3,
        other => panic!("production recovery emitted an unknown fate {other}"),
    }
}
