//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(
    census: &mut worth_relational_contracts::internal_port::inline_type_sizes::census::Census,
) {
    {
        let mut owner = census.owner::<RelationalInstalledApplicationPublicationAuthority>(
            "crates/worth-relational/src/mvcc/publication/application_publication.rs",
            "RelationalInstalledApplicationPublicationAuthority",
            128 * 1024,
        );
        owner.field::<u64>("runtime_instance_id");
        owner.field::<u64>("custom_invariant_generation");
        owner.field::<[u8; 32]>("schema_digest");
        owner.field::<worth_relational_contracts::internal_port::schema::installed_source_scopes::InstalledSourceScopeOwnership>(
            "source_scope_ownership",
        );
        owner.field::<worth_proof::InstalledApplicationPublicationAuthority>("authority");
    }
    crate::publication::ownership::authored_restoration::inline_type_budget::collect(census);
}
