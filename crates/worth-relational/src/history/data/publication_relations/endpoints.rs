//! Live endpoint identity is checked without adopting external entities.
use super::Stop;
use crate::identity::data::{EntityId, KindId};
use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::overlay::{entity_partition_source_for_slot, PartitionAccess};
use worth_execution::MapKernelContext;

pub(super) fn validate(
    state: &dyn PartitionAccess,
    registry: &RelationalSchemaRegistry,
    kind: KindId,
    source: EntityId,
    target: EntityId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    context
        .checkpoint(4)
        .map_err(|cause| Stop::Request(cause.into()))?;
    let live_kind = |id: EntityId| {
        entity_partition_source_for_slot(state, id.partition_id, id.slot_index())
            .and_then(|(_, partition)| partition.entity_arena.get(&id))
            .filter(|row| row.is_live())
            .and_then(|row| row.kind_id())
            .ok_or_else(|| {
                Stop::from("publication relation endpoint generation is absent or retired")
            })
    };
    let source_kind = live_kind(source)?;
    let target_kind = live_kind(target)?;
    registry
        .entity_registration(source_kind)
        .map_err(|_| "relation source kind absent")?;
    registry
        .entity_registration(target_kind)
        .map_err(|_| "relation target kind absent")?;
    let declaration = registry
        .relation_registration(kind)
        .map_err(|_| "relation kind absent")?;
    // SchemaControlled was resolved by genuine native mutation admission.
    // Association checks committed/candidate truth; it grants no new effect.
    if source.partition_id != target.partition_id
        && declaration.cross_context_policy == crate::config::data::CrossContextPolicy::Forbid
    {
        return Err("publication relation violates installed cross-context law".into());
    }
    for contract in &declaration.relation_integrity.endpoint_kind_contracts {
        let work = contract
            .allowed_source_kinds
            .len()
            .checked_add(contract.allowed_target_kinds.len())
            .and_then(|work| work.checked_add(3))
            .and_then(|work| u64::try_from(work).ok())
            .ok_or(Stop::Request(
                crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
            ))?;
        context
            .checkpoint(work)
            .map_err(|cause| Stop::Request(cause.into()))?;
        if !contract.allowed_source_kinds.contains(&source_kind)
            || !contract.allowed_target_kinds.contains(&target_kind)
            || (!contract.self_edges_allowed && source == target)
            || (source.partition_id != target.partition_id
                && contract.cross_context_policy
                    != crate::config::data::CrossContextPolicy::AllowExplicit)
        {
            return Err("publication relation endpoints differ from installed contract".into());
        }
    }
    Ok(())
}
