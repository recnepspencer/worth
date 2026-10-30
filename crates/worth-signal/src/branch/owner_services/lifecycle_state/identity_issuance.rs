//! Non-reusable lifecycle identities, issued by the live Signal owner.
use super::SignalOwnerLifecycleIdentity;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SIGNAL_OWNER_LIFECYCLE_IDENTITY: AtomicU64 = AtomicU64::new(1);

pub(super) fn next_lifecycle_identity() -> SignalOwnerLifecycleIdentity {
    let identity = {
        let update_counter = &NEXT_SIGNAL_OWNER_LIFECYCLE_IDENTITY;
        let mut current = update_counter.load(Ordering::Relaxed);
        loop {
            let Some(updated_value) = current.checked_add(1) else {
                break Err(current);
            };
            match update_counter.compare_exchange_weak(
                current,
                updated_value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => current = reloaded_value,
            }
        }
    }
    .expect("Signal owner lifecycle identity exhausted");
    SignalOwnerLifecycleIdentity(identity)
}
