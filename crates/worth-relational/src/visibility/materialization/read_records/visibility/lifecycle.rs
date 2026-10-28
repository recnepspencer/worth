use crate::identity::data::VersionId;
use crate::storage::data::RecordLifecycleState;

pub(in super::super) fn lifecycle_storage_visible(lifecycle: RecordLifecycleState) -> bool {
    lifecycle != RecordLifecycleState::Reusable
}

pub(in super::super) fn historical_lifecycle(
    retired_at: Option<VersionId>,
    version_id: VersionId,
) -> RecordLifecycleState {
    if retired_at.is_some_and(|retired_at| retired_at <= version_id) {
        RecordLifecycleState::DeletedRetained
    } else {
        RecordLifecycleState::Live
    }
}

pub(in super::super) fn historical_retired_at(
    retired_at: Option<VersionId>,
    version_id: VersionId,
) -> Option<VersionId> {
    retired_at.filter(|retired_at| *retired_at <= version_id)
}

/// A retained audit edge is still queryable after its historical retirement.
pub(in super::super) fn historical_relation_lifecycle(
    partition: &crate::storage::overlay::PartitionState,
    slot: usize,
    metadata: &crate::storage::substrate::VersionedRelationMetadata,
    version: VersionId,
) -> RecordLifecycleState {
    let retired = historical_retired_at(metadata.retired_at, version);
    if retired.is_some()
        && partition
            .relation_arena
            .get_slot(slot)
            .is_some_and(|current| {
                current.generation() == metadata.generation
                    && current.lifecycle() == RecordLifecycleState::RetainedDanglingForAudit
            })
    {
        RecordLifecycleState::RetainedDanglingForAudit
    } else {
        historical_lifecycle(retired, version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn historical_retirement_does_not_reveal_a_future_transition() {
        assert_eq!(
            historical_retired_at(Some(VersionId(9)), VersionId(8)),
            None
        );
        assert_eq!(
            historical_retired_at(Some(VersionId(9)), VersionId(9)),
            Some(VersionId(9))
        );
    }
}
