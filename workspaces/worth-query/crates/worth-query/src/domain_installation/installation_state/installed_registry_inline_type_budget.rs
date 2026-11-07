//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryDomainInstallationRegistry>("workspaces/worth-query/crates/worth-query/src/domain_installation/installation_state/installed_registry.rs", "WorthQueryDomainInstallationRegistry", 128 * 1024);
        owner.field::<WorthQueryRuntimeAuthorityIdentity>("runtime_authority");
        owner.field::<WorthQueryDomainInstallationGeneration>("generation");
        owner.field::<WorthQueryDomainInstallationGenerationLease>("generation_lease");
        owner.field::<Vec<WorthQueryInstalledDomainRecord>>("records");
        owner.field::<HashMap<TypeId, usize>>("by_marker_type");
        owner.field::<WorthQueryInstalledDomainExecutionIndex>("execution_index");
        owner.field::<Arc<worth_query_installation::facade::WorthQueryInstalledPackageIndex>>(
            "portable_index",
        );
        owner.field::<AtomicUsize>("handle_lookups");
    }
}
