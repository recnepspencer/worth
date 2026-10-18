//! Cold native readmission of the exact present derived payload retirement set.

use crate::branch::AdmittedRelationalBranchBasis;
use crate::history::data::CanonicalCommitEnvelope;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::runtime::{CodecFieldLocus, RelationalRuntime};
use crate::transactions::data::RecordRef;

mod relations;

type OutputOrigin = (
    EntityId,
    KindId,
    std::sync::Arc<super::canonical_commit_envelope::CanonicalRequiredOutputOrigin>,
);

#[derive(Debug)]
pub(crate) struct OwnedCodecRetirementClosure {
    pub(crate) entities: Vec<EntityId>,
    pub(crate) relations: Vec<RelationId>,
    origins: Vec<OutputOrigin>,
    _memory: worth_execution::ExecutionMemoryReservation,
    _relation_memory: worth_execution::ExecutionMemoryReservation,
}

impl OwnedCodecRetirementClosure {
    pub(crate) fn producers(&self) -> impl Iterator<Item = (EntityId, KindId, &str, &[u8; 32])> {
        self.origins.iter().map(|(id, kind, origin)| {
            (
                *id,
                *kind,
                origin.producer.as_str(),
                &origin.dependency_identity,
            )
        })
    }
}

#[derive(Debug)]
pub(crate) enum CodecRetirementClosureStop {
    Request(crate::execution::RelationalExecutionDenialCause),
    Payload(String),
}

impl From<String> for CodecRetirementClosureStop {
    fn from(value: String) -> Self {
        Self::Payload(value)
    }
}
impl From<&str> for CodecRetirementClosureStop {
    fn from(value: &str) -> Self {
        Self::Payload(value.into())
    }
}

/// Recovery lane: scans current records against changed loci, then affected
/// targets against retained linear ancestry and its actual operations. Vec cycle
/// detection costs O(ancestry²); memory is bounded by native records + catalog.
/// Missing history, merges and unknown writers fail closed.
pub(crate) fn derive_codec_retirement_closure(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    changed_fields: &[CodecFieldLocus],
) -> Result<OwnedCodecRetirementClosure, CodecRetirementClosureStop> {
    let result = execution
        .in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    derive_metered(runtime, basis, changed_fields, context, lease)
                })
        })
        .map_err(request)?
        .map_err(request)?
        .0;
    execution.consult().map_err(request)?;
    result
}

