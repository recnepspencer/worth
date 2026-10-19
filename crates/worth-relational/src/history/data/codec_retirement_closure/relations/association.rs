//! Association authenticates owned edges without adopting independent incoming references.
use super::{charge, CodecRetirementClosureStop as Stop, OutputOrigin, RelationClosurePurpose};
use crate::branch::RelationalBranchRoot;
use crate::history::data::canonical_commit_envelope::CanonicalRequiredOutputOrigin;
use crate::identity::data::RelationId;
use crate::runtime::RelationalRuntime;
use std::sync::Arc;
use worth_execution::{ExecutionResourceLease, MapKernelContext};

pub(in super::super) fn validate_complete_publication(
    runtime: &RelationalRuntime,
    root: &RelationalBranchRoot,
    origin: &Arc<CanonicalRequiredOutputOrigin>,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), Stop> {
    charge(context, origin.members.len())?;
    let bytes = origin
        .members
        .len()
        .checked_mul(std::mem::size_of::<OutputOrigin>())
        .ok_or_else(super::memory_overflow)?;
    let _memory = super::reserve(lease, bytes)?;
    let origins = origin
        .members
        .iter()
        .map(|member| (member.id, member.kind, Arc::clone(origin)))
        .collect::<Vec<_>>();
    let envelope = runtime
        .history
        .canonical_envelope(origin.commit.commit_id)
        .ok_or("complete relation publication absent")?;
    charge(context, origin.relations.len())?;
    if origin
        .relations
        .windows(2)
        .any(|pair| pair[0].id >= pair[1].id)
    {
        return Err("incident relation roster is not distinct canonical membership".into());
    }
    // A claimed edge must pass provenance even when its current endpoints moved outside.
    for relation in &origin.relations {
        charge(
            context,
            1 + 2 * (usize::BITS - origins.len().leading_zeros()) as usize,
        )?;
        if origin.outputs.binary_search(&relation.source).is_err()
            || origin.outputs.binary_search(&relation.target).is_err()
        {
            return Err(
                "claimed incident relation endpoints are outside complete producer membership"
                    .into(),
            );
        }
        super::super::relation_payload::validate(
            runtime,
            root,
            &envelope,
            relation.id,
            context,
            lease,
        )?;
    }
    for partition_id in root.partition_ids_iter() {
        charge(context, 1)?;
        let partition = root
            .partition_state(partition_id)
            .ok_or("native relation partition missing")?;
        for slot in partition.relation_arena.occupied_slots_iter() {
            charge(
                context,
                3 + 2 * (usize::BITS - origins.len().leading_zeros()) as usize
                    + (usize::BITS - origin.relations.len().leading_zeros()) as usize,
            )?;
            let row = partition
                .relation_arena
                .get_slot(slot)
                .ok_or("native relation disappeared")?;
            if !row.is_live() {
                continue;
            }
            let ends = row
                .extra()
                .endpoints
                .as_ref()
                .ok_or("live native relation has no endpoints")?;
            let source = origins
                .binary_search_by_key(&ends.source, |(id, _, _)| *id)
                .ok();
            let target = origins
                .binary_search_by_key(&ends.target, |(id, _, _)| *id)
                .ok();
            if source.is_none() && target.is_none() {
                continue;
            }
            let id = RelationId::new(partition_id, slot as u64, row.generation());
            let claimed = origin
                .relations
                .binary_search_by_key(&id, |relation| relation.id)
                .is_ok();
            match (source, target) {
                (Some(_), Some(_)) if claimed => (),
                (None, Some(_)) if !claimed => {
                    super::incoming_reference::validate(root, id, context)?;
                }
                _ => {
                    return Err(super::incident_boundary::IncidentRelation {
                        id,
                        kind: row.kind_id(),
                        source: ends.source,
                        target: ends.target,
                    }
                    .outside_closure(
                        RelationClosurePurpose::CompletePublicationAssociation,
                        (source, target),
                        &origins,
                    ))
                }
            }
        }
    }
    Ok(())
}
