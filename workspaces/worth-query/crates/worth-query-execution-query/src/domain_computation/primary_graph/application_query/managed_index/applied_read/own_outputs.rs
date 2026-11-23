//! A successor round (PR C §1.5) probes a managed index at the applied
//! candidate, where Native's applied Source and Parent projection read every
//! record as it is now. A record the round's own chain wrote is read there as
//! the chain left it, not as it was before the chain: the view is incoherent.
//! The probe marks it and refuses before the decision sees a row, so the
//! round stops typed (`PriorEffectRead`), exactly as a traversal that reaches
//! a chain record does. A probe that names no chain record is independent.
use super::Denial;
use crate::domain_computation::primary_graph::{
    application_attempt::{
        fact::touch_key, WorthQueryApplicationObservedFact as Fact,
    },
    invariant_projection::read_basis::OwnOutputOverlay,
    source_facts::SourceFactsAllocation,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::identity::EntityId;

/// The probed scope itself is a record the chain wrote.
pub(super) fn require_scope(own: Option<&OwnOutputOverlay>, scope: EntityId) -> Result<(), Denial> {
    require(own, [scope])
}

/// Every Source and Parent coordinate the probe read, including the
/// coordinates of rows it did not select and of empty adjacencies.
pub(super) fn require_facts(
    execution: &ExecutionRequest<'_, '_>,
    own: Option<&OwnOutputOverlay>,
    facts: &[Fact],
) -> Result<(), Denial> {
    if own.is_none() {
        return Ok(());
    }
    for fact in facts {
        SourceFactsAllocation::work(execution, 1)?;
        match fact {
            Fact::SourceAdjacencyRevision { anchor, .. } | Fact::Adjacency { anchor, .. } => {
                require(own, [*anchor])?
            }
            Fact::Relation { from, to, .. } => require(own, [*from, *to])?,
            fact => require(own, touch_key::fact_record(fact))?,
        }
    }
    Ok(())
}

fn require(
    own: Option<&OwnOutputOverlay>,
    records: impl IntoIterator<Item = EntityId>,
) -> Result<(), Denial> {
    let Some(own) = own else {
        return Ok(());
    };
    for record in records {
        if own.covers(record) {
            own.mark_incoherent(record);
            return Err(Denial::PriorEffectRead);
        }
    }
    Ok(())
}
