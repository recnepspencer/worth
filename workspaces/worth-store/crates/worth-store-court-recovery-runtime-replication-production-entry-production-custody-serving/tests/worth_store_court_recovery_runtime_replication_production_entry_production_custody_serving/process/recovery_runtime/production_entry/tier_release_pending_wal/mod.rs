//! A selected Tier checkpoint and process-killed V3 drop share one recovery suffix.

use worth_store_testkit_recovery_runtime::production_entry::tier_release_pending_wal::{
    produce_pending_world, run_tier_checkpoint_then_killed_released_wal, MARKER_ENV, ROLE_ENV,
};

#[test]
fn tier_checkpoint_then_killed_released_wal_reopens_once() {
    std::thread::Builder::new()
        .name("mixed-tier-release-recovery".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(run_tier_checkpoint_then_killed_released_wal)
        .unwrap()
        .join()
        .expect("mixed Tier/V3 recovery worker");
}

#[test]
fn mixed_tier_pending_child() {
    let Some(marker) = std::env::var_os(MARKER_ENV) else {
        return;
    };
    assert_eq!(std::env::var(ROLE_ENV).as_deref(), Ok("produce"));
    produce_pending_world(std::path::Path::new(&marker));
}
