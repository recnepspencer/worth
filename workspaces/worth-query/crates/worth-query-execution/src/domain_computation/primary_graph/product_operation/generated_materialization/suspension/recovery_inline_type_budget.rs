//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryGeneratedOutputSuspensionRecovery>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/generated_materialization/suspension/recovery.rs", "WorthQueryGeneratedOutputSuspensionRecovery", 128 * 1024);
        owner.field::<RecoveryState>("state");
    }
}
