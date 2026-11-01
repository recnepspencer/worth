//! Controlled ownership ends only when the actual original patch is destroyed.
use super::super::field_authoring_candidate::{self as candidate, CandidateDenial, FieldCandidate};
use super::*;
use crate::execution::RelationalExecutionDenialCause as Cause;
#[derive(Debug)]
pub(crate) enum MutationPatchDenial {
    Request(Cause),
    Conflict(CommitConflict),
}
impl MutationPatchDenial {
    pub(crate) fn candidate(target: RecordAspectPatchTarget, denial: CandidateDenial) -> Self {
        match denial {
            CandidateDenial::Request(cause) => Self::Request(cause),
            CandidateDenial::Semantic(denial) => Self::Conflict(conflict(target, denial)),
        }
    }
}
impl From<Cause> for MutationPatchDenial {
    fn from(cause: Cause) -> Self {
        Self::Request(cause)
    }
}
impl From<MutationPatchDenial> for crate::transactions::data::TransactionCommitError {
    fn from(denial: MutationPatchDenial) -> Self {
        match denial {
            MutationPatchDenial::Request(cause) => Self::execution(cause.into()),
            MutationPatchDenial::Conflict(error) => Self::conflict(error),
        }
    }
}
impl From<MutationPatchDenial> for crate::mvcc::RelationalTransactionReadDenial {
    fn from(denial: MutationPatchDenial) -> Self {
        match denial {
            MutationPatchDenial::Request(cause) => Self::Request(cause),
            MutationPatchDenial::Conflict(error) => Self::Conflict(error),
        }
    }
}
pub(crate) fn readmit_candidate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mut input: FieldCandidate,
    purpose: PortablePatchReadmissionPurpose,
    plan: Option<&LoweredAspectContractPlan>,
    target: RecordAspectPatchTarget,
) -> Result<MutationAspectPatch, MutationPatchDenial> {
    // Declare the reservation before every owned input/output of this phase.
    let mut funding = input.funding.take();
    let operations = std::mem::take(&mut input.operations);
    let portable = PortableRecordAspectPatch::new(operations);
    candidate::admit_readmission(execution, &mut funding, &portable, plan)?;
    let patch =
        readmit_patch(portable, purpose, plan, target).map_err(MutationPatchDenial::Conflict)?;
    let retained = candidate::retained_patch_bytes(&patch)?;
    if funding
        .as_ref()
        .is_some_and(|memory| retained > memory.bytes())
    {
        return Err(Cause::ChargedBytesOverflow.into());
    }
    // The real readmitter consumed/dropped portable rosters and lookup scratch.
    candidate::resize(execution, &mut funding, retained)?;
    Ok(MutationAspectPatch::controlled(patch, funding))
}
