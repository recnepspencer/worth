use super::*;
use crate::intent_admission::dx::{
    non_admitted_runtime_violation, WorthQueryRuntimeIntentAdmissionReviewData,
};
use crate::intent_admission::{
    WorthQueryAuthoritativeMutationExecutionBinding,
    WorthQueryAuthoritativeMutationExecutionHandoff, WorthQueryAuthoritativeMutationPreflight,
    WorthQueryIntentAdmissionDecision,
};
use crate::runtime::runtime_writes::WorthQueryWriteAdmissionExecutionRecord;

impl WorthQueryRuntime {
    pub fn write_intent(
        &mut self,
        command: WorthQueryWriteCommand,
    ) -> crate::intent_admission::WorthQueryRuntimeWriteIntentAuthoring<'_> {
        crate::intent_admission::WorthQueryRuntimeWriteIntentAuthoring::new(self, command)
    }

    pub(crate) fn review_authoritative_runtime_write(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        command: WorthQueryWriteCommand,
    ) -> Result<WorthQueryRuntimeIntentAdmissionReviewData, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Write)?;
        let seed = self.build_authoritative_mutation_intent_seed(request_execution, command);
        let request = crate::intent_admission::WorthQueryRawIntentAdmissionRequest::authoritative_write_entrypoint(seed)
            .map_err(|violation| {
                WorthQueryRuntimeError::Workspace(WorthQueryWorkspaceError::new(violation.message()))
            })?;
        Ok(WorthQueryRuntimeIntentAdmissionReviewData::from_request(
            request,
        ))
    }

    pub(crate) fn resolve_reviewed_admitted_authoritative_write_handoff(
        &self,
        review: WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> Result<WorthQueryAuthoritativeMutationExecutionHandoff, WorthQueryRuntimeError> {
        match review.decision().clone() {
            WorthQueryIntentAdmissionDecision::Admitted(
                crate::intent_admission::WorthQueryAdmittedIntentPlan::AuthoritativeMutation(plan),
            ) => Ok(WorthQueryAuthoritativeMutationExecutionHandoff::from_plan(
                plan,
            )),
            WorthQueryIntentAdmissionDecision::Admitted(_) => {
                Err(self.authoritative_write_non_admitted_error(&review))
            }
            WorthQueryIntentAdmissionDecision::Advisory(_)
            | WorthQueryIntentAdmissionDecision::Violation(_) => {
                Err(self.authoritative_write_non_admitted_error(&review))
            }
        }
    }

    pub(crate) fn authoritative_write_non_admitted_error(
        &self,
        review: &WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> WorthQueryRuntimeError {
        let seed = review
            .request()
            .authoritative_mutation_seed()
            .expect("scalar write review must preserve mutation seed");
        match seed.preflight() {
            WorthQueryAuthoritativeMutationPreflight::Admitted { .. } => {
                let violation = non_admitted_runtime_violation(review);
                WorthQueryRuntimeError::Workspace(WorthQueryWorkspaceError::new(
                    violation.message(),
                ))
            }
            WorthQueryAuthoritativeMutationPreflight::BindingDenied(denial) => {
                WorthQueryRuntimeError::MutationBindingDenied(denial.clone())
            }
            WorthQueryAuthoritativeMutationPreflight::AssertionDenied(denial) => {
                WorthQueryRuntimeError::ExistingTruthAssertionDenied(denial.clone())
            }
            WorthQueryAuthoritativeMutationPreflight::ContinuityDenied(denial) => {
                WorthQueryRuntimeError::MutationContinuityDenied(denial.clone())
            }
            WorthQueryAuthoritativeMutationPreflight::ContractDenied(denial) => {
                WorthQueryRuntimeError::MutationContractDenied(denial.clone())
            }
            WorthQueryAuthoritativeMutationPreflight::NamingDenied(denial) => {
                WorthQueryRuntimeError::MutationNamingDenied(denial.clone())
            }
            WorthQueryAuthoritativeMutationPreflight::TargetReferenceDenied(denial) => {
                WorthQueryRuntimeError::MutationTargetReferenceDenied(denial.clone())
            }
        }
    }

    pub(crate) fn prepare_authoritative_mutation_execution_binding(
        &self,
        handoff: WorthQueryAuthoritativeMutationExecutionHandoff,
    ) -> WorthQueryAuthoritativeMutationExecutionBinding {
        WorthQueryAuthoritativeMutationExecutionBinding::from_handoff(handoff)
    }

    pub(crate) fn execute_authoritative_mutation_execution_binding(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryAuthoritativeMutationExecutionBinding,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Write)?;
        let handoff = binding.handoff().clone();
        let shared_admission = WorthQueryWriteAdmissionExecutionRecord {
            family: binding.family(),
            entrypoint: binding.entrypoint(),
            execution_seam: binding.execution_seam(),
            request_detail: review_request_detail(handoff.command()),
            request_digest: handoff.request_digest().to_string(),
            eligibility_trace: handoff.eligibility_trace().clone(),
            decision_digest: handoff.decision_digest().to_string(),
            handoff_digest: handoff.handoff_digest().to_string(),
            binding_digest: binding.binding_digest().to_string(),
        };
        self.execute_authoritative_write_command_direct(
            execution,
            handoff.command().clone(),
            handoff.admitted_mutation().clone(),
            handoff.verified_existing_truth_assertion().cloned(),
            Some(shared_admission),
        )
    }
}

fn review_request_detail(command: &WorthQueryWriteCommand) -> String {
    match command.declared_entity_identity_ref() {
        Some(identity) => identity
            .evidence_identity()
            .reporting_projection()
            .to_string(),
        None => command
            .declared_collection_identity()
            .map(|collection| collection.as_str().to_string())
            .unwrap_or_else(|| "scalar-write".to_string()),
    }
}
