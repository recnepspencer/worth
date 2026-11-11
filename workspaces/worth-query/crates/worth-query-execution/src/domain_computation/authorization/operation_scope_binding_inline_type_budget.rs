//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryOperationScopeEntityBinding>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/authorization/operation_scope_binding.rs", "WorthQueryOperationScopeEntityBinding", 128 * 1024);
        owner.field::<u32>("partition_id");
        owner.field::<u64>("local_slot");
        owner.field::<u32>("generation");
    }
}
