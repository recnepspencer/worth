//! Candidate-local endpoint disposition comes only from the two Native images.
use crate::validation::engine::state_view::InvariantStateView;
use worth_relational_contracts::internal_port::identity::data::record_ids::EntityId;

pub(super) enum EndpointRetirement {
    Current,
    RetiredByCandidate,
    Unavailable,
}

pub(super) fn classify(
    before: &InvariantStateView<'_>,
    after: &InvariantStateView<'_>,
    entity: EntityId,
) -> EndpointRetirement {
    if after.entity_metadata(entity).is_some() {
        EndpointRetirement::Current
    } else if before.entity_metadata(entity).is_some() {
        EndpointRetirement::RetiredByCandidate
    } else {
        EndpointRetirement::Unavailable
    }
}
