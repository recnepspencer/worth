//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<CanonicalAuthoredRestoration>("crates/worth-relational/src/history/data/canonical_commit_envelope/required_output_origin/authored_restoration.rs", "CanonicalAuthoredRestoration", 128 * 1024);
        owner.field::<CommitId>("retirement");
        owner.field::<EntityId>("original_anchor");
        owner.field::<EntityId>("anchor");
        owner.field::<Vec<(EntityId, EntityId)>>("entities");
        owner.field::<Vec<(
            crate::identity::data::RelationId,
            crate::identity::data::RelationId,
        )>>("relations");
        owner.field::<Vec<Arc<CanonicalRequiredOutputOrigin>>>("publications");
        owner.field::<Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>>(
            "_decoded_memory",
        );
    }
}
