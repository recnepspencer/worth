//! Incident stored events from the actual aggregate row and exact selected root.
use super::cohorts::StoredAggregate;
use super::funding::{AggregateWork, BuildResult, PlanDenial};
use super::observations::Acquisition;
use super::record::{LineageRecord, LINEAGE_ASPECT};
use super::{retention_extent as extent, AggregateLineageEvent, AggregateLineageKind};
use crate::identity::data::RelationId;
use crate::schema::data::AggregateIdentityContractDeclaration;
use crate::validation::engine::state_view::InvariantStateView;
use std::collections::BTreeSet;
use worth_execution::MapKernelFailure;
use worth_foundational::facade::{AspectKey, ContractValidatedAspectValueView};

pub(super) fn read(
    view: &InvariantStateView<'_>,
    aggregate: &StoredAggregate,
    contract: &AggregateIdentityContractDeclaration,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<BTreeSet<AggregateLineageEvent>> {
    let mut events = BTreeSet::new();
    visit(view, aggregate, contract, work, |event, work| {
        work.retain(extent::tree_entry::<AggregateLineageEvent, ()>())?;
        events.insert(event);
        Ok(())
    })?;
    Ok(events)
}

#[derive(Clone, Copy)]
pub(super) struct MergeContinuation {
    pub(super) target: StoredAggregate,
    pub(super) event: AggregateLineageEvent,
}

pub(super) fn visit(
    view: &InvariantStateView<'_>,
    aggregate: &StoredAggregate,
    contract: &AggregateIdentityContractDeclaration,
    work: &mut AggregateWork<'_, '_, '_, '_>,
    mut emit: impl FnMut(AggregateLineageEvent, &mut AggregateWork<'_, '_, '_, '_>) -> BuildResult<()>,
) -> BuildResult<Option<MergeContinuation>> {
    let mut ids = BTreeSet::new();
    for outgoing in [true, false] {
        let candidates = view.relation_candidate_count(aggregate.entity, outgoing) as u64;
        work.checkpoint(
            candidates
                .checked_mul(8)
                .and_then(|n| n.checked_add(1))
                .ok_or(MapKernelFailure::ResultCapacityExceeded)?,
        )?;
        let mut acquired = || work.acquired(Acquisition::Relation);
        let selected = if outgoing {
            view.outgoing_relations_for_entity_observed(aggregate.entity, &mut acquired)?
        } else {
            view.incoming_relations_for_entity_observed(aggregate.entity, &mut acquired)?
        };
        work.acquired(Acquisition::Adjacency)?;
        ids.extend(selected);
    }
    let mut events = BTreeSet::new();
    let mut continuation = None;
    for id in ids {
        work.checkpoint(1)?;
        let Some(edge) = view.relation_metadata(id) else {
            continue;
        };
        work.acquired(Acquisition::Relation)?;
        if edge.kind_id != contract.storage.lineage_kind {
            continue;
        }
        let record = lineage_record(view, id, work)?;
        let from = super::stored_record::read_scalar(view, edge.source, contract, work)?;
        let to = super::stored_record::read_scalar(view, edge.target, contract, work)?;
        if from.identity == to.identity
            || (from.identity != aggregate.record.identity
                && to.identity != aggregate.record.identity)
            || (record.kind == AggregateLineageKind::MergedInto
                && from.retired != Some(record.occurrence))
        {
            return work.invalid(PlanDenial::InvalidStoredLineage);
        }
        let event = AggregateLineageEvent {
            occurrence: record.occurrence,
            kind: record.kind,
            from: from.identity,
            to: to.identity,
        };
        if !events.insert(event) {
            return work.invalid(PlanDenial::InvalidStoredLineage);
        }
        if event.kind == AggregateLineageKind::MergedInto && edge.source == aggregate.entity {
            if continuation.is_some() {
                return work.invalid(PlanDenial::InvalidStoredLineage);
            }
            continuation = Some(MergeContinuation {
                target: StoredAggregate {
                    entity: edge.target,
                    record: to,
                },
                event,
            });
        }
        emit(event, work)?;
    }
    Ok(continuation)
}

fn lineage_record(
    view: &InvariantStateView<'_>,
    relation: RelationId,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<LineageRecord> {
    let Some(state) = view.relation_aspect_state(relation) else {
        return work.invalid(PlanDenial::InvalidStoredLineage);
    };
    work.acquired(Acquisition::Aspect)?;
    let Some(aspect) = state.get(&AspectKey::new(LINEAGE_ASPECT).expect("fixed lineage aspect"))
    else {
        return work.invalid(PlanDenial::InvalidStoredLineage);
    };
    let ContractValidatedAspectValueView::Struct(fields) = aspect.view() else {
        return work.invalid(PlanDenial::InvalidStoredLineage);
    };
    LineageRecord::decode(|locator| fields.get(locator.field_path().fields().first()?))
        .ok_or(MapKernelFailure::Domain(PlanDenial::InvalidStoredLineage))
}
