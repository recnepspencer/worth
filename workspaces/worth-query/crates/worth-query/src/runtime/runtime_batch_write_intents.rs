use super::*;
use crate::intent_admission::dx::non_admitted_runtime_violation;
use crate::intent_admission::{
    WorthQueryAuthoritativeMutationBatchExecutionBinding,
    WorthQueryAuthoritativeMutationBatchExecutionHandoff, WorthQueryIntentAdmissionDecision,
};
use crate::runtime::runtime_writes::WorthQueryWriteAdmissionExecutionRecord;

impl WorthQueryRuntime {
    pub fn write_batch_intent(
        &mut self,
        commands: Vec<WorthQueryWriteCommand>,
    ) -> crate::intent_admission::WorthQueryRuntimeWriteBatchIntentAuthoring<'_> {
        crate::intent_admission::WorthQueryRuntimeWriteBatchIntentAuthoring::new(self, commands)
    }

    pub(crate) fn review_authoritative_runtime_write_batch(
        &self,
        commands: Vec<WorthQueryWriteCommand>,
    ) -> Result<
        crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData,
        WorthQueryRuntimeError,
    > {
        self.review_authoritative_runtime_write_batch_with_graph_artifacts(
            commands,
            WorthQueryGraphCompositionBreadth::empty(),
            WorthQueryGraphCompositionProgram::empty(),
        )
    }

    pub(crate) fn review_authoritative_runtime_write_batch_with_graph_artifacts(
        &self,
        commands: Vec<WorthQueryWriteCommand>,
        graph_composition_breadth: WorthQueryGraphCompositionBreadth,
        graph_composition_program: WorthQueryGraphCompositionProgram,
    ) -> Result<
        crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData,
        WorthQueryRuntimeError,
    > {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Write)?;
        let seed = crate::intent_admission::WorthQueryAuthoritativeMutationBatchIntentSeed::new(
            commands,
            graph_composition_breadth,
            graph_composition_program,
        );
        let request = crate::intent_admission::WorthQueryRawIntentAdmissionRequest::authoritative_write_batch_entrypoint(seed)
            .map_err(|violation| WorthQueryRuntimeError::Workspace(WorthQueryWorkspaceError::new(violation.message())))?;
        Ok(
            crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData::from_request(
                request,
            ),
        )
    }

    pub(crate) fn resolve_reviewed_admitted_authoritative_write_batch_handoff(
        &self,
        review: crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> Result<WorthQueryAuthoritativeMutationBatchExecutionHandoff, WorthQueryRuntimeError> {
        admitted_authoritative_mutation_batch_handoff_from_review(review)
    }

    pub(crate) fn prepare_authoritative_mutation_batch_execution_binding(
        &self,
        handoff: WorthQueryAuthoritativeMutationBatchExecutionHandoff,
    ) -> WorthQueryAuthoritativeMutationBatchExecutionBinding {
        WorthQueryAuthoritativeMutationBatchExecutionBinding::from_handoff(handoff)
    }

    pub(crate) fn execute_authoritative_mutation_batch_execution_binding(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryAuthoritativeMutationBatchExecutionBinding,
    ) -> Result<WorthQueryBatchWriteReceipt, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Write)?;
        let handoff = binding.handoff().clone();
        let shared_admission = WorthQueryWriteAdmissionExecutionRecord {
            family: binding.family(),
            entrypoint: binding.entrypoint(),
            execution_seam: binding.execution_seam(),
            request_detail: format!("batch-write:{}", handoff.commands().len()),
            request_digest: handoff.request_digest().to_string(),
            eligibility_trace: handoff.eligibility_trace().clone(),
            decision_digest: handoff.decision_digest().to_string(),
            handoff_digest: handoff.handoff_digest().to_string(),
            binding_digest: binding.binding_digest().to_string(),
        };
        self.execute_authoritative_write_batch_direct(
            execution,
            handoff.commands().to_vec(),
            handoff.graph_composition_breadth().clone(),
            handoff.graph_composition_program().clone(),
            Some(shared_admission),
        )
    }
}

fn admitted_authoritative_mutation_batch_handoff_from_review(
    review: crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData,
) -> Result<WorthQueryAuthoritativeMutationBatchExecutionHandoff, WorthQueryRuntimeError> {
    match review.decision().clone() {
        WorthQueryIntentAdmissionDecision::Admitted(
            crate::intent_admission::WorthQueryAdmittedIntentPlan::AuthoritativeMutationBatch(plan),
        ) => Ok(WorthQueryAuthoritativeMutationBatchExecutionHandoff::from_plan(plan)),
        WorthQueryIntentAdmissionDecision::Admitted(_)
        | WorthQueryIntentAdmissionDecision::Advisory(_)
        | WorthQueryIntentAdmissionDecision::Violation(_) => {
            let violation = non_admitted_runtime_violation(&review);
            Err(WorthQueryRuntimeError::Workspace(
                WorthQueryWorkspaceError::new(violation.message()),
            ))
        }
    }
}
