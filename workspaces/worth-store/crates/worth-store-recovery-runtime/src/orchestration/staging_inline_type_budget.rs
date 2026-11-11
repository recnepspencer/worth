//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<RecoveryStagingCancellation>("workspaces/worth-store/crates/worth-store-recovery-runtime/src/orchestration/staging.rs", "RecoveryStagingCancellation", 128 * 1024);
    }
}
