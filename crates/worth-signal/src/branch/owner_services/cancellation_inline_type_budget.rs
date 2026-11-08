//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalOwnerCancellationSource>(
            "crates/worth-signal/src/branch/owner_services/cancellation.rs",
            "SignalOwnerCancellationSource",
            128 * 1024,
        );
        owner.field::<Arc<AtomicBool>>("requested");
    }
    {
        let mut owner = census.owner::<SignalOwnerCancellationToken>(
            "crates/worth-signal/src/branch/owner_services/cancellation.rs",
            "SignalOwnerCancellationToken",
            128 * 1024,
        );
        owner.field::<Arc<AtomicBool>>("requested");
    }
}
