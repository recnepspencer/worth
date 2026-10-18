//! Authenticate incident relations after both endpoint entities are proven.
use super::{charge, memory_overflow, reserve, CodecRetirementClosureStop, OutputOrigin};
use crate::branch::RelationalBranchRoot;
use crate::identity::data::RelationId;
use crate::runtime::RelationalRuntime;
use crate::transactions::data::{ExistingRecordTarget, RecordRef};
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease, MapKernelContext};

pub(super) fn derive_closed_relations(
    runtime: &RelationalRuntime,
    root: &RelationalBranchRoot,
    origins: &[OutputOrigin],
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(Vec<RelationId>, ExecutionMemoryReservation), CodecRetirementClosureStop> {
    let capacity = root.relation_slot_count();
    let history_bound = runtime.history.catalog_len();
    let bytes = capacity
        .checked_mul(std::mem::size_of::<RelationId>())
        .ok_or_else(memory_overflow)?;
    let memory = reserve(lease, bytes)?;
    let scratch = bytes
        .checked_add(
            history_bound
                .checked_mul(std::mem::size_of::<crate::history::data::CommitId>())
                .ok_or_else(memory_overflow)?,
        )
        .ok_or_else(memory_overflow)?;
    let _scratch = reserve(lease, scratch)?;
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
                return Err("codec retirement has unsupported incident relation consequences outside its proven output closure".into());
            };
            if !std::sync::Arc::ptr_eq(&origins[source].2, &origins[target].2) {
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
    let mut remaining = relations.clone();
    let mut visited = Vec::with_capacity(history_bound);
    let mut next = root.commit_id();
    while !remaining.is_empty() {
        charge(context, 1 + visited.len())?;
        let commit = next.ok_or("incident relation lost its canonical producer history")?;
        if visited.len() >= history_bound || visited.contains(&commit) {
            return Err("codec relation ancestry contains a cycle".into());
        }
        visited.push(commit);
        let envelope = runtime
            .history
            .canonical_envelope(commit)
            .ok_or("incident relation lost its canonical producer history")?;
        super::require_linear_history(&envelope)?;
        let mut position = 0;
        while position < remaining.len() {
            let id = remaining[position];
            charge(
                context,
                envelope
                    .patch
                    .authoritative_record_patches
                    .len()
                    .saturating_add(envelope.merged_plan.merged_intents.len()),
            )?;
            if !envelope
                .patch
                .authoritative_record_patches
                .iter()
                .any(|patch| patch.target == RecordRef::Relation(id))
                && !envelope.merged_plan.merged_intents.iter().any(|intent| {
                    intent.existing_record_target() == Some(ExistingRecordTarget::Relation(id))
                })
            {
                position += 1;
                continue;
            }
            let record = root
                .partition_state(id.partition_id)
                .and_then(|partition| partition.relation_arena.get(&id))
                .ok_or("incident relation generation changed")?;
            let endpoints = record
                .extra()
                .endpoints
                .as_ref()
                .ok_or("incident relation lost endpoints")?;
            charge(
                context,
                1 + (usize::BITS - origins.len().leading_zeros()) as usize,
            )?;
            let source = origins
                .binary_search_by_key(&endpoints.source, |(id, _, _)| *id)
                .map_err(|_| "incident relation source is outside the proven closure")?;
            let origin = envelope
                .required_output_origin
                .as_ref()
                .ok_or("incident relation latest write is not its installed producer create")?;
            if !std::sync::Arc::ptr_eq(origin, &origins[source].2) {
                return Err(
                    "incident relation and endpoints do not share one performed output origin"
                        .into(),
                );
            }
            let kind = record
                .kind_id()
                .ok_or("incident relation has no physical kind")?;
            if let Some(payload) = &record.extra().authoritative_aspect_state {
                for (_, value) in payload.aspects().entries() {
                    charge(context, value.semantic_byte_width())?;
                }
            }
            envelope.readmit_required_output_relation(
                id,
                kind,
                endpoints.source,
                endpoints.target,
                record.extra().authoritative_aspect_state.as_ref(),
                context,
            )?;
            remaining.swap_remove(position);
        }
        next = envelope.commit.parents.first().copied();
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
