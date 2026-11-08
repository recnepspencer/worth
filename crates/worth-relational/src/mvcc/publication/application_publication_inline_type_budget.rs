//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<RelationalInstalledApplicationPublicationAuthority>(
            "crates/worth-relational/src/mvcc/publication/application_publication.rs",
            "RelationalInstalledApplicationPublicationAuthority",
            128 * 1024,
        );
        owner.field::<u64>("runtime_instance_id");
        owner.field::<u64>("custom_invariant_generation");
        owner.field::<[u8; 32]>("schema_digest");
        owner.field::<scope_ownership::InstalledSourceScopeOwnership>("source_scope_ownership");
        owner.field::<worth_proof::InstalledApplicationPublicationAuthority>("authority");
    }
    super::authored_restoration::inline_type_budget::collect(census);
}
