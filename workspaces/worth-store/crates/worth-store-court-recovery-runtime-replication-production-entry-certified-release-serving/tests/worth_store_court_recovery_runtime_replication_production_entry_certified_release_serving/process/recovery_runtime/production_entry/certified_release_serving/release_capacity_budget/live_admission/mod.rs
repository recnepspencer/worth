//! Real competing Recovery custody must stop the first release before effects.

#[allow(unused_imports)]
use crate::production_entry::certified_release_serving::release_capacity_budget::live_admission::*;

#[test]
fn competing_recovery_owner_denies_before_effects_then_partial_drop_reopens() {
    std::thread::Builder::new()
        .name("release-live-recovery-admission".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("live release admission worker")
        .join()
        .expect("live release admission worker did not panic");
}
