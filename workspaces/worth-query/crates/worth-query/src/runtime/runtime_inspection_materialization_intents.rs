use super::runtime_inspection_materialization_identity::bundle_snapshot_identity;
use super::*;
use crate::intent_admission::dx::{
    non_admitted_runtime_violation, WorthQueryRuntimeIntentAdmissionReviewData,
};
use crate::intent_admission::{
    WorthQueryDerivedInspectionExecutionBinding, WorthQueryDerivedInspectionExecutionHandoff,
    WorthQueryDerivedMaterializationExecutionBinding,
    WorthQueryDerivedMaterializationExecutionHandoff, WorthQueryDerivedViewIntentSeed,
    WorthQueryIntentAdmissionDecision,
};
use crate::runtime::{
    WorthQueryBatchWriteReceipt, WorthQueryBatchWriteRetainedArtifact,
    WorthQueryDerivedMaterializationBundle, WorthQueryDerivedMaterializationTarget,
};
use std::collections::BTreeMap;

impl WorthQueryWorkspace {
    pub fn materialize_batch_write_artifact_binding(
        &mut self,
        receipt: &WorthQueryBatchWriteReceipt,
        artifact_name: impl Into<String>,
        targets: impl IntoIterator<Item = WorthQueryDerivedMaterializationTarget>,
    ) -> Result<WorthQueryBatchWriteRetainedArtifact, WorthQueryRuntimeError> {
        WorthQueryBatchWriteRetainedArtifact::build(self, receipt, artifact_name, targets)
    }

    pub fn materialize_derived_artifact_binding(
        &mut self,
        artifact_name: impl Into<String>,
        targets: impl IntoIterator<Item = WorthQueryDerivedMaterializationTarget>,
    ) -> Result<crate::runtime::WorthQueryDerivedArtifactBinding, WorthQueryRuntimeError> {
        let retained_targets = targets.into_iter().collect::<Vec<_>>();
        self.materialize_derived_artifact_bundle(retained_targets.clone())?
            .bind_retained_artifact(artifact_name, retained_targets)
    }

    pub fn materialize_derived_artifact_bundle(
        &mut self,
        targets: impl IntoIterator<Item = WorthQueryDerivedMaterializationTarget>,
    ) -> Result<WorthQueryDerivedMaterializationBundle, WorthQueryRuntimeError> {
        let mut retained_targets = targets.into_iter().collect::<Vec<_>>();
        retained_targets.sort();
        retained_targets.dedup();

        let mut materializations = BTreeMap::new();
        for target in retained_targets {
            let result = self.materialize_derived_target(&target)?;
            materializations.insert(target, result);
        }
        let snapshot_identity = bundle_snapshot_identity(&materializations)?;
        Ok(WorthQueryDerivedMaterializationBundle::new(
            snapshot_identity,
            materializations,
        ))
    }

