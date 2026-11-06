//! Borrowed completion phase: the brand keeps custody of its admission and inputs.
use super::*;
use crate::domain_computation::authorization::{
    ReadCompletionAdmission, WorthQueryAdmittedOperation,
};
use crate::domain_computation::primary_graph::vocabulary::application_attempt::fact::WorthQueryApplicationFactKey;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::scope_bindings::InstalledScopePosition;
use crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind;
use {
    crate::domain_computation::primary_graph::application_contribution::CompletedComputationRetention,
    crate::domain_computation::primary_graph::computation_retention::prior_absence::PriorAbsence,
    crate::domain_computation::primary_graph::invariant_projection::ExpectedFactKeys,
    crate::domain_computation::primary_graph::invariant_projection::ObservedComputationInputs,
    crate::domain_computation::primary_graph::value_decode::ValueDecodeState,
};

type ComputationReads = (ObservedComputationInputs, CompletedComputationRetention);

// All fields borrow the original attempt. On refusal its admission, decode
// grants, lease and remaining facts still drop in their original field order.
pub(super) struct MutationReadAttemptCore<'attempt, 'request, 'authority> {
    pub(super) execution: &'attempt worth_execution::ExecutionRequest<'request, 'authority>,
    pub(super) value_decode: &'attempt ValueDecodeState,
    pub(super) admission: ReadCompletionAdmission<'attempt>,
    pub(super) lease: &'attempt WorthQueryApplicationSnapshotLease,
    pub(super) expected_facts: &'attempt mut Option<ExpectedFactKeys>,
    pub(super) computation_reads: &'attempt mut ComputationReads,
    pub(super) installed_read_scopes:
        &'attempt mut BTreeMap<WorthQueryApplicationFactKey, InstalledScopePosition>,
    pub(super) facts:
        &'attempt mut BTreeMap<WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact>,
    pub(super) source_facts: &'attempt mut SourceFacts,
}

impl MutationReadAttemptCore<'_, '_, '_> {
    pub(super) fn take_computation_reads(&mut self) -> ComputationReads {
        std::mem::replace(
            self.computation_reads,
            (
                ObservedComputationInputs::None,
                CompletedComputationRetention::Absent(PriorAbsence::NotProduced),
            ),
        )
    }

    pub(super) fn check_completion_live(&self) -> Result<(), WorthQueryApplicationAttemptDenial> {
        check_completion_live(self.execution, self.value_decode, &self.admission)
    }
}

pub(super) fn check_completion_live(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    value_decode: &ValueDecodeState,
    admission: &WorthQueryAdmittedOperation,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    // An ignored decode refusal must never seal facts or advance the phase.
    if let Some(cause) = value_decode.denial() {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::ValueDecode(cause),
            admission.operation(),
        ));
    }
    execution.consult().map_err(|cause| {
        denial(
            WorthQueryApplicationAttemptDenialKind::ValueDecode(
                crate::domain_computation::vocabulary::primary_graph::value_decode::WorthQueryValueDecodeDenial::Scope(cause),
            ),
            admission.operation(),
        )
    })?;
    super::super::check_request_live(admission.publication_request(), admission.operation())
}
