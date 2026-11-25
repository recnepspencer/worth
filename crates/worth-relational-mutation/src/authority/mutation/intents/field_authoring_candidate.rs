//! Original-request construction of field-authored portable candidates.
use crate::contracts::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use crate::contracts::schema::data::aspect_contracts::lowered_plan::LoweredAspectContractPlan;
use crate::contracts::transactions::data::aspect_field_patch::AspectFieldPatch;
use crate::contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use crate::contracts::transactions::data::outcomes::conflict_class::record_aspect_patch_denial::RecordAspectPatchDenial;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{PortableAspectPatchOperation, PortablePatchReadmissionPurpose};
pub mod allocation;
pub mod selection;
#[cfg(test)]
pub mod tests;
pub(super) use allocation::{admit_readmission, resize, retained_patch_bytes};

#[derive(Clone, Copy)]
pub(crate) enum FieldAuthoringDomain {
    Entity,
    Relation,
}
#[derive(Debug)]
pub(crate) enum CandidateDenial {
    Request(Cause),
    Semantic(RecordAspectPatchDenial),
}
impl From<Cause> for CandidateDenial {
    fn from(cause: Cause) -> Self {
        Self::Request(cause)
    }
}
impl From<RecordAspectPatchDenial> for CandidateDenial {
    fn from(denial: RecordAspectPatchDenial) -> Self {
        Self::Semantic(denial)
    }
}
/// Short-lived builder: endpoint augmentation moves these same operations.
pub struct FieldCandidate {
    pub(super) operations: Vec<PortableAspectPatchOperation>,
    pub(super) funding: Option<ExecutionMemoryReservation>,
}
impl FieldCandidate {
    pub fn operations(&self) -> &[PortableAspectPatchOperation] {
        &self.operations
    }

    /// This causal relation-prefix copy is admitted BEFORE any owned clone.
    pub(super) fn copy_portable(
        request: &ExecutionRequest<'_, '_>,
        source: &worth_foundational::facade::PortableRecordAspectPatch,
    ) -> Result<Self, Cause> {
        let bytes = source
            .clone_retained_storage_bytes()
            .ok_or(Cause::ChargedBytesOverflow)?;
        let funding = request.reserve_retained_memory(
            u64::try_from(bytes).map_err(|_| Cause::ChargedBytesOverflow)?,
        )?;
        Ok(Self {
            operations: source.operations().to_vec(),
            funding,
        })
    }
    pub(super) fn empty() -> Self {
        Self {
            operations: Vec::new(),
            funding: None,
        }
    }
}
pub(crate) fn lower(
    request: &ExecutionRequest<'_, '_>,
    fields: &AspectFieldPatch,
    purpose: PortablePatchReadmissionPurpose,
    plan: Option<&LoweredAspectContractPlan>,
    kind_id: crate::contracts::identity::data::primitives::KindId,
    domain: FieldAuthoringDomain,
) -> Result<FieldCandidate, CandidateDenial> {
    if fields.is_empty() {
        return match purpose {
            PortablePatchReadmissionPurpose::RecordCreation => Ok(FieldCandidate::empty()),
            _ => Err(RecordAspectPatchDenial::EmptyFieldAuthoringPatch.into()),
        };
    }
    let plan = plan.ok_or(RecordAspectPatchDenial::MissingAspectPlan { kind_id })?;
    let scratch = request
        .reserve_retained_memory(allocation::backing::<selection::Selected<'_>>(
            fields.len(),
        )?)
        .map_err(Cause::from)?;
    let rows = selection::select(fields, plan, domain)?;
    let (retained, transient) = allocation::input(&rows, purpose)?;
    let funding = request
        .reserve_retained_memory(retained)
        .map_err(Cause::from)?;
    let boxing = request
        .reserve_retained_memory(transient)
        .map_err(Cause::from)?;
    let operations = selection::build(&rows, purpose)?;
    drop((rows, boxing, scratch));
    Ok(FieldCandidate {
        operations,
        funding,
    })
}

/// Lower only the submitted fields, using kind metadata from the actual applied
/// candidate. This does not read or reconstruct the entity's aspect state.
pub fn lower_entity(
    execution: &ExecutionRequest<'_, '_>,
    intent: &crate::contracts::transactions::data::intents::mutation_intent::UpdateEntityFieldsIntent,
    state: &impl worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess,
    plans: &crate::contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog,
) -> Result<
    FieldCandidate,
    crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
>{
    use crate::contracts::transactions::data::outcomes::commit_conflict::CommitConflict;
    use crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass;
    use crate::contracts::transactions::data::outcomes::conflict_class::EntityFieldUpdateMissingState;
    use crate::contracts::transactions::data::outcomes::conflict_class::record_aspect_patch_denial::RecordAspectPatchTarget;
    let entity = intent.entity_id;
    super::super::stale_targets::ensure_entity_target_is_current(state, entity)?;
    let missing = |missing| {
        CommitConflict::new(ConflictClass::EntityFieldUpdateStateInconsistency {
            entity_id: entity,
            missing,
        })
    };
    let partition = state
        .get_current_partition(entity.partition_id)
        .ok_or_else(|| missing(EntityFieldUpdateMissingState::Partition))?;
    let slot = partition
        .entity_arena()
        .get_slot(entity.slot_index())
        .ok_or_else(|| missing(EntityFieldUpdateMissingState::Slot))?;
    let kind = slot
        .kind_id()
        .ok_or_else(|| missing(EntityFieldUpdateMissingState::KindId))?;
    lower(
        execution,
        &intent.fields,
        PortablePatchReadmissionPurpose::RecordMutation,
        plans.entity_plans.get(&kind),
        kind,
        FieldAuthoringDomain::Entity,
    )
    .map_err(|denial| {
        super::record_aspect_patch::MutationPatchDenial::candidate(
            RecordAspectPatchTarget::Entity {
                entity_id: entity,
                kind_id: kind,
            },
            denial,
        )
        .into()
    })
}
