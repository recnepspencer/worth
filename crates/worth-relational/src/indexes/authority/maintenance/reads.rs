use super::work::MaintenanceWork;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;
use crate::runtime::VisibilityProjectionView;
use crate::storage::data::{EntityReadRecord, RecordLifecycleState, RelationReadRecord};
use std::collections::BTreeSet;

pub(super) fn entity(
    view: Option<&VisibilityProjectionView<'_>>,
    id: EntityId,
    work: &mut MaintenanceWork,
) -> Result<Option<EntityReadRecord>, Denial> {
    let Some(view) = view else {
        return Ok(None);
    };
    work.read()?;
    Ok(view
        .authoritative_entity_record(id)
        .filter(|record| record.lifecycle == RecordLifecycleState::Live))
}

pub(super) fn relation(
    view: Option<&VisibilityProjectionView<'_>>,
    id: RelationId,
    work: &mut MaintenanceWork,
) -> Result<Option<RelationReadRecord>, Denial> {
    let Some(view) = view else {
        return Ok(None);
    };
    work.read()?;
    // Full index builds include audit-retained relations but not retired ones.
    // A deleted endpoint alone does not remove field or ordering membership.
    Ok(view.authoritative_relation_record(id).filter(|record| {
        matches!(
            record.lifecycle,
            RecordLifecycleState::Live | RecordLifecycleState::RetainedDanglingForAudit
        )
    }))
}

pub(super) fn adjacency(
    view: &VisibilityProjectionView<'_>,
    entity: EntityId,
    kind: KindId,
    outgoing: bool,
    work: &mut MaintenanceWork,
) -> Result<Vec<RelationReadRecord>, Denial> {
    work.charge(1)?;
    let frontier = BTreeSet::from([entity]);
    let result =
        view.bounded_index_relations_for_frontier(&frontier, kind, outgoing, work.remaining());
    match result {
        Ok(read) => {
            let units = read.work_units();
            work.charge(units)?;
            work.counts.adjacency_work_units += units;
            Ok(read.into_records())
        }
        Err(exceeded) => {
            let units = exceeded.consumed_work_units();
            work.charge(units)?;
            work.counts.adjacency_work_units += units;
            Err(Denial::WorkBudgetExceeded)
        }
    }
}
