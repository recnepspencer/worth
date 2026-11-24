//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(
    census: &mut worth_relational_contracts::internal_port::inline_type_sizes::census::Census,
) {
    {
        let mut owner = census.owner::<RelationalAuthoredRestoration>("crates/worth-relational/src/mvcc/publication/application_publication/authored_restoration.rs", "RelationalAuthoredRestoration", 128 * 1024);
        owner.field::<AdmittedRelationalBranchBasis>("before");
        owner.field::<Arc<CanonicalCommitEnvelope>>("retirement");
        owner.field::<CreatedEntityRef>("anchor");
        owner.field::<Vec<(EntityId, CreatedEntityRef)>>("entities");
        owner.field::<Vec<(RelationId, CreatedRelationRef)>>("relations");
        owner.field::<Vec<RelationalRequiredOutputPublication>>("publications");
        owner.field::<Vec<MutationIntent>>("inverse");
    }
}
