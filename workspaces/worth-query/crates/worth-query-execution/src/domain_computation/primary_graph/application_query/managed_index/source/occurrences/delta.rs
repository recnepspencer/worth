//! The exact Source delta of one maintained update: a row entered, left, or
//! changed when its re-materialized entry differs from the retained one.
use super::super::projection::Entry;
use super::MaterialDenial;
use crate::domain_computation::primary_graph::application_query::ProjectionContentComparisonFault as Fault;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    content::RelationalContentDenial,
    identity::EntityId,
    runtime::{
        DerivedRelationReadDenial, ManagedIndexSourceChanges, ManagedIndexSourcePreparationDenial,
        ManagedIndexSourceScope,
    },
    transactions::RelationalExecutionDenialCause,
};

/// Committed-side rows left or changed; proposed-side rows entered or changed.
#[derive(Default)]
pub(super) struct SourceDelta {
    committed: Vec<EntityId>,
    proposed: Vec<EntityId>,
}
impl SourceDelta {
    /// Room for every scheduled pair, reserved before any row is classified.
    pub(super) fn with_capacity(
        scope: &ManagedIndexSourceScope<'_>,
        pairs: usize,
    ) -> Result<Self, MaterialDenial> {
        let bytes = pairs
            .checked_mul(2 * size_of::<EntityId>())
            .ok_or(RelationalExecutionDenialCause::AllocationFailed)?;
        scope
            .read_budget()
            .claim_retained(bytes as u64)
            .map_err(DerivedRelationReadDenial::Structural)?;
        let mut delta = Self::default();
        for side in [&mut delta.committed, &mut delta.proposed] {
            side.try_reserve_exact(pairs)
                .map_err(|_| RelationalExecutionDenialCause::AllocationFailed)?;
        }
        Ok(delta)
    }
    /// Classify one scheduled pair by the owner's own content equality.
    pub(super) fn observe(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        scope: &ManagedIndexSourceScope<'_>,
        old: Option<&Entry>,
        new: Option<&Entry>,
    ) -> Result<(), MaterialDenial> {
        match (old, new) {
            (Some(old), Some(new)) => {
                if !same(execution, scope, old, new)? {
                    self.committed.push(old.occurrence.entity());
                    self.proposed.push(new.occurrence.entity());
                }
            }
            (Some(old), None) => self.committed.push(old.occurrence.entity()),
            (None, Some(new)) => self.proposed.push(new.occurrence.entity()),
            (None, None) => {}
        }
        Ok(())
    }
    /// The delta in entity order. A scope born from its predecessor has no
    /// committed side: its delta is relative to the inherited step.
    pub(super) fn finish(mut self, born: bool) -> ManagedIndexSourceChanges {
        if born {
            self.committed.clear();
        }
        self.committed.sort_unstable();
        self.proposed.sort_unstable();
        ManagedIndexSourceChanges::new(Arc::from(self.committed), Arc::from(self.proposed))
    }
}

fn same(
    execution: &ExecutionRequest<'_, '_>,
    scope: &ManagedIndexSourceScope<'_>,
    old: &Entry,
    new: &Entry,
) -> Result<bool, MaterialDenial> {
    let budget = scope.read_budget();
    old.same_content(
        new,
        &mut |work| {
            budget
                .checkpoint(work)
                .map_err(|cause| DerivedRelationReadDenial::Structural(cause).into())
        },
        &mut |bytes| {
            execution
                .reserve_retained_memory(bytes)
                .map_err(|_| RelationalExecutionDenialCause::AllocationFailed.into())
        },
        &mut |fault| match fault {
            Fault::CapacityOverflow | Fault::AllocationFailed => {
                RelationalExecutionDenialCause::AllocationFailed.into()
            }
            Fault::ContentInconsistency => content(RelationalContentDenial::EncodingInconsistency),
        },
        &mut |left, right| left.same_content(execution, right).map_err(content),
    )
}
fn content(cause: RelationalContentDenial) -> MaterialDenial {
    match cause {
        RelationalContentDenial::Execution(cause) => MaterialDenial::Storage(cause),
        cause => MaterialDenial::Native(ManagedIndexSourcePreparationDenial::Content(cause)),
    }
}
