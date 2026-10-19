//! Check committed reference identity against the exact root's installed endpoint laws.
use super::{charge, CodecRetirementClosureStop as Stop};
use crate::branch::RelationalBranchRoot;
use crate::config::data::CrossContextPolicy;
use crate::identity::data::{EntityId, KindId, RelationId};
use worth_execution::MapKernelContext;

pub(super) fn validate(
    root: &RelationalBranchRoot,
    id: RelationId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    charge(context, 4)?;
    let partition = root
        .partition_state(id.partition_id)
        .ok_or("incoming reference partition absent")?;
    let row = partition
        .relation_arena
        .get(&id)
        .filter(|row| row.is_live())
        .ok_or("incoming reference identity absent or retired")?;
    let kind = row.kind_id().ok_or("incoming reference kind absent")?;
    let ends = row
        .extra()
        .endpoints
        .as_ref()
        .ok_or("incoming reference endpoints absent")?;
    let registry = root.schema_authority().registry();
    let installed = registry
        .relation_registration(kind)
        .map_err(|_| Stop::from("incoming reference kind is not installed"))?;
    if root.schema_authority().relation_aspect_plan(kind).is_none()
        || partition
            .relation_arena
            .metadata_history_at(id.slot_index())
            .is_none_or(|history| history.len() == 0)
    {
        return Err(
            "incoming reference lacks installed native contract or committed position".into(),
        );
    }
    let source = endpoint_kind(root, ends.source)?;
    let target = endpoint_kind(root, ends.target)?;
    if ends.source.partition_id != ends.target.partition_id
        && installed.cross_context_policy != CrossContextPolicy::AllowExplicit
    {
        return Err("incoming reference violates installed cross-context law".into());
    }
    for contract in &installed.relation_integrity.endpoint_kind_contracts {
        charge(
            context,
            1 + contract.allowed_source_kinds.len() + contract.allowed_target_kinds.len(),
        )?;
        if !contract.allowed_source_kinds.contains(&source)
            || !contract.allowed_target_kinds.contains(&target)
            || (!contract.self_edges_allowed && ends.source == ends.target)
            || (ends.source.partition_id != ends.target.partition_id
                && contract.cross_context_policy != CrossContextPolicy::AllowExplicit)
        {
            return Err("incoming reference violates installed endpoint contract".into());
        }
    }
    // The row is already committed native truth. This check grants neither producer
    // ownership nor permission to change/delete it or its external source entity.
    Ok(())
}

fn endpoint_kind(root: &RelationalBranchRoot, id: EntityId) -> Result<KindId, Stop> {
    let row = root
        .partition_state(id.partition_id)
        .and_then(|partition| partition.entity_arena.get(&id))
        .filter(|row| row.is_live())
        .ok_or("incoming reference endpoint identity absent or retired")?;
    let kind = row
        .kind_id()
        .ok_or("incoming reference endpoint kind absent")?;
    root.schema_authority()
        .registry()
        .entity_registration(kind)
        .map_err(|_| Stop::from("incoming reference endpoint kind is not installed"))?;
    Ok(kind)
}
