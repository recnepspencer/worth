//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryGranularInvalidationInstallation>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/granular_invalidation/installation.rs", "WorthQueryGranularInvalidationInstallation", 128 * 1024);
        owner.field::<ApplicationSchemaBindingIdentity>("binding_identity");
        owner.field::<super::super::super::WorthQueryPrimaryGraphIntegrationHandle>("integration");
        owner.field::<Arc<GranularInvalidationRuntimeSeal>>("runtime_seal");
        owner.field::<u64>("generation");
        owner.field::<crate::domain_computation::execution_runtime::product_world::WorthQueryProductSharedRoot>("product_root");
    }
}