fn derive_metered(
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    changed_fields: &[CodecFieldLocus],
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<OwnedCodecRetirementClosure, CodecRetirementClosureStop> {
    charge(context, 1)?;
    if basis.descriptor().runtime_instance_id() != runtime.runtime_instance_id()
        || !basis.is_current()
    {
        return Err("codec provenance requires the current recovered native basis".into());
    }
    let root = &basis.inner.root;
    for locus in changed_fields {
        charge(context, locus.field_key.as_str().len())?;
        if !matches!(
            &locus.binding,
            worth_foundational::facade::AspectBinding::EntityField { .. }
        ) {
            return Err("codec provenance requires its native entity field carrier".into());
        }
    }
    let entity_bound = root.entity_slot_count();
    let history_bound = runtime.history.catalog_len();
    let entity_bytes = entity_bound
        .checked_mul(std::mem::size_of::<EntityId>())
        .ok_or_else(memory_overflow)?;
    let retained_bytes = entity_bytes
        .checked_add(
            entity_bound
                .checked_mul(std::mem::size_of::<(
                    EntityId,
                    KindId,
                    std::sync::Arc<super::canonical_commit_envelope::CanonicalRequiredOutputOrigin>,
                )>())
                .ok_or_else(memory_overflow)?,
        )
        .ok_or_else(memory_overflow)?;
    let scratch_bytes = entity_bytes
        .checked_add(
            history_bound
                .checked_mul(std::mem::size_of::<crate::history::data::CommitId>())
                .ok_or_else(memory_overflow)?,
        )
        .ok_or_else(memory_overflow)?;
    let memory = reserve(lease, retained_bytes)?;
    let _scratch_memory = reserve(lease, scratch_bytes)?;
    let mut affected = Vec::with_capacity(entity_bound);
    let mut origins = Vec::with_capacity(entity_bound);
    for partition_id in root.partition_ids_iter() {
        charge(context, 1)?;
        let partition = root
            .partition_state(partition_id)
            .ok_or("native partition disappeared")?;
        for slot in partition.entity_arena.occupied_slots_iter() {
            charge(context, 1)?;
            let record = partition
                .entity_arena
                .get_slot(slot)
                .ok_or("native record disappeared")?;
            if !record.is_live() {
                continue;
            }
            let Some(payload) = record.extra().authoritative_aspect_state.as_ref() else {
                continue;
            };
            let mut has_changed_present_aspect = false;
            for locus in changed_fields {
                charge(context, 1)?;
                if Some(locus.kind_id) != record.kind_id() {
                    continue;
                }
                for (key, _) in payload.aspects().entries() {
                    charge(
                        context,
                        key.as_str()
                            .len()
                            .saturating_add(locus.aspect_key.as_str().len())
                            .saturating_add(1),
                    )?;
                    if key == &locus.aspect_key {
                        has_changed_present_aspect = true;
                        break;
                    }
                }
                if has_changed_present_aspect {
                    break;
                }
            }
            // The enclosing aspect carries old contract identity/revision even
            // when the changed optional field itself has no present bytes.
            if has_changed_present_aspect {
                affected.push(EntityId::new(
                    partition_id,
                    slot as u64,
                    record.generation(),
                ));
            }
        }
    }
    let mut remaining = affected.clone();
    let mut visited = Vec::with_capacity(history_bound);
    let mut next = root.commit_id();
    while !remaining.is_empty() {
        charge(context, 1)?;
        let commit = next.ok_or("affected present field has no native producer history")?;
        charge(context, visited.len())?;
        if visited.len() >= history_bound || visited.contains(&commit) {
            return Err("codec provenance ancestry contains a cycle".into());
        }
        visited.push(commit);
        let envelope = runtime
            .history
            .canonical_envelope(commit)
            .ok_or("affected present field lost its canonical producer history")?;
        require_linear_history(&envelope)?;
        charge(
            context,
            envelope
                .patch
                .authoritative_record_patches
                .len()
                .saturating_add(envelope.merged_plan.merged_intents.len())
                .saturating_add(remaining.len()),
        )?;
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
            let touched = envelope
                .patch
                .authoritative_record_patches
                .iter()
                .any(|record| record.target == RecordRef::Entity(id))
                || envelope
                    .merged_plan
                    .merged_intents
                    .iter()
                    .any(|intent| intent_writes_entity(intent, id));
            if !touched {
                position += 1;
                continue;
            }
            let partition = root
                .partition_state(id.partition_id)
                .ok_or("affected output partition missing")?;
            let record = partition
                .entity_arena
                .get(&id)
                .ok_or("affected output generation changed")?;
            let payload = record
                .extra()
                .authoritative_aspect_state
                .as_ref()
                .ok_or("affected output payload missing")?;
            for (_, value) in payload.aspects().entries() {
                charge(context, value.semantic_byte_width())?;
            }
            // The newest real operation wins even if it supplied equal bytes.
            // An ordinary writer cannot inherit a predecessor's producer proof.
            envelope.readmit_required_output_payload(id, payload, context)?;
            let origin = envelope
                .required_output_origin
                .as_ref()
                .ok_or("admitted output lost its canonical producer origin")?;
            let kind = record
                .kind_id()
                .ok_or("admitted output lost native physical kind")?;
            origins.push((id, kind, std::sync::Arc::clone(origin)));
            remaining.swap_remove(position);
        }
        next = envelope.commit.parents.first().copied();
    }
    let sort_work = affected
        .len()
        .checked_mul((usize::BITS - affected.len().leading_zeros()) as usize + 1)
        .and_then(|work| work.checked_mul(2))
        .ok_or(CodecRetirementClosureStop::Request(
            crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
        ))?;
    charge(context, sort_work)?;
    affected.sort_unstable();
    origins.sort_unstable_by_key(|(entity, _, _)| *entity);
    let (relations, relation_memory) =
        relations::derive_closed_relations(runtime, root, &origins, context, lease)?;
    Ok(OwnedCodecRetirementClosure {
        entities: affected,
        relations,
        origins,
        _memory: memory,
        _relation_memory: relation_memory,
    })
}

fn intent_writes_entity(intent: &crate::transactions::data::MutationIntent, id: EntityId) -> bool {
    if matches!(
        intent,
        crate::transactions::data::MutationIntent::Entity(
            crate::transactions::data::EntityMutationIntent::Revalidate(_)
        )
    ) {
        return false;
    }
    intent.existing_record_target()
        == Some(crate::transactions::data::ExistingRecordTarget::Entity(id))
}

fn require_linear_history(envelope: &CanonicalCommitEnvelope) -> Result<(), String> {
    if envelope.commit.parents.len() > 1
        || envelope.merge_execution_authority.is_some()
        || !envelope.merge_parent_branches.is_empty()
        || !envelope.merge_base_commits.is_empty()
    {
        return Err("codec provenance does not admit merged physical output ownership".into());
    }
    Ok(())
}

pub(super) fn charge(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    work: usize,
) -> Result<(), CodecRetirementClosureStop> {
    context
        .checkpoint(u64::try_from(work).map_err(|_| {
            CodecRetirementClosureStop::Request(
                crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
            )
        })?)
        .map_err(request)
}

pub(super) fn request(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> CodecRetirementClosureStop {
    CodecRetirementClosureStop::Request(cause.into())
}

fn memory_overflow() -> CodecRetirementClosureStop {
    CodecRetirementClosureStop::Request(
        crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
    )
}

fn reserve(
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    bytes: usize,
) -> Result<worth_execution::ExecutionMemoryReservation, CodecRetirementClosureStop> {
    worth_execution::ExecutionMemoryReservation::reserve_in_scope(
        lease,
        u64::try_from(bytes).map_err(|_| memory_overflow())?,
    )
    .map_err(|denial| {
        CodecRetirementClosureStop::Request(crate::execution::denial::lease_denial(denial))
    })
}
