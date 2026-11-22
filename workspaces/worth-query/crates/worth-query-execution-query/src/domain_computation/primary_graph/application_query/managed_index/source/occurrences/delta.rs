//! The exact Source delta of one maintained update: a row entered, left, or
//! changed when its re-materialized entry differs from the retained one.
use super::super::projection::Entry;
use super::MaterialDenial;
use crate::domain_computation::primary_graph::application_query::ProjectionContentComparisonFault as Fault;
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
/// Storage grows only as rows are classified, each growth claimed on the
/// Source's budget first, so an empty delta keeps no capacity. A scope born
/// from its ordered predecessor has no committed side and never stores one.
pub struct SourceDelta {
    born: bool,
    committed: Vec<EntityId>,
    proposed: Vec<EntityId>,
}
impl SourceDelta {
    pub fn new(born: bool) -> Self {
        Self {
            born,
            committed: Vec::new(),
            proposed: Vec::new(),
        }
    }
    /// Classify one scheduled pair by the owner's own content equality.
    pub fn observe(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        scope: &ManagedIndexSourceScope<'_>,
        old: Option<&Entry>,
        new: Option<&Entry>,
    ) -> Result<(), MaterialDenial> {
        match (old, new) {
            (Some(old), Some(new)) => {
                if !same(execution, scope, old, new)? {
                    self.left(scope, old)?;
                    push(scope, &mut self.proposed, new.occurrence.entity())?;
                }
            }
            (Some(old), None) => self.left(scope, old)?,
            (None, Some(new)) => push(scope, &mut self.proposed, new.occurrence.entity())?,
            (None, None) => {}
        }
        Ok(())
    }
    fn left(
        &mut self,
        scope: &ManagedIndexSourceScope<'_>,
        old: &Entry,
    ) -> Result<(), MaterialDenial> {
        if self.born {
            return Ok(());
        }
        push(scope, &mut self.committed, old.occurrence.entity())
    }
    /// The delta in entity order; a born scope's is relative to the inherited
    /// step.
    pub fn finish(mut self) -> ManagedIndexSourceChanges {
        self.committed.sort_unstable();
        self.proposed.sort_unstable();
        // The admitted vectors move into shared ownership; nothing is copied.
        ManagedIndexSourceChanges::new(self.committed, self.proposed)
    }
}

impl super::PersistentSourceRows {
    /// A born scope's delta relative to its predecessor's membership
    /// `baseline`: each row absent from it or with different facts. Inherited
    /// unchanged rows are not lent, and a born scope has no committed side.
    /// One checkpoint and one comparison per built row.
    pub fn born_changes(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: &ManagedIndexSourceScope<'_>,
        baseline: &Self,
    ) -> Result<ManagedIndexSourceChanges, MaterialDenial> {
        let mut delta = SourceDelta::new(true);
        for row in self.keys() {
            scope
                .read_budget()
                .checkpoint(1)
                .map_err(DerivedRelationReadDenial::Structural)?;
            delta.observe(
                execution,
                scope,
                baseline.entry(row).map(|entry| &**entry),
                self.entry(row).map(|entry| &**entry),
            )?;
        }
        Ok(delta.finish())
    }
}

/// One classified row, its growth claimed before the vector grows: capacity
/// doubles, so the claims stay within twice the retained rows.
fn push(
    scope: &ManagedIndexSourceScope<'_>,
    side: &mut Vec<EntityId>,
    row: EntityId,
) -> Result<(), MaterialDenial> {
    if side.len() == side.capacity() {
        let more = side.capacity().max(4);
        let bytes = more
            .checked_mul(size_of::<EntityId>())
            .ok_or(RelationalExecutionDenialCause::AllocationFailed)?;
        scope
            .read_budget()
            .claim_retained(bytes as u64)
            .map_err(DerivedRelationReadDenial::Structural)?;
        side.try_reserve_exact(more)
            .map_err(|_| RelationalExecutionDenialCause::AllocationFailed)?;
    }
    side.push(row);
    Ok(())
}

fn same(
    execution: &ExecutionRequest<'_, '_>,
    scope: &ManagedIndexSourceScope<'_>,
    old: &Entry,
    new: &Entry,
) -> Result<bool, MaterialDenial> {
    let budget = scope.read_budget();
    old.same_facts(
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
