use super::*;
use crate::intent_admission::dx::{
    non_admitted_runtime_violation, WorthQueryRuntimeIntentAdmissionReviewData,
};
use crate::intent_admission::{
    WorthQueryExistingTruthProbeExecutionBinding, WorthQueryExistingTruthProbeExecutionHandoff,
    WorthQueryExistingTruthProbeIntentSeed, WorthQueryExistingTruthProbeRoutingPreflight,
    WorthQueryIntentAdmissionDecision, WorthQueryIntentDecisionTraceEnvelope,
};

impl WorthQueryRuntime {
    pub fn probe_existing_intent(
        &self,
        request: WorthQueryExistingTruthProbeRequest,
    ) -> crate::intent_admission::WorthQueryRuntimeExistingTruthProbeIntentAuthoring<'_> {
        crate::intent_admission::WorthQueryRuntimeExistingTruthProbeIntentAuthoring::new(
            self, request,
        )
    }

    pub(crate) fn review_existing_truth_probe_routing(
        &self,
        request: WorthQueryExistingTruthProbeRequest,
    ) -> Result<WorthQueryRuntimeIntentAdmissionReviewData, WorthQueryRuntimeError> {
        let seed = self.build_existing_truth_probe_intent_seed(request);
        let request = crate::intent_admission::WorthQueryRawIntentAdmissionRequest::existing_truth_probe_entrypoint(seed)
            .expect("existing truth probe intent seed should always author successfully");
        Ok(WorthQueryRuntimeIntentAdmissionReviewData::from_request(
            request,
        ))
    }

    pub(crate) fn resolve_reviewed_admitted_existing_truth_probe_handoff(
        &self,
        review: WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> Result<WorthQueryExistingTruthProbeExecutionHandoff, WorthQueryRuntimeError> {
        match review.decision().clone() {
            WorthQueryIntentAdmissionDecision::Admitted(
                crate::intent_admission::WorthQueryAdmittedIntentPlan::ExistingTruthProbeRouting(
                    plan,
                ),
            ) => Ok(WorthQueryExistingTruthProbeExecutionHandoff::from_plan(
                plan,
            )),
            _ => Err(self.existing_truth_probe_non_admitted_error(&review)),
        }
    }

    pub(crate) fn existing_truth_probe_non_admitted_error(
        &self,
        review: &WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> WorthQueryRuntimeError {
        let seed = review
            .request()
            .existing_truth_probe_seed()
            .expect("probe routing review must preserve probe routing seed");
        match seed.preflight() {
            WorthQueryExistingTruthProbeRoutingPreflight::BindingDenied(denial) => {
                WorthQueryRuntimeError::MutationBindingDenied(denial.clone())
            }
            WorthQueryExistingTruthProbeRoutingPreflight::ProbeDenied(denial) => {
                WorthQueryRuntimeError::ExistingTruthProbeDenied(denial.clone())
            }
            WorthQueryExistingTruthProbeRoutingPreflight::Admitted => {
                let violation = non_admitted_runtime_violation(review);
                WorthQueryRuntimeError::ExistingTruthProbeDenied(
                    WorthQueryExistingTruthProbeDenial::new(
                        seed.request().binding(),
                        WorthQueryExistingTruthProbeDenialKind::BackendProbeUnsupported,
                        None,
                        violation.message(),
                    ),
                )
            }
        }
    }

    pub(crate) fn prepare_existing_truth_probe_execution_binding(
        &self,
        handoff: WorthQueryExistingTruthProbeExecutionHandoff,
    ) -> WorthQueryExistingTruthProbeExecutionBinding {
        WorthQueryExistingTruthProbeExecutionBinding::from_handoff(handoff)
    }

    pub(crate) fn execute_existing_truth_probe_execution_binding(
        &self,
        binding: WorthQueryExistingTruthProbeExecutionBinding,
    ) -> Result<WorthQueryExistingTruthProbeResult, WorthQueryRuntimeError> {
        let probe = self
            .backend
            .probe_existing_truth(binding.request())
            .map_err(WorthQueryRuntimeError::ExistingTruthProbeDenied)?;
        let snapshot_identity = self.current_snapshot_identity()?;
        let receipt = WorthQueryExistingTruthProbeReceipt::from_probe(
            binding.request(),
            &probe,
            snapshot_identity,
        );
        let mut result = WorthQueryExistingTruthProbeResult::new(probe, receipt);
        let decision_trace_envelope =
            WorthQueryIntentDecisionTraceEnvelope::for_admitted_execution_parts(
                binding.family(),
                binding.entrypoint(),
                binding
                    .request()
                    .binding()
                    .authoritative_identity()
                    .as_str(),
                binding.handoff().request_digest(),
                binding.handoff().eligibility_trace().clone(),
                binding.handoff().decision_digest(),
                binding.handoff().handoff_digest(),
                binding.execution_seam(),
                binding.request().binding().binding_digest().as_str(),
                result.receipt().probe_digest(),
                "existing-truth-probe",
            );
        let snapshot_evidence_identity = self.current_snapshot_identity()?.evidence_identity();
        let execution_provenance =
            WorthQueryIntentExecutionProvenance::for_shared_execution_typed_parts(
                binding.family(),
                binding.entrypoint(),
                binding.execution_seam(),
                binding.handoff().decision_digest(),
                binding.handoff().handoff_digest(),
                binding.binding_digest(),
                result.receipt().probe_digest(),
                &snapshot_evidence_identity,
            );
        result.attach_intent_admission_evidence(decision_trace_envelope, execution_provenance);
        Ok(result)
    }

    pub(crate) fn build_existing_truth_probe_intent_seed(
        &self,
        request: WorthQueryExistingTruthProbeRequest,
    ) -> WorthQueryExistingTruthProbeIntentSeed {
        let preflight = match self.backend.admit_existing_truth_binding(request.binding()) {
            Err(denial) => WorthQueryExistingTruthProbeRoutingPreflight::BindingDenied(denial),
            Ok(()) => match self.probe_existing_support_denial(request.binding()) {
                Some(denial) => WorthQueryExistingTruthProbeRoutingPreflight::ProbeDenied(denial),
                None => WorthQueryExistingTruthProbeRoutingPreflight::Admitted,
            },
        };
        WorthQueryExistingTruthProbeIntentSeed::new(request, preflight)
    }

    fn probe_existing_support_denial(
        &self,
        binding: &WorthQueryExistingTruthTargetBinding,
    ) -> Option<WorthQueryExistingTruthProbeDenial> {
        let support_profile = self.backend.support_profile();
        let row = support_profile
            .bridge_backed_verification_support_rows()
            .iter()
            .find(|row| {
                row.operation_family() == "probe_existing"
                    && row.target_binding_family()
                        == binding.family().bridge_backed_support_family()
            })?;
        let supported = match support_profile.posture() {
            WorthQueryRuntimeBackendPosture::Scaffold => row.scaffold_profile_supported(),
            WorthQueryRuntimeBackendPosture::Primary => {
                row.primary_bridge_backed_runtime_supported()
            }
        };
        if supported {
            return None;
        }
        Some(WorthQueryExistingTruthProbeDenial::new(
            binding,
            WorthQueryExistingTruthProbeDenialKind::BackendProbeUnsupported,
            None,
            row.denial_class_when_primary_unsupported().unwrap_or(
                "this runtime backend does not admit backend-verified existing-truth probes yet",
            ),
        ))
    }
}
