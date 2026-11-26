//! Cold native readmission of the exact present derived payload retirement set.

use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::canonical_commit_envelope::OutputOriginHistory;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::internal_port::branch::basis::AdmittedRelationalBranchBasisOwnerPort as _;
use crate::schema::codec_field_comparison::CodecFieldLocus;

pub mod authored_cascade;
pub mod authored_restoration;
pub mod authored_retirement;
pub mod coupled_selection;
pub mod historical_selection;
pub mod history_request;
pub use historical_selection::antecedent as historical_producer_antecedent;
pub mod scope_dependents;
pub mod source_scope_closure;
pub use scope_dependents::select_owned_scope_dependent;
pub use scope_dependents::OwnedScopeDependentSelection;
pub mod codec_disposition;
pub mod restoration_payload;
pub use authored_retirement::select_authored_retirement;
pub mod materialization;
pub mod membership;
pub use materialization::{
    capture_materialization_producers, prepare_materialization_continuity,
    write_matches as output_materialization_write_matches,
};
pub mod output_correspondence;
pub mod payload_reconstruction;
pub mod producer_retirement;
pub mod publication_failure;
pub use publication_failure::PublicationHistoryFailure;
pub mod publication_succession;
pub mod relation_payload;
pub mod relations;
pub mod route_selection;
pub use output_correspondence::{validate_codec_output_correspondence, SelectedOutputPublication};
pub use route_selection::readmit_required_output_route;

type OutputOrigin = (
    EntityId,
    KindId,
    std::sync::Arc<super::canonical_commit_envelope::CanonicalRequiredOutputOrigin>,
);

#[derive(Debug)]
pub struct OwnedCodecRetirementClosure {
    pub entities: Vec<EntityId>,
    pub relations: Vec<RelationId>,
    origins: Vec<OutputOrigin>,
    _memory: worth_execution::ExecutionMemoryReservation,
    _relation_memory: worth_execution::ExecutionMemoryReservation,
}

#[derive(Debug)]
pub enum CodecRetirementClosureStop {
    Request(crate::execution::denial_cause::RelationalExecutionDenialCause),
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
pub fn derive_codec_retirement_closure<'r>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: impl Into<
        crate::history::data::codec_retirement_closure::history_request::RetirementHistoryInputs<
            'r,
        >,
    >,
    basis: &AdmittedRelationalBranchBasis,
    changed_fields: &[CodecFieldLocus],
) -> Result<OwnedCodecRetirementClosure, CodecRetirementClosureStop> {
    let runtime = &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest::new(runtime, execution);
    if basis.descriptor().runtime_instance_id() != runtime.input.projection.runtime_identity
        || !basis.is_current()
    {
        return Err("codec provenance requires the current recovered native basis".into());
    }
    // Recovery-lane whole-root admission lives in this basis' transaction
    // residency, released with the basis rather than retained by the root.
    basis
        .root_read()
        .demand_all_full(execution, runtime.input.projection.symbols)
        .map_err(|failure| CodecRetirementClosureStop::Request(failure.into()))?;
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
    runtime: &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest<'_, '_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    changed_fields: &[CodecFieldLocus],
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<OwnedCodecRetirementClosure, CodecRetirementClosureStop> {
    charge(context, 1)?;
    if basis.descriptor().runtime_instance_id() != runtime.input.projection.runtime_identity
        || !basis.is_current()
    {
        return Err("codec provenance requires the current recovered native basis".into());
    }
    let root = &basis.root_read();
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
    let history_bound = runtime.input.projection.history.catalog_len();
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
            .partition_state(partition_id)?
            .ok_or("native partition disappeared")?;
        collect_affected_partition(partition, changed_fields, context, &mut affected)?;
    }
    let mut origin_frames = OutputOriginHistory::default();
    let validated_history = &mut origin_frames;
    for &id in &affected {
        let partition = root
            .partition_state(id.partition_id)?
            .ok_or("affected output partition missing")?;
        let row = partition
            .entity_arena
            .get(&id)
            .ok_or("affected output generation changed")?;
        let payload = row
            .extra()
            .authoritative_aspect_state()
            .ok_or("affected output payload missing")?;
        let selected = membership::latest(runtime, basis, id, validated_history, context, lease)?;
        if selected.authored_cascade_restoration.is_some()
            || selected.authored_cascade_retirement.is_some()
            || selected.producer_cascade_retirement.is_some()
        {
            return Err(
                "grouped authored publication requires distinct codec-transition custody".into(),
            );
        }
        let origin = membership::validate_member(
            runtime,
            basis,
            &selected,
            id,
            Some(payload),
            validated_history,
            context,
            lease,
        )?;
        let kind = row.kind_id().ok_or("affected native output kind missing")?;
        origins.push((id, kind, origin));
    }
    let sort_work = affected
        .len()
        .checked_mul((usize::BITS - affected.len().leading_zeros()) as usize + 1)
        .and_then(|work| work.checked_mul(2))
        .ok_or(CodecRetirementClosureStop::Request(
            crate::execution::denial_cause::RelationalExecutionDenialCause::WorkCounterOverflow,
        ))?;
    charge(context, sort_work)?;
    affected.sort_unstable();
    origins.sort_unstable_by_key(|(entity, _, _)| *entity);
    let (relations, relation_memory) = relations::derive_closed_relations(
        runtime,
        root,
        &origins,
        relations::RelationClosurePurpose::CodecRetirement,
        validated_history,
        context,
        lease,
    )?;
    Ok(OwnedCodecRetirementClosure {
        entities: affected,
        relations,
        origins,
        _memory: memory,
        _relation_memory: relation_memory,
    })
}

