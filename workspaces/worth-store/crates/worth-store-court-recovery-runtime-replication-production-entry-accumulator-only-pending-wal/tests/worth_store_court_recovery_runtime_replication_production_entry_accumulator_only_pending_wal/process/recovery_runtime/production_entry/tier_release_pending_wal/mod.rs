#[test]
fn mixed_tier_pending_child() {
    use worth_store_testkit_recovery_runtime::production_entry::tier_release_pending_wal::{
        produce_pending_world, MARKER_ENV, ROLE_ENV,
    };
    let Some(marker) = std::env::var_os(MARKER_ENV) else {
        return;
    };
    assert_eq!(std::env::var(ROLE_ENV).as_deref(), Ok("produce"));
    produce_pending_world(std::path::Path::new(&marker));
}
