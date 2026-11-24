//! Separate immutable descriptor DATA from actual current/external read HOLDS.
use super::*;
use worth_execution::ExecutionMemoryReservation;
#[derive(Debug)]
pub(in crate::internal_port::durability) struct CompleteComponentHolds {
    pub(super) _current: Vec<worth_relational_branch::internal_port::history::retention::obligation::RelationalObservationRetentionObligation>,
    pub(super) _external: worth_relational_branch::internal_port::history::retention::owner::component_capture::CapturedExternalCheckpointComponents,
    pub(super) _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug)]
struct Data {
    rows: Vec<(RelationalBranchBasisDescriptor, Arc<RelationalBranchRoot>)>,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug)]
pub struct CapturedCompleteComponents {
    data: Data,
    holds: CompleteComponentHolds,
}
impl CapturedCompleteComponents {
    pub(in crate::internal_port::durability) fn observations(
        &self,
    ) -> impl ExactSizeIterator<Item = (&RelationalBranchBasisDescriptor, &Arc<RelationalBranchRoot>)>
    {
        self.data
            .rows
            .iter()
            .map(|(descriptor, root)| (descriptor, root))
    }
    /// Consumers first admit their independent per-root/Empty copies. Moving
    /// the same holds then explicitly reclaims only this transient DATA owner.
    pub(in crate::internal_port::durability) fn into_holds(self) -> CompleteComponentHolds {
        let Self { data, holds } = self;
        drop(data);
        holds
    }
    pub(super) fn new(
        rows: Vec<(RelationalBranchBasisDescriptor, Arc<RelationalBranchRoot>)>,
        memory: Option<ExecutionMemoryReservation>,
        holds: CompleteComponentHolds,
    ) -> Self {
        Self {
            data: Data {
                rows,
                _memory: memory,
            },
            holds,
        }
    }
}
