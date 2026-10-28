//! Actual candidate state adapters borrow the one original aggregate payer.
use super::funding::{AggregateWork, BuildResult, PlanDenial};
use super::observations::Acquisition;
use crate::derived_relation::evaluation::SourceRead;
use crate::derived_relation::{
    DerivedRelationAdjacency as Adjacency, DerivedRelationDenial as DefinitionDenial,
    DerivedRelationReadDenial as Denial, DerivedRelationReads as Reads,
};
use crate::identity::data::{EntityId, KindId};
use crate::validation::data::StructuralRelationRecord;
use crate::validation::engine::state_view::InvariantStateView;
use std::cell::RefCell;
use worth_execution::MapKernelFailure;

pub(super) fn with_views<T>(
    before: &InvariantStateView<'_>,
    after: &InvariantStateView<'_>,
    work: &mut AggregateWork<'_, '_, '_, '_>,
    read: impl FnOnce(
        &mut StateSource<'_, '_, '_, '_, '_, '_, '_>,
        &mut StateSource<'_, '_, '_, '_, '_, '_, '_>,
    ) -> Result<T, Denial>,
) -> BuildResult<T> {
    let work = RefCell::new(work);
    let failure = RefCell::new(None);
    let mut before = StateSource {
        view: before,
        work: &work,
        failure: &failure,
        reads: Reads::default(),
    };
    let mut after = StateSource {
        view: after,
        work: &work,
        failure: &failure,
        reads: Reads::default(),
    };
    let result = read(&mut before, &mut after);
    // The private callback cannot swallow a real original funding refusal.
    // No RefMut survives any SourceRead method or another adapter invocation.
    match failure.into_inner() {
        Some(cause) => Err(cause),
        None => result.map_err(|cause| MapKernelFailure::Domain(PlanDenial::DerivedRead(cause))),
    }
}

pub(super) struct StateSource<'view, 'state, 'borrow, 'work, 'scope, 'authority, 'memory> {
    view: &'view InvariantStateView<'state>,
    work: &'view RefCell<&'borrow mut AggregateWork<'work, 'scope, 'authority, 'memory>>,
    failure: &'view RefCell<Option<MapKernelFailure<PlanDenial>>>,
    reads: Reads,
}
impl StateSource<'_, '_, '_, '_, '_, '_, '_> {
    fn funded<T>(&self, result: BuildResult<T>) -> Result<T, Denial> {
        result.map_err(|cause| {
            let mut captured = self.failure.borrow_mut();
            if captured.is_none() {
                *captured = Some(cause);
            }
            // This sentinel never crosses with_views: it returns the captured
            // exact cause, preserving direct original stops and categories.
            DefinitionDenial::WorkBudgetExceeded.into()
        })
    }
    fn acquired(&self, kind: Acquisition) -> Result<(), Denial> {
        let result = self.work.borrow_mut().acquired(kind);
        self.funded(result)
    }
}
impl SourceRead for StateSource<'_, '_, '_, '_, '_, '_, '_> {
    fn checkpoint(&mut self, units: usize) -> Result<(), Denial> {
        let result = self.work.borrow_mut().checkpoint(units as u64);
        self.funded(result)
    }
    fn retain(&mut self, bytes: usize) -> Result<(), Denial> {
        let result = self.work.borrow_mut().retain(bytes as u64);
        self.funded(result)
    }
    fn entity_kind(&mut self, id: EntityId) -> Result<KindId, Denial> {
        self.checkpoint(1)?;
        let entity = self
            .view
            .entity_metadata(id)
            .ok_or(DefinitionDenial::RecordUnavailable)?;
        self.acquired(Acquisition::Entity)?;
        self.reads.entity_records = self
            .reads
            .entity_records
            .checked_add(1)
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
        Ok(entity.kind_id)
    }
    fn adjacency(&mut self, probe: Adjacency) -> Result<Vec<StructuralRelationRecord>, Denial> {
        let count = self
            .view
            .relation_candidate_count(probe.anchor, probe.outgoing);
        self.retain(
            count
                .checked_mul(size_of::<crate::identity::data::RelationId>())
                .and_then(|n| n.checked_add(size_of::<Vec<crate::identity::data::RelationId>>()))
                .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
        )?;
        self.checkpoint(
            count
                .checked_mul(8)
                .and_then(|n| n.checked_add(1))
                .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
        )?;
        let ids = if probe.outgoing {
            self.view
                .outgoing_relations_for_entity_observed(probe.anchor, || {
                    self.acquired(Acquisition::Relation)
                })?
        } else {
            self.view
                .incoming_relations_for_entity_observed(probe.anchor, || {
                    self.acquired(Acquisition::Relation)
                })?
        };
        self.acquired(Acquisition::Adjacency)?;
        self.reads.adjacency_lists = self
            .reads
            .adjacency_lists
            .checked_add(1)
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
        let mut records = Vec::new();
        for id in ids {
            self.checkpoint(1)?;
            let Some(record) = self.view.relation_metadata(id) else {
                continue;
            };
            self.acquired(Acquisition::Relation)?;
            self.reads.relation_records = self
                .reads
                .relation_records
                .checked_add(1)
                .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
            if record.kind_id == probe.relation {
                self.retain(size_of::<StructuralRelationRecord>())?;
                records.push(StructuralRelationRecord {
                    relation_id: id,
                    kind_id: record.kind_id,
                    source: record.source,
                    target: record.target,
                });
            }
        }
        Ok(records)
    }
    fn reads(&self) -> Reads {
        self.reads
    }
}