    pub fn materialize_intent<T>(
        &mut self,
        view: &WorthQueryDerivedViewHandle<T>,
    ) -> crate::intent_admission::WorthQueryWorkspaceDerivedMaterializationIntentAuthoring<'_> {
        crate::intent_admission::WorthQueryWorkspaceDerivedMaterializationIntentAuthoring::new(
            self,
            view.name().to_string(),
        )
    }

    pub fn inspect_derived_intent<T>(
        &mut self,
        view: &WorthQueryDerivedViewHandle<T>,
    ) -> crate::intent_admission::WorthQueryWorkspaceDerivedInspectionIntentAuthoring<'_> {
        crate::intent_admission::WorthQueryWorkspaceDerivedInspectionIntentAuthoring::new(
            self,
            view.name().to_string(),
        )
    }

    pub(crate) fn review_derived_materialization(
        &self,
        view_name: String,
    ) -> Result<WorthQueryRuntimeIntentAdmissionReviewData, WorthQueryRuntimeError> {
        self.runtime
            .review_runtime_derived_materialization(view_name)
    }

    pub(crate) fn resolve_reviewed_admitted_derived_materialization_handoff(
        &self,
        review: WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> Result<WorthQueryDerivedMaterializationExecutionHandoff, WorthQueryRuntimeError> {
        self.runtime
            .resolve_reviewed_admitted_derived_materialization_handoff(review)
    }

    pub(crate) fn derived_materialization_non_admitted_error(
        &self,
        review: &WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> WorthQueryRuntimeError {
        self.runtime
            .derived_materialization_non_admitted_error(review)
    }

    pub(crate) fn into_runtime_derived_materialization_binding(
        &self,
        handoff: WorthQueryDerivedMaterializationExecutionHandoff,
    ) -> WorthQueryDerivedMaterializationExecutionBinding {
        self.runtime
            .prepare_derived_materialization_execution_binding(handoff)
    }

    pub(crate) fn execute_bound_derived_materialization(
        &self,
        binding: WorthQueryDerivedMaterializationExecutionBinding,
    ) -> Result<WorthQueryDerivedMaterializationResult, WorthQueryRuntimeError> {
        self.runtime
            .execute_derived_materialization_execution_binding(binding)
    }

    pub(crate) fn review_derived_inspection(
        &self,
        view_name: String,
    ) -> Result<WorthQueryRuntimeIntentAdmissionReviewData, WorthQueryRuntimeError> {
        self.runtime.review_runtime_derived_inspection(view_name)
    }

    pub(crate) fn resolve_reviewed_admitted_derived_inspection_handoff(
        &self,
        review: WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> Result<WorthQueryDerivedInspectionExecutionHandoff, WorthQueryRuntimeError> {
        self.runtime
            .resolve_reviewed_admitted_derived_inspection_handoff(review)
    }

    pub(crate) fn derived_inspection_non_admitted_error(
        &self,
        review: &WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> WorthQueryRuntimeError {
        self.runtime.derived_inspection_non_admitted_error(review)
    }

    pub(crate) fn into_runtime_derived_inspection_binding(
        &self,
        handoff: WorthQueryDerivedInspectionExecutionHandoff,
    ) -> WorthQueryDerivedInspectionExecutionBinding {
        self.runtime
            .prepare_derived_inspection_execution_binding(handoff)
    }

    pub(crate) fn execute_bound_derived_inspection(
        &self,
        binding: WorthQueryDerivedInspectionExecutionBinding,
    ) -> Result<WorthQueryDerivedInspectionResult, WorthQueryRuntimeError> {
        self.runtime
            .execute_derived_inspection_execution_binding(binding)
    }

    fn materialize_derived_target(
        &mut self,
        target: &WorthQueryDerivedMaterializationTarget,
    ) -> Result<WorthQueryDerivedMaterializationResult, WorthQueryRuntimeError> {
        let review = self
            .review_derived_materialization(target.terminal_view_name_projection().to_string())?;
        let handoff = self.resolve_reviewed_admitted_derived_materialization_handoff(review)?;
        let binding = self.into_runtime_derived_materialization_binding(handoff);
        self.execute_bound_derived_materialization(binding)
    }
}

impl WorthQueryRuntime {
    pub(crate) fn review_runtime_derived_materialization(
        &self,
        view_name: String,
    ) -> Result<WorthQueryRuntimeIntentAdmissionReviewData, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Computed)?;
        let seed = self.derived_view_intent_seed(&view_name)?;
        let request =
            crate::intent_admission::WorthQueryRawIntentAdmissionRequest::derived_materialization_entrypoint(seed)
                .map_err(|violation| {
                    WorthQueryRuntimeError::MissingDerivedView(violation.message().to_string())
                })?;
        Ok(WorthQueryRuntimeIntentAdmissionReviewData::from_request(
            request,
        ))
    }

    pub(crate) fn review_runtime_derived_inspection(
        &self,
        view_name: String,
    ) -> Result<WorthQueryRuntimeIntentAdmissionReviewData, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Inspect)?;
        let seed = self.derived_view_intent_seed(&view_name)?;
        let request =
            crate::intent_admission::WorthQueryRawIntentAdmissionRequest::derived_inspection_entrypoint(seed)
                .map_err(|violation| {
                    WorthQueryRuntimeError::MissingDerivedView(violation.message().to_string())
                })?;
        Ok(WorthQueryRuntimeIntentAdmissionReviewData::from_request(
            request,
        ))
    }

    pub(crate) fn resolve_reviewed_admitted_derived_materialization_handoff(
        &self,
        review: WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> Result<WorthQueryDerivedMaterializationExecutionHandoff, WorthQueryRuntimeError> {
        match review.decision().clone() {
            WorthQueryIntentAdmissionDecision::Admitted(
                crate::intent_admission::WorthQueryAdmittedIntentPlan::DerivedMaterialization(plan),
            ) => Ok(WorthQueryDerivedMaterializationExecutionHandoff::from_plan(
                plan,
            )),
            _ => Err(self.derived_materialization_non_admitted_error(&review)),
        }
    }

    pub(crate) fn resolve_reviewed_admitted_derived_inspection_handoff(
        &self,
        review: WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> Result<WorthQueryDerivedInspectionExecutionHandoff, WorthQueryRuntimeError> {
        match review.decision().clone() {
            WorthQueryIntentAdmissionDecision::Admitted(
                crate::intent_admission::WorthQueryAdmittedIntentPlan::DerivedInspection(plan),
            ) => Ok(WorthQueryDerivedInspectionExecutionHandoff::from_plan(plan)),
            _ => Err(self.derived_inspection_non_admitted_error(&review)),
        }
    }

    pub(crate) fn derived_materialization_non_admitted_error(
        &self,
        review: &WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> WorthQueryRuntimeError {
        WorthQueryRuntimeError::MissingDerivedView(
            non_admitted_runtime_violation(review).message().to_string(),
        )
    }

    pub(crate) fn derived_inspection_non_admitted_error(
        &self,
        review: &WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> WorthQueryRuntimeError {
        WorthQueryRuntimeError::MissingDerivedView(
            non_admitted_runtime_violation(review).message().to_string(),
        )
    }

    pub(crate) fn prepare_derived_materialization_execution_binding(
        &self,
        handoff: WorthQueryDerivedMaterializationExecutionHandoff,
    ) -> WorthQueryDerivedMaterializationExecutionBinding {
        WorthQueryDerivedMaterializationExecutionBinding::from_handoff(handoff)
    }

    pub(crate) fn prepare_derived_inspection_execution_binding(
        &self,
        handoff: WorthQueryDerivedInspectionExecutionHandoff,
    ) -> WorthQueryDerivedInspectionExecutionBinding {
        WorthQueryDerivedInspectionExecutionBinding::from_handoff(handoff)
    }

    pub(crate) fn execute_derived_materialization_execution_binding(
        &self,
        binding: WorthQueryDerivedMaterializationExecutionBinding,
    ) -> Result<WorthQueryDerivedMaterializationResult, WorthQueryRuntimeError> {
        let evidence = self.derived_view_evidence(binding.view_name())?;
        let rows = self
            .derived_views
            .get(&WorthQueryDerivedMaterializationTarget::new(
                binding.view_name(),
            ))
            .map(|runtime| runtime.materialization.retained_rows().to_vec())
            .ok_or_else(|| {
                WorthQueryRuntimeError::MissingDerivedView(binding.view_name().to_string())
            })?;
        let snapshot_identity = self.current_snapshot_identity()?;
        let receipt = WorthQueryDerivedMaterializationReceipt::from_evidence(
            &evidence,
            snapshot_identity.clone(),
        );
        let mut result = WorthQueryDerivedMaterializationResult::from_retained_rows(rows, receipt);
        let decision_trace_envelope =
            WorthQueryIntentDecisionTraceEnvelope::for_admitted_execution_parts(
                binding.family(),
                binding.entrypoint(),
                binding.view_name(),
                binding.handoff().request_digest(),
                binding.handoff().eligibility_trace().clone(),
                binding.handoff().decision_digest(),
                binding.handoff().handoff_digest(),
                binding.execution_seam(),
                binding.view_name(),
                result.receipt().result_digest(),
                "derived-view-materialization",
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

    pub(crate) fn execute_derived_inspection_execution_binding(
        &self,
        binding: WorthQueryDerivedInspectionExecutionBinding,
    ) -> Result<WorthQueryDerivedInspectionResult, WorthQueryRuntimeError> {
        let evidence = self.derived_view_evidence(binding.view_name())?;
        let snapshot_identity = self.current_snapshot_identity()?;
        let receipt =
            WorthQueryDerivedInspectionReceipt::from_evidence(&evidence, snapshot_identity.clone());
        let mut result = WorthQueryDerivedInspectionResult::new(evidence, receipt);
        let decision_trace_envelope =
            WorthQueryIntentDecisionTraceEnvelope::for_admitted_execution_parts(
                binding.family(),
                binding.entrypoint(),
                binding.view_name(),
                binding.handoff().request_digest(),
                binding.handoff().eligibility_trace().clone(),
                binding.handoff().decision_digest(),
                binding.handoff().handoff_digest(),
                binding.execution_seam(),
                binding.view_name(),
                result.receipt().result_digest(),
                "derived-view-inspection",
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

    fn derived_view_intent_seed(
        &self,
        view_name: &str,
    ) -> Result<WorthQueryDerivedViewIntentSeed, WorthQueryRuntimeError> {
        let handle =
            WorthQueryDerivedViewHandle::<crate::runtime::WorthQueryUnrefinedLiveShape>::new(
                view_name,
            );
        let evidence = self.derived_view_evidence(view_name)?;
        Ok(WorthQueryDerivedViewIntentSeed::new(
            &handle,
            evidence.authority_lane(),
            evidence.dependency_digest(),
            evidence.materialization_digest(),
            evidence.inspection_digest(),
            evidence.materialized_row_count(),
        ))
    }

    fn derived_view_evidence(
        &self,
        view_name: &str,
    ) -> Result<WorthQueryComputedInspectionEvidence, WorthQueryRuntimeError> {
        self.derived_views
            .get(&WorthQueryDerivedMaterializationTarget::new(view_name))
            .map(WorthQueryComputedInspectionEvidence::from_runtime)
            .ok_or_else(|| WorthQueryRuntimeError::MissingDerivedView(view_name.to_string()))
    }
}
