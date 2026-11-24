//! Original-request fixed metadata for one complete immutable adjacency bucket.
use super::{admit, borrowed_denial, consult, Denial, Direction};
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::storage::data::RecordLifecycleState as Lifecycle;
use crate::storage::partition::AdjacencyDirection;
use crate::visibility::materialization::read_records::projection::{
    RelationalAdjacencyVisit as Visit, RelationalRelationMetadata, VisibilityProjectionView,
};
use worth_execution::{
    ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder, MapKernelContext,
};

pub(super) fn read(
    view: &VisibilityProjectionView,
    execution: &ExecutionRequest<'_, '_>,
    anchor: EntityId,
    kind: KindId,
    direction: Direction,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<ExecutionArray<Option<RelationalRelationMetadata>>, Denial> {
    view.demand_regions_head(execution, [anchor.partition_id])
        .map_err(Denial::Execution)?;
    let native = match direction {
        Direction::Outgoing => AdjacencyDirection::Outgoing,
        Direction::Incoming => AdjacencyDirection::Incoming,
    };
    let count = if let Some(head) = view.admitted_head_region(anchor.partition_id) {
        let table = head.adjacency(native);
        admit(context, table.navigation_work_bound() as u64)?;
        if let Some(bucket) = table.get(anchor.slot_index()) {
            admit(context, bucket.navigation_work_bound() as u64)?;
            bucket.kind_ids(kind).len()
        } else {
            0
        }
    } else {
        0
    };
    // This is the actual immutable selected bucket extent, not a global count
    // estimate. The builder owns original-request admission/fallible storage.
    let mut rows =
        ExecutionRequestArrayBuilder::allocate(*execution, count).map_err(array_denial)?;
    let mut iterator_memory = None;
    view.try_visit_adjacency_ids(execution, anchor, kind, direction, |event| {
        consult(execution)?;
        match event {
            Visit::Prepare { work, bytes } => {
                admit(context, work)?;
                if bytes != 0 {
                    iterator_memory = execution
                        .reserve_retained_memory(bytes)
                        .map_err(|cause| Denial::Execution(cause.into()))?;
                }
            }
            Visit::Relation(relation) => {
                rows.push(live(
                    view, execution, relation, anchor, kind, direction, context,
                )?)
                .map_err(array_denial)?;
            }
            _ => {}
        }
        Ok::<_, Denial>(())
    })
    .map_err(Denial::Execution)??;
    drop(iterator_memory);
    rows.seal().map_err(array_denial)
}

fn live(
    view: &VisibilityProjectionView,
    execution: &ExecutionRequest<'_, '_>,
    relation: RelationId,
    anchor: EntityId,
    kind: KindId,
    direction: Direction,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Option<RelationalRelationMetadata>, Denial> {
    if relation.generation_value() == 0 {
        return Err(Denial::SourceUnavailable);
    }
    view.demand_regions_head(execution, [relation.partition_id])
        .map_err(Denial::Execution)?;
    let Some(head) = view.admitted_head_region(relation.partition_id) else {
        return Ok(None);
    };
    let Some(slot) = head
        .arena::<crate::storage::substrate::RelationRecordKind>()
        .get_slot(relation.slot_index())
    else {
        return Ok(None);
    };
    if slot.generation() != relation.generation_value() {
        return Ok(None);
    }
    match slot.lifecycle() {
        Lifecycle::RetainedDanglingForAudit => return Err(Denial::UnsupportedVisibility),
        Lifecycle::MaterializationUnavailable => return Err(Denial::SourceUnavailable),
        Lifecycle::Live => {}
        _ => return Ok(None),
    }
    admit(
        context,
        view.exact_relation_metadata_read_work_bound()
            .ok_or(Denial::ExactBasisRequired)?,
    )?;
    let metadata = view
        .read_exact_relation_metadata(execution, relation)
        .map_err(borrowed_denial)?
        .ok_or(Denial::SourceUnavailable)?;
    let endpoint = match direction {
        Direction::Outgoing => metadata.source,
        Direction::Incoming => metadata.target,
    };
    if metadata.kind_id != kind || endpoint != anchor {
        return Ok(None);
    }
    if metadata.source.generation_value() == 0 || metadata.target.generation_value() == 0 {
        return Err(Denial::EntityMismatch);
    }
    Ok(Some(metadata))
}

fn array_denial(cause: worth_execution::ExecutionRequestArrayDenial) -> Denial {
    match cause {
        worth_execution::ExecutionRequestArrayDenial::Request(cause) => {
            Denial::Execution(cause.into())
        }
        worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => {
            Denial::Allocation(cause)
        }
    }
}
