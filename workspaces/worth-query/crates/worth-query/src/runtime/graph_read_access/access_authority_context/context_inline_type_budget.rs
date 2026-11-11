//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryGraphReadAccessAuthorityContext>("workspaces/worth-query/crates/worth-query/src/runtime/graph_read_access/access_authority_context/context.rs", "WorthQueryGraphReadAccessAuthorityContext", 128 * 1024);
        owner.field::<WorthQueryGraphReadAccessBasisScope>("basis_scope");
        owner.field::<Option<AdmittedPolicyTenantContext>>("policy_tenant");
        owner.field::<Option<RelationshipProofAdmission>>("relationship_proof");
        owner.field::<WorthQueryGraphReadAccessAuthorityReceipt>("receipt");
    }
}
