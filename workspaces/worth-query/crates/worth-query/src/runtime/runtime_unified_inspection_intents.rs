use super::*;
use crate::intent_admission::dx::{
    non_admitted_runtime_violation, WorthQueryRuntimeIntentAdmissionReviewData,
};
use crate::intent_admission::{
    WorthQueryGenericInspectionIntentSeed, WorthQueryIntentAdmissionDecision,
    WorthQueryIntentDecisionTraceEnvelope, WorthQueryUnifiedInspectionExecutionBinding,
    WorthQueryUnifiedInspectionExecutionHandoff,
};

impl WorthQueryRuntime {
    pub(crate) fn inspect_live_view_name_result(
        &self,
        view_name: &str,
    ) -> Result<WorthQueryUnifiedInspectionResult, WorthQueryRuntimeError> {
        let seed = WorthQueryGenericInspectionIntentSeed::from_target(
            WorthQueryInspectionTarget::LiveView { name: view_name },
        )
        .expect("live view names should always lower into unified inspection seeds");
        let review = self.review_unified_inspection(seed)?;
        let handoff = self.resolve_reviewed_admitted_unified_inspection_handoff(review)?;
        let binding = self.prepare_unified_inspection_execution_binding(handoff);
        self.execute_unified_inspection_execution_binding(binding)
    }

