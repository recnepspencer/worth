//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<HeavyCaptureWitness>("crates/worth-signal/src/logic/transaction/runtime/state/runtime_state/branch_state_capture.rs", "HeavyCaptureWitness", 128 * 1024);
    }
}