pub fn collect_affected_partition(
    partition: &crate::storage::overlay::PartitionState,
    changed_fields: &[CodecFieldLocus],
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    affected: &mut Vec<EntityId>,
) -> Result<(), CodecRetirementClosureStop> {
    let partition_id = partition.partition_id;
    for slot in partition.entity_arena.occupied_slots_iter() {
        charge(context, 1)?;
        let record = partition
            .entity_arena
            .get_slot(slot)
            .ok_or("native record disappeared")?;
        if !record.is_live() {
            continue;
        }
        let Some(payload) = record.extra().authoritative_aspect_state() else {
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
    Ok(())
}

fn intent_writes_entity(
    intent: &crate::transactions::data::intents::mutation_intent::MutationIntent,
    id: EntityId,
) -> bool {
    if matches!(
        intent,
        crate::transactions::data::intents::mutation_intent::MutationIntent::Entity(
            crate::transactions::data::intents::mutation_intent::EntityMutationIntent::Revalidate(
                _
            )
        )
    ) {
        return false;
    }
    intent.existing_record_target()
        == Some(crate::transactions::data::primitives::ExistingRecordTarget::Entity(id))
}

fn require_linear_history(
    envelope: &crate::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBody,
) -> Result<(), String> {
    if !envelope.is_unmerged_output_publication_step() {
        return Err("codec provenance does not admit merged physical output ownership".into());
    }
    Ok(())
}

pub fn charge(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    work: usize,
) -> Result<(), CodecRetirementClosureStop> {
    crate::execution::request_stage::checkpoint(
        context,
        u64::try_from(work).map_err(|_| {
            CodecRetirementClosureStop::Request(
                crate::execution::denial_cause::RelationalExecutionDenialCause::WorkCounterOverflow,
            )
        })?,
    )
    .map_err(request)
}

pub fn request(
    cause: impl Into<crate::execution::denial_cause::RelationalExecutionDenialCause>,
) -> CodecRetirementClosureStop {
    CodecRetirementClosureStop::Request(cause.into())
}

pub fn memory_overflow() -> CodecRetirementClosureStop {
    CodecRetirementClosureStop::Request(
        crate::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
    )
}

pub fn reserve(
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

impl From<crate::storage::data::RelationalStorageUnavailable> for CodecRetirementClosureStop {
    fn from(failure: crate::storage::data::RelationalStorageUnavailable) -> Self {
        Self::Request(
            crate::execution::denial_cause::RelationalExecutionDenialCause::StorageUnavailable(
                failure,
            ),
        )
    }
}

pub mod owned_publications;
