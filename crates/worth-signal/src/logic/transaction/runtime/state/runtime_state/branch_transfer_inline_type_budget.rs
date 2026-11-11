//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<ReadyBranchLifecycleTransfer<(), (), ()>>("crates/worth-signal/src/logic/transaction/runtime/state/runtime_state/branch_transfer.rs", "ReadyBranchLifecycleTransfer", 128 * 1024);
        owner.field::<BranchLifecycleTransfer<(), (), ()>>("transfer");
    }
}
