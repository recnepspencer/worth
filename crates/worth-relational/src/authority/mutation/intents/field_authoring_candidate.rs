//! Original-request construction of field-authored portable candidates.
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::schema::data::LoweredAspectContractPlan;
use crate::transactions::data::{AspectFieldPatch, RecordAspectPatchDenial};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{PortableAspectPatchOperation, PortablePatchReadmissionPurpose};
mod allocation;
mod selection;
#[cfg(test)]
mod tests;
pub(super) use allocation::{admit_readmission, readmission_bytes, resize, retained_patch_bytes};

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
pub(crate) struct FieldCandidate {
    pub(super) operations: Vec<PortableAspectPatchOperation>,
    pub(super) funding: Option<ExecutionMemoryReservation>,
}
impl FieldCandidate {
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
    kind_id: crate::identity::data::KindId,
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
