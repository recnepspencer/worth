//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<SignalOwnerOperationBoundaryForTest>(
            "crates/worth-signal/src/branch/owner_services/operation_control.rs",
            "SignalOwnerOperationBoundaryForTest",
            128 * 1024,
        );
    }
}
