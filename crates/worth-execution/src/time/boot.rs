use std::sync::atomic::{AtomicU64, Ordering};

use worth_foundational::time::BootEpoch;

static NEXT_BOOT: AtomicU64 = AtomicU64::new(1);

/// Allocates a process-local identity shared by the built-in clock adapters.
#[allow(
    deprecated,
    reason = "fetch_update supports the workspace Rust 1.94 minimum"
)]
pub(super) fn next_boot() -> (BootEpoch, u64) {
    let id = NEXT_BOOT
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .expect("clock boot identities exhausted");
    (BootEpoch::new(id), id)
}
