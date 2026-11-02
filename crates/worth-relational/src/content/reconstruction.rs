//! Complete visible-record traversal is explicit reconstructive work.
use crate::schema::data::RelationalSchemaRegistry;
use worth_foundational::facade::{AspectShape, ScalarAspectType};

/// Explicit reconstructive work, never used by an ordinary unchanged schema.
pub(super) fn visit_records(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &dyn crate::storage::overlay::CurrentPartitionAccess,
    callback: &mut dyn FnMut(
        &crate::transactions::data::RecordRef,
    ) -> Result<(), super::RelationalContentDenial>,
) -> Result<(), super::RelationalContentDenial> {
    use super::RelationalContentDenial as ContentDenial;
    use crate::identity::data::{EntityId, RelationId};
    use crate::transactions::data::RecordRef;
    super::control::run(execution, |control| {
        for partition_id in state.partition_ids_iter() {
            control.checkpoint(1)?;
            state
                .demand_head_partition(execution, partition_id)
                .map_err(|cause| match cause {
                    crate::storage::residency::RelationalResidencyDemandFailure::Content(cause) => {
                        cause
                    }
                    cause => ContentDenial::Execution(cause.into()),
                })?;
            let partition = state
                .get_current_partition(partition_id)
                .ok_or(ContentDenial::EncodingInconsistency)?;
            for slot in partition.entity_arena().live_slots() {
                control.checkpoint(1)?;
                let view = partition
                    .entity_arena()
                    .get_slot(slot)
                    .ok_or(ContentDenial::EncodingInconsistency)?;
                callback(&RecordRef::Entity(EntityId::new(
                    partition_id,
                    u64::try_from(slot).map_err(|_| ContentDenial::ArithmeticOverflow)?,
                    view.generation(),
                )))?;
            }
            for slot in partition.relation_arena().live_slots() {
                control.checkpoint(1)?;
                let view = partition
                    .relation_arena()
                    .get_slot(slot)
                    .ok_or(ContentDenial::EncodingInconsistency)?;
                callback(&RecordRef::Relation(RelationId::new(
                    partition_id,
                    u64::try_from(slot).map_err(|_| ContentDenial::ArithmeticOverflow)?,
                    view.generation(),
                )))?;
            }
        }
        Ok(())
    })
}

/// Current declarations and every carried contract jointly prove absence. Old
/// contracts can still decode stored values after a kind changes its declarations.
pub(super) fn schema_has_references(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    registry: &RelationalSchemaRegistry,
    retained: &[worth_foundational::facade::AspectContract],
) -> Result<bool, super::RelationalContentDenial> {
    super::control::run(execution, |control| {
        let kinds = registry
            .entity_kinds
            .values()
            .map(|kind| &kind.aspect_contract_declarations)
            .chain(
                registry
                    .relation_kinds
                    .values()
                    .map(|kind| &kind.aspect_contract_declarations),
            );
        for declarations in kinds {
            control.checkpoint(1)?;
            for aspect in &declarations.aspects {
                if contract_has_references(control, &aspect.contract)? {
                    return Ok(true);
                }
            }
        }
        for contract in retained {
            if contract_has_references(control, contract)? {
                return Ok(true);
            }
        }
        Ok(false)
    })
}

fn contract_has_references(
    control: &mut super::RelationalContentControl<'_, '_, '_>,
    contract: &worth_foundational::facade::AspectContract,
) -> Result<bool, super::RelationalContentDenial> {
    control.checkpoint(1)?;
    match contract.shape() {
        AspectShape::Content | AspectShape::Scalar(ScalarAspectType::ContentRef) => {
            return Ok(true)
        }
        AspectShape::Struct(shape) => {
            for field in shape.fields() {
                control.checkpoint(1)?;
                if field.value_type() == ScalarAspectType::ContentRef {
                    return Ok(true);
                }
            }
        }
        _ => {}
    }
    Ok(false)
}
