#[allow(unused_imports)]
use crate::phase_eight_process::production::size_independence::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use worth_store_offline_verifier::RecoveryObserverReport;
use {
    worth_store_recovery_runtime::RecoveryReportCounters,
    worth_store_recovery_runtime::RecoveryReportOutcome,
};

use crate::history;
use {
    crate::phase_eight_process::production::harness::ProcessWorld,
    crate::phase_eight_process::production::harness::RuntimeProcess,
    crate::production::harness::markers::RecoveryFateMarker,
};

#[test]
fn recovery_work_is_independent_of_unrelated_persisted_store_size() {
    let world = crate::phase_eight_process::production::harness::ProcessWorld::start(
        "candidate-publication",
        0,
        1,
    );
    let small_root = world.writer.root.clone();
    let large_root = world.parent_path().join("size-independence-large-root");
    copy_persisted_store_root(&small_root, &large_root);

    let small_bytes = persisted_store_bytes(&small_root);
    let small_observer = world.observe_root(&small_root, "size-small-before-recovery");
    let small_fates =
        crate::phase_eight_process::history::persisted_fates::classify_persisted_fates(
            &world.writer.expected,
            &small_root,
        )
        .expect("small persisted fate oracle");

    let added_bytes = add_unrelated_residue_bytes(&large_root, small_bytes);
    let large_bytes = persisted_store_bytes(&large_root);
    assert!(
        large_bytes >= small_bytes.saturating_mul(MINIMUM_STORE_SIZE_MULTIPLIER),
        "paired fixture must substantially increase unrelated Store bytes: small={small_bytes}, large={large_bytes}, added={added_bytes}"
    );

    let large_observer = world.observe_root(&large_root, "size-large-before-recovery");
    let large_fates =
        crate::phase_eight_process::history::persisted_fates::classify_persisted_fates(
            &world.writer.expected,
            &large_root,
        )
        .expect("large persisted fate oracle");
    assert_eq!(
        small_fates, large_fates,
        "checkpoint/tail/damage scope changed"
    );
    assert_eq!(
        large_observer.report.residue_bytes(),
        small_observer
            .report
            .residue_bytes()
            .saturating_add(added_bytes),
        "independent observer did not see the planted persisted bytes as unrelated residue"
    );
    assert!(large_observer.report.bytes_read() > small_observer.report.bytes_read());
    assert_eq!(
        selected_recovery_basis(&small_observer.report),
        selected_recovery_basis(&large_observer.report),
        "selected checkpoint/tail basis changed with unrelated Store bytes"
    );

    let small_runtime = world.recover_root(&small_root, "size-small-recovery");
    let large_runtime = world.recover_root(&large_root, "size-large-recovery");
    assert_ne!(
        small_runtime.process_id, large_runtime.process_id,
        "paired recoveries must run in distinct physical_store_recover processes"
    );
    assert_ne!(small_runtime.marker.runtime, large_runtime.marker.runtime);
    assert_eq!(small_runtime.marker.store, large_runtime.marker.store);
    assert_eq!(
        small_runtime.marker.root_generation,
        large_runtime.marker.root_generation
    );
    assert_eq!(
        recovery_work(&small_runtime),
        recovery_work(&large_runtime),
        "recovery work evidence scaled with unrelated persisted Store bytes"
    );
}
