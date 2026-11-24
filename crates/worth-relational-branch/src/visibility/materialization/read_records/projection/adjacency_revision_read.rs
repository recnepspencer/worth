use crate::identity::data::{EntityId, KindId, VersionId};
use crate::storage::overlay::PartitionAccess;
use crate::storage::partition::AdjacencyDirection;

use super::VisibilityProjectionView;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalAdjacencyDirection {
    Outgoing,
    Incoming,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdjacencyStructuralRevision {
    revision: Option<VersionId>,
    work_units: usize,
}

/// Why a bounded structural-revision read gave no answer. Each cause is
/// distinct so a caller can tell an unpaid read from a missing anchor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdjacencyStructuralRevisionDenial {
    Execution(crate::execution::denial_cause::RelationalExecutionDenialCause),
    /// The caller's maximum cannot pay the one owner-index lookup. Nothing
    /// was read.
    WorkBudgetExceeded,
    /// The anchor is not visible at this snapshot.
    AnchorUnavailable,
    /// The snapshot's basis retains no adjacency index to read.
    BasisUnavailable,
}

impl AdjacencyStructuralRevision {
    pub const fn revision(self) -> Option<VersionId> {
        self.revision
    }

    pub const fn work_units(self) -> usize {
        self.work_units
    }
}

impl VisibilityProjectionView {
    pub fn bounded_adjacency_structural_revision(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        anchor: EntityId,
        relation_kind: KindId,
        direction: RelationalAdjacencyDirection,
        maximum_work: usize,
    ) -> Result<AdjacencyStructuralRevision, AdjacencyStructuralRevisionDenial> {
        if maximum_work < 1 {
            return Err(AdjacencyStructuralRevisionDenial::WorkBudgetExceeded);
        }
        execution
            .consult()
            .map_err(|stop| AdjacencyStructuralRevisionDenial::Execution(stop.into()))?;
        if self
            .entity_record_with_projection_scope(
                execution,
                anchor,
                super::ProjectionAspectScope::empty(),
                |_| Some(()),
            )
            .map_err(AdjacencyStructuralRevisionDenial::Execution)?
            .is_none()
        {
            return Err(AdjacencyStructuralRevisionDenial::AnchorUnavailable);
        }
        let direction = match direction {
            RelationalAdjacencyDirection::Outgoing => AdjacencyDirection::Outgoing,
            RelationalAdjacencyDirection::Incoming => AdjacencyDirection::Incoming,
        };
        let Some(root) = self.basis.root() else {
            return Err(AdjacencyStructuralRevisionDenial::BasisUnavailable);
        };
        let revision = if self.is_exact_basis() {
            self.admitted_head_region(anchor.partition_id)
                .and_then(|partition| partition.adjacency(direction).get(anchor.slot_index()))
                .and_then(|adjacency| adjacency.structural_revision(relation_kind))
        } else {
            root.get_partition(anchor.partition_id)
                .and_then(|partition| direction.table(partition).get(anchor.slot_index()))
                .and_then(|adjacency| adjacency.structural_revision(relation_kind))
        };
        Ok(AdjacencyStructuralRevision {
            revision,
            work_units: 1,
        })
    }
}
