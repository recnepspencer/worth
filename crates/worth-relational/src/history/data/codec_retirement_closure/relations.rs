//! Authenticate incident relations after both endpoint entities are proven.
use super::{charge, memory_overflow, reserve, CodecRetirementClosureStop, OutputOrigin};
use crate::branch::RelationalBranchRoot;
use crate::identity::data::RelationId;
use crate::runtime::RelationalRuntime;
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease, MapKernelContext};

mod incident_boundary;

#[derive(Clone, Copy)]
pub(super) enum RelationClosurePurpose {
    CompletePublicationAssociation,
    CodecRetirement,
}

/// Complete cold association uses the same incident-edge court as retirement,
/// including when no member is selected for retirement.
pub(super) fn validate_complete_publication(
    runtime: &RelationalRuntime,
    root: &RelationalBranchRoot,
    origin: &std::sync::Arc<
        crate::history::data::canonical_commit_envelope::CanonicalRequiredOutputOrigin,
    >,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), CodecRetirementClosureStop> {
    charge(context, origin.members.len())?;
    let bytes = origin
        .members
        .len()
        .checked_mul(std::mem::size_of::<OutputOrigin>())
        .ok_or_else(memory_overflow)?;
    let _memory = reserve(lease, bytes)?;
    let origins = origin
        .members
        .iter()
        .map(|member| (member.id, member.kind, std::sync::Arc::clone(origin)))
        .collect::<Vec<_>>();
    let (relations, _relation_memory) = derive_closed_relations(
        runtime,
        root,
        &origins,
        RelationClosurePurpose::CompletePublicationAssociation,
        context,
        lease,
    )?;
    charge(
        context,
        relations.len().saturating_add(origin.relations.len()),
    )?;
    if !relations
        .iter()
        .copied()
        .eq(origin.relations.iter().map(|relation| relation.id))
    {
        return Err("incident relation roster differs from complete producer publication".into());
    }
    Ok(())
}

pub(super) fn derive_closed_relations(
    runtime: &RelationalRuntime,
    root: &RelationalBranchRoot,
    origins: &[OutputOrigin],
    purpose: RelationClosurePurpose,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(Vec<RelationId>, ExecutionMemoryReservation), CodecRetirementClosureStop> {
    let capacity = root.relation_slot_count();
    let bytes = capacity
        .checked_mul(std::mem::size_of::<RelationId>())
        .ok_or_else(memory_overflow)?;
    let memory = reserve(lease, bytes)?;
    let mut relations = Vec::with_capacity(capacity);
    for partition_id in root.partition_ids_iter() {
        charge(context, 1)?;
        let partition = root
            .partition_state(partition_id)
            .ok_or("native relation partition missing")?;
        for slot in partition.relation_arena.occupied_slots_iter() {
            charge(
                context,
                3 + 2 * (usize::BITS - origins.len().leading_zeros()) as usize,
            )?;
            let record = partition
                .relation_arena
                .get_slot(slot)
                .ok_or("native relation disappeared")?;
            if !record.is_live() {
                continue;
            }
            let endpoints = record
                .extra()
                .endpoints
                .as_ref()
                .ok_or("live native relation has no endpoints")?;
            let source = origins
                .binary_search_by_key(&endpoints.source, |(id, _, _)| *id)
                .ok();
            let target = origins
                .binary_search_by_key(&endpoints.target, |(id, _, _)| *id)
                .ok();
            if source.is_none() && target.is_none() {
                continue;
            }
            let (Some(source), Some(target)) = (source, target) else {
                return Err(incident_boundary::IncidentRelation {
                    id: RelationId::new(partition_id, slot as u64, record.generation()),
                    kind: record.kind_id(),
                    source: endpoints.source,
                    target: endpoints.target,
                }
                .outside_closure(purpose, (source, target), origins));
            };
            if origins[source].2.commit != origins[target].2.commit {
                return Err(
                    "incident relation endpoints have different native producer origins".into(),
                );
            }
            relations.push(RelationId::new(
                partition_id,
                slot as u64,
                record.generation(),
            ));
        }
    }
    for id in &relations {
        let partition = root
            .partition_state(id.partition_id)
            .ok_or("relation partition absent")?;
        let row = partition
            .relation_arena
            .get(id)
            .ok_or("relation generation absent")?;
        let ends = row
            .extra()
            .endpoints
            .as_ref()
            .ok_or("relation endpoints absent")?;
        let source = origins
            .binary_search_by_key(&ends.source, |(id, _, _)| *id)
            .map_err(|_| "relation source outside complete output closure")?;
        let envelope = runtime
            .history
            .canonical_envelope(origins[source].2.commit.commit_id)
            .ok_or("complete relation publication absent")?;
        super::relation_payload::validate(runtime, root, &envelope, *id, context, lease)?;
    }
    charge(
        context,
        relations
            .len()
            .checked_mul((usize::BITS - relations.len().leading_zeros()) as usize + 1)
            .ok_or_else(memory_overflow)?,
    )?;
    relations.sort_unstable();
    Ok((relations, memory))
}
