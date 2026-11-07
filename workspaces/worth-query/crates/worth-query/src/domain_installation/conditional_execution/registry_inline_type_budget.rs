//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryConditionalExecutionRegistry>("workspaces/worth-query/crates/worth-query/src/domain_installation/conditional_execution/registry.rs", "WorthQueryConditionalExecutionRegistry", 128 * 1024);
        owner.field::<Vec<AuthoritativeConditionalInstallation>>("authoritative");
        owner.field::<HashMap<ConditionalOperationKey, Vec<Arc<WorthQueryInstalledConditionalNode>>>>("by_operation");
    }
}
