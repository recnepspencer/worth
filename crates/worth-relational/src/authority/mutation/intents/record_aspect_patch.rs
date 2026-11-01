use worth_foundational::facade::{
    readmit_portable_record_aspect_patch, AuthoritativeRecordAspectPatch,
    AuthoritativeRecordAspectState, PortablePatchReadmissionPurpose, PortableRecordAspectPatch,
};
use worth_proof::TransitionOutcome;

use crate::schema::data::LoweredAspectContractPlan;
use crate::transactions::data::{
    CommitConflict, ConflictClass, RecordAspectPatchDenial, RecordAspectPatchTarget,
};

use super::super::outcomes::MutationAspectPatch;
mod controlled;
pub(crate) use controlled::{readmit_candidate, readmit_original_portable, MutationPatchDenial};

/// Existing ordinary portable input: its independent copy remains unqualified.
pub(crate) fn readmit(
    candidate: PortableRecordAspectPatch,
    purpose: PortablePatchReadmissionPurpose,
    plan: Option<&LoweredAspectContractPlan>,
    target: RecordAspectPatchTarget,
) -> Result<MutationAspectPatch, CommitConflict> {
    readmit_patch(candidate, purpose, plan, target).map(MutationAspectPatch::from_ordinary)
}

fn readmit_patch(
    candidate: PortableRecordAspectPatch,
    purpose: PortablePatchReadmissionPurpose,
    plan: Option<&LoweredAspectContractPlan>,
    target: RecordAspectPatchTarget,
) -> Result<AuthoritativeRecordAspectPatch, CommitConflict> {
    if candidate.is_empty() && plan.is_none() {
        return Ok(AuthoritativeRecordAspectPatch::empty());
    }
    let plan = plan.ok_or_else(|| {
        conflict(
            target,
            RecordAspectPatchDenial::MissingAspectPlan {
                kind_id: target_kind(target),
            },
        )
    })?;
    match readmit_portable_record_aspect_patch(candidate, purpose, plan) {
        TransitionOutcome::Success(patch) => Ok(patch),
        TransitionOutcome::Denied(denial) => Err(conflict(
            target,
            RecordAspectPatchDenial::ReadmissionDenied(denial),
        )),
    }
}

pub(crate) fn readmit_field_authoring(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    fields: &crate::transactions::data::AspectFieldPatch,
    purpose: PortablePatchReadmissionPurpose,
    plan: Option<&LoweredAspectContractPlan>,
    target: RecordAspectPatchTarget,
    domain: super::field_authoring_candidate::FieldAuthoringDomain,
) -> Result<MutationAspectPatch, MutationPatchDenial> {
    let candidate = super::field_authoring_candidate::lower(
        execution,
        fields,
        purpose,
        plan,
        target_kind(target),
        domain,
    )
    .map_err(|denial| MutationPatchDenial::candidate(target, denial))?;
    readmit_candidate(execution, candidate, purpose, plan, target)
}

pub(crate) fn apply_entity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    current: Option<&AuthoritativeRecordAspectState>,
    patch: &MutationAspectPatch,
    target: RecordAspectPatchTarget,
) -> Result<crate::storage::substrate::EntityExtra, crate::transactions::data::TransactionCommitError>
{
    crate::storage::substrate::EntityExtra::apply_payload(execution, current, patch.as_patch())
        .map_err(|denial| payload_denial(target, denial))
}
pub(crate) fn apply_relation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    current: Option<&AuthoritativeRecordAspectState>,
    patch: &MutationAspectPatch,
    target: RecordAspectPatchTarget,
) -> Result<
    crate::storage::substrate::RelationExtra,
    crate::transactions::data::TransactionCommitError,
> {
    crate::storage::substrate::RelationExtra::apply_payload(execution, current, patch.as_patch())
        .map_err(|denial| payload_denial(target, denial))
}
pub(crate) fn payload_denial(
    target: RecordAspectPatchTarget,
    denial: crate::storage::substrate::ExtraPayloadDenial,
) -> crate::transactions::data::TransactionCommitError {
    match denial {
        crate::storage::substrate::ExtraPayloadDenial::Request(cause) => {
            crate::transactions::data::TransactionCommitError::execution(cause.into())
        }
        crate::storage::substrate::ExtraPayloadDenial::Application(denial) => {
            crate::transactions::data::TransactionCommitError::conflict(conflict(
                target,
                RecordAspectPatchDenial::ApplicationDenied(denial),
            ))
        }
    }
}

pub(super) fn published_patch(patch: MutationAspectPatch) -> Option<MutationAspectPatch> {
    (!patch.as_patch().is_empty()).then_some(patch)
}

fn target_kind(target: RecordAspectPatchTarget) -> crate::identity::data::KindId {
    match target {
        RecordAspectPatchTarget::EntityCreation { kind_id }
        | RecordAspectPatchTarget::RelationCreation { kind_id }
        | RecordAspectPatchTarget::Entity { kind_id, .. }
        | RecordAspectPatchTarget::Relation { kind_id, .. } => kind_id,
    }
}

pub(crate) fn conflict(
    target: RecordAspectPatchTarget,
    denial: RecordAspectPatchDenial,
) -> CommitConflict {
    CommitConflict::new(ConflictClass::RecordAspectPatchDenied { target, denial })
}
