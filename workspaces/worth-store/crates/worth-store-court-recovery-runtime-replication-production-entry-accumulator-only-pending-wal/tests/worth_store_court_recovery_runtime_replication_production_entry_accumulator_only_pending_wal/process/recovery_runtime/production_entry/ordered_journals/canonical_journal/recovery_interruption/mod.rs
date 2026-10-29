//! A recovery killed at each of its own stages leaves a store the next
//! recovery settles to the same journal: the interrupted publication rolls
//! forward exactly once, and nothing the first attempt did is repeated or lost.

#[allow(unused_imports)] use worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::recovery_interruption::*;
use {std::fs, std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};

use {worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint, worth_store::physical_runtime::PhysicalRecoveryYieldpointStage};
use {worth_store_recovery_runtime::WorthStoreRecovery};

use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::ENTRIES, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::Stop, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::assert_journal_holds, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::child, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::launch, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::payload, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::placement, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::publication, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::serve};

#[test]
fn a_recovery_killed_at_each_of_its_stages_settles_to_the_same_journal() {
    let mut reached_any = false;
    for stage in STAGES {
        let (root, _marker) = launch(worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::Stop::AfterWalDurability);
        reached_any |= kill_recovery_at(&root, stage);
        let serving = serve(&root, stage.label());
        assert_journal_holds(&serving, ENTRIES);
        publication(&serving, placement(), ENTRIES + 1, payload(ENTRIES + 1))
            .expect("publication after an interrupted recovery");
        serving.close();
        let serving = serve(&root, "recovery after the interrupted one");
        assert_journal_holds(&serving, ENTRIES + 1);
        serving.close();
    }
    assert!(reached_any, "no recovery stage was interrupted");
}

#[test]
fn recovery_interruption_child() {
    let Some(label) = std::env::var_os(RECOVERY_ROLE_ENV) else {
        return;
    };
    let stage = worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::from_label(label.to_str().unwrap())
        .expect("recovery stage label");
    let root = std::path::PathBuf::from(std::env::var_os(RECOVERY_ROOT_ENV).unwrap());
    let markers = std::path::PathBuf::from(std::env::var_os(RECOVERY_MARKERS_ENV).unwrap());
    let yieldpoint = worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint::new(
        stage,
        markers.join("reached"),
        markers.join("release"),
        markers.join("cancel"),
    );
    let _ = worth_store_recovery_runtime::WorthStoreRecovery::recover_with_process_yieldpoint(
        worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(&root),
        yieldpoint,
    );
    std::fs::write(markers.join("finished"), b"1").unwrap();
}
