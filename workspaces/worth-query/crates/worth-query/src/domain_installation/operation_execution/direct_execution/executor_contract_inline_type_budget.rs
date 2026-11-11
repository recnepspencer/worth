//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryWorkflowOperation>("workspaces/worth-query/crates/worth-query/src/domain_installation/operation_execution/direct_execution/executor_contract.rs", "WorthQueryWorkflowOperation", 128 * 1024);
    }
}
