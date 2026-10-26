//! Successful acquisitions at the aggregate reader's actual Native boundaries.
use super::funding::{BuildResult, PlanDenial};
use worth_execution::MapKernelFailure;

/// Native acquisitions, distinct from private work units and allocation bounds.
/// A failed probe costs work but does not acquire a record. These observations
/// grant no graph, result or execution authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AggregateReadObservations {
    pub adjacency_lists: usize,
    pub entity_records: usize,
    pub relation_records: usize,
    pub aspect_states: usize,
}

pub(super) enum Acquisition {
    Adjacency,
    Entity,
    Relation,
    Aspect,
}

impl AggregateReadObservations {
    pub(super) fn acquired(&mut self, acquisition: Acquisition) -> BuildResult<()> {
        let count = match acquisition {
            Acquisition::Adjacency => &mut self.adjacency_lists,
            Acquisition::Entity => &mut self.entity_records,
            Acquisition::Relation => &mut self.relation_records,
            Acquisition::Aspect => &mut self.aspect_states,
        };
        *count = count
            .checked_add(1)
            .ok_or(MapKernelFailure::Domain(PlanDenial::Execution(
                crate::execution::RelationalExecutionDenialCause::ResultCapacityExceeded,
            )))?;
        Ok(())
    }
}