    pub(crate) fn review_unified_inspection(
        &self,
        seed: WorthQueryGenericInspectionIntentSeed,
    ) -> Result<WorthQueryRuntimeIntentAdmissionReviewData, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Inspect)?;
        let request =
            crate::intent_admission::WorthQueryRawIntentAdmissionRequest::generic_inspection_entrypoint(seed)
                .expect("generic inspection seed should always author successfully");
        Ok(WorthQueryRuntimeIntentAdmissionReviewData::from_request(
            request,
        ))
    }

    pub(crate) fn resolve_reviewed_admitted_unified_inspection_handoff(
        &self,
        review: WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> Result<WorthQueryUnifiedInspectionExecutionHandoff, WorthQueryRuntimeError> {
        match review.decision().clone() {
            WorthQueryIntentAdmissionDecision::Admitted(
                crate::intent_admission::WorthQueryAdmittedIntentPlan::UnifiedInspection(plan),
            ) => Ok(WorthQueryUnifiedInspectionExecutionHandoff::from_plan(plan)),
            _ => Err(self.unified_inspection_non_admitted_error(&review)),
        }
    }

    pub(crate) fn unified_inspection_non_admitted_error(
        &self,
        review: &WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> WorthQueryRuntimeError {
        let violation = non_admitted_runtime_violation(review);
        WorthQueryRuntimeError::MissingLiveView(violation.message().to_string())
    }

    pub(crate) fn prepare_unified_inspection_execution_binding(
        &self,
        handoff: WorthQueryUnifiedInspectionExecutionHandoff,
    ) -> WorthQueryUnifiedInspectionExecutionBinding {
        WorthQueryUnifiedInspectionExecutionBinding::from_handoff(handoff)
    }

    pub(crate) fn execute_unified_inspection_execution_binding(
        &self,
        binding: WorthQueryUnifiedInspectionExecutionBinding,
    ) -> Result<WorthQueryUnifiedInspectionResult, WorthQueryRuntimeError> {
        let inspection = self.inspect_from_generic_seed(binding.seed())?;
        let snapshot_identity = self.current_snapshot_identity()?;
        let receipt = WorthQueryUnifiedInspectionReceipt::from_inspection(
            binding.seed().request_label().clone(),
            &inspection,
            snapshot_identity.clone(),
        );
        let mut result = WorthQueryUnifiedInspectionResult::new(inspection, receipt);
        let decision_trace_envelope =
            WorthQueryIntentDecisionTraceEnvelope::for_admitted_execution_parts(
                binding.family(),
                binding.entrypoint(),
                binding.seed().request_label().as_str(),
                binding.handoff().request_digest(),
                binding.handoff().eligibility_trace().clone(),
                binding.handoff().decision_digest(),
                binding.handoff().handoff_digest(),
                binding.execution_seam(),
                binding.seed().request_input_digest(),
                result.receipt().result_digest(),
                "unified-inspection",
            );
        let snapshot_evidence_identity = snapshot_identity.evidence_identity();
        let execution_provenance =
            WorthQueryIntentExecutionProvenance::for_shared_execution_typed_parts(
                binding.family(),
                binding.entrypoint(),
                binding.execution_seam(),
                binding.handoff().decision_digest(),
                binding.handoff().handoff_digest(),
                binding.binding_digest(),
                result.receipt().result_digest(),
                &snapshot_evidence_identity,
            );
        result.attach_intent_admission_evidence(decision_trace_envelope, execution_provenance);
        Ok(result)
    }

    pub(crate) fn inspect_from_generic_seed(
        &self,
        seed: &WorthQueryGenericInspectionIntentSeed,
    ) -> Result<WorthQueryInspection, WorthQueryRuntimeError> {
        match seed.target() {
            crate::intent_admission::WorthQueryGenericInspectionIntentTargetSeed::LiveView {
                view_name,
            } => {
                let state = self
                    .live_subscriptions
                    .get(&WorthQueryLiveArtifactTarget::from_view_name(view_name))
                    .ok_or_else(|| WorthQueryRuntimeError::MissingLiveSubscription(view_name.clone()))?;
                Ok(WorthQueryInspection::LiveView(
                    WorthQueryLiveViewInspection::from_state(state),
                ))
            }
            crate::intent_admission::WorthQueryGenericInspectionIntentTargetSeed::Effect {
                effect_name,
            } => Ok(WorthQueryInspection::Effect(
                self.inspect_effect_by_name(effect_name)?,
            )),
            crate::intent_admission::WorthQueryGenericInspectionIntentTargetSeed::WriteReceipt(
                receipt,
            ) => {
                let runtime_evidence = self
                    .backend
                    .inspect_write_receipt(receipt, &self.evidence_authority)?;
                Ok(WorthQueryInspection::WriteReceipt(
                    WorthQueryWriteReceiptInspection::new(receipt, runtime_evidence),
                ))
            }
            crate::intent_admission::WorthQueryGenericInspectionIntentTargetSeed::BatchWriteReceipt(
                receipt,
            ) => Ok(WorthQueryInspection::BatchWriteReceipt(
                WorthQueryBatchWriteReceiptInspection::new(receipt),
            )),
            crate::intent_admission::WorthQueryGenericInspectionIntentTargetSeed::IntentReceipt(
                receipt,
            ) => Ok(WorthQueryInspection::IntentReceipt(
                WorthQueryIntentReceiptInspection::from_receipt(receipt),
            )),
            crate::intent_admission::WorthQueryGenericInspectionIntentTargetSeed::IntentDenial(
                evidence,
            ) => Ok(WorthQueryInspection::IntentDenial(
                WorthQueryIntentDenialInspection::from_evidence(evidence),
            )),
            crate::intent_admission::WorthQueryGenericInspectionIntentTargetSeed::EffectIntentReceipt(
                receipt,
            ) => Ok(WorthQueryInspection::EffectIntentReceipt(
                WorthQueryEffectIntentReceiptInspection::from_receipt(receipt),
            )),
            crate::intent_admission::WorthQueryGenericInspectionIntentTargetSeed::PreviewBinding(
                binding,
            ) => Ok(WorthQueryInspection::PreviewBinding(
                WorthQueryPreviewBindingInspection::from_binding(binding),
            )),
            crate::intent_admission::WorthQueryGenericInspectionIntentTargetSeed::PreviewOutcome(
                outcome,
            ) => Ok(WorthQueryInspection::PreviewOutcome(
                WorthQueryPreviewOutcomeInspection::from_outcome(outcome),
            )),
            crate::intent_admission::WorthQueryGenericInspectionIntentTargetSeed::PreviewIntentReceipt(
                receipt,
            ) => Ok(WorthQueryInspection::PreviewIntentReceipt(
                WorthQueryPreviewIntentReceiptInspection::from_receipt(receipt),
            )),
            crate::intent_admission::WorthQueryGenericInspectionIntentTargetSeed::BranchIntentReceipt(
                receipt,
            ) => Ok(WorthQueryInspection::BranchIntentReceipt(
                WorthQueryBranchIntentReceiptInspection::from_receipt(receipt),
            )),
        }
    }
}
