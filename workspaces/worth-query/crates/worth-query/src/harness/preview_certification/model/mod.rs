mod artifact;
mod digests;
mod lane;
mod perturbation;
mod rejection;
mod row_validation;

pub use artifact::{
    MilestoneFivePointTwoPreviewCertificationArtifact, PreviewBundleCompletenessReport,
};
pub use lane::{PreviewCertificationLane, PreviewLaneEvaluationClass, PreviewLaneLifecycleState};
pub use perturbation::PreviewPerturbationClass;
pub use rejection::{PreviewCertificationRejection, PreviewFailureClass};

use super::super::certification::{
    CanonicalCertificationRow, CertificationMatrix, RejectionCertificationRow,
};
use crate::preview::{
    PreviewBindingCounters, PreviewBindingFailureClass, PreviewComparisonError,
    PreviewComparisonFailureClass, PreviewEvaluationClass, PreviewExecutionEnvelope,
    PreviewLiveError, PreviewLiveFailureClass,
};

pub type PreviewCertificationRow =
    CanonicalCertificationRow<PreviewPerturbationClass, PreviewCertificationLane>;
pub type PreviewRejectionRow = RejectionCertificationRow<
    PreviewPerturbationClass,
    PreviewCertificationLane,
    PreviewCertificationRejection,
>;
pub type PreviewCertificationMatrix = CertificationMatrix<
    PreviewPerturbationClass,
    PreviewCertificationLane,
    PreviewCertificationRejection,
>;

impl PreviewCertificationLane {
    pub(super) fn from_execution(binding: &PreviewExecutionEnvelope) -> Self {
        let binding_tuple = binding.binding().basis().binding_tuple();
        Self {
            query_digest: binding
                .binding()
                .basis()
                .binding_tuple()
                .canonical_query_digest()
                .as_str()
                .to_string(),
            result_shape_digest: binding
                .binding()
                .basis()
                .binding_tuple()
                .canonical_result_shape_digest()
                .as_str()
                .to_string(),
            preview_session_identity: binding
                .binding()
                .basis()
                .binding_tuple()
                .preview_session_identity()
                .bridge_admission_evidence()
                .terminal_projection_for_reporting()
                .to_string(),
            evaluation_class: match binding_tuple.evaluation_class() {
                PreviewEvaluationClass::ReadOnly(_) => PreviewLaneEvaluationClass::ReadOnly,
                PreviewEvaluationClass::PromotionEligible(_) => {
                    PreviewLaneEvaluationClass::PromotionEligible
                }
            },
            lifecycle_state_kind: match binding_tuple.lifecycle_state_kind() {
                worth_runtime_bridge::facade::BridgePreviewLifecycleStateKind::Active => {
                    PreviewLaneLifecycleState::Active
                }
                worth_runtime_bridge::facade::BridgePreviewLifecycleStateKind::Admitted => {
                    PreviewLaneLifecycleState::Admitted
                }
                worth_runtime_bridge::facade::BridgePreviewLifecycleStateKind::Declared => {
                    PreviewLaneLifecycleState::Declared
                }
                worth_runtime_bridge::facade::BridgePreviewLifecycleStateKind::Promoted => {
                    PreviewLaneLifecycleState::Promoted
                }
                worth_runtime_bridge::facade::BridgePreviewLifecycleStateKind::Discarded => {
                    PreviewLaneLifecycleState::Discarded
                }
            },
            binding_digest: binding_tuple.digest().to_string(),
            preview_execution_digest: binding.report().preview_execution_digest().to_string(),
            comparison_eligibility_digest: binding
                .report()
                .comparison_eligibility_digest()
                .to_string(),
            workflow_foundation_digest: binding.report().workflow_foundation_digest().to_string(),
            promotion_parity_digest: None,
            preview_live_digest: None,
            preview_live_subscription_digest: None,
            preview_live_family: None,
            counters: binding.counters().binding_counters().clone(),
            execution_counters: binding.counters().clone(),
            comparison_counters: None,
            preview_live_counters: None,
        }
    }

    pub(super) fn with_promotion_parity(
        mut self,
        admission: &crate::preview::PromotionParityPreviewComparisonAdmission,
    ) -> Self {
        self.promotion_parity_digest = Some(admission.as_preview_comparison().digest().to_string());
        self.comparison_counters = Some(admission.as_preview_comparison().counters().clone());
        self
    }

    pub(super) fn with_preview_live(
        mut self,
        preview_live: &crate::preview::PreviewLiveExecutionEnvelope,
    ) -> Self {
        let admitted = preview_live.preview_live();
        self.preview_live_digest = Some(admitted.report().digest().to_string());
        self.preview_live_subscription_digest =
            Some(admitted.report().live_subscription_digest().to_string());
        self.preview_live_family = Some(admitted.report().live_family().to_string());
        self.preview_live_counters = Some(preview_live.counters().clone());
        self
    }

    pub(super) fn with_preview_live_rebind(
        mut self,
        rebound_execution: &crate::preview::PreviewLiveExecutionEnvelope,
        rebind: &crate::preview::PreviewLiveRebindArtifact,
    ) -> Self {
        self = self.with_preview_live(rebound_execution);
        let mut counters = self.preview_live_counters.take().unwrap_or_default();
        counters.absorb(rebind.counters());
        self.preview_live_counters = Some(counters);
        self
    }

    pub fn has_required_outputs(&self) -> bool {
        !self.query_digest.is_empty()
            && !self.result_shape_digest.is_empty()
            && !self.preview_session_identity.is_empty()
            && !self.binding_digest.is_empty()
            && !self.preview_execution_digest.is_empty()
            && !self.comparison_eligibility_digest.is_empty()
            && !self.workflow_foundation_digest.is_empty()
            && match (
                self.preview_live_digest.as_ref(),
                self.preview_live_subscription_digest.as_ref(),
                self.preview_live_family.as_ref(),
                self.preview_live_counters.as_ref(),
            ) {
                (None, None, None, None) => true,
                (Some(digest), Some(subscription), Some(family), Some(_)) => {
                    !digest.is_empty() && !subscription.is_empty() && !family.is_empty()
                }
                _ => false,
            }
    }
}

impl PreviewCertificationRejection {
    pub(super) fn from_runtime_failure(
        failure_class: &PreviewBindingFailureClass,
        counters: &PreviewBindingCounters,
    ) -> Self {
        Self {
            failure_class: match failure_class {
                PreviewBindingFailureClass::InvalidPreviewBasis => {
                    PreviewFailureClass::InvalidPreviewBasis
                }
                PreviewBindingFailureClass::UnsupportedPreviewQueryFamily => {
                    PreviewFailureClass::UnsupportedPreviewFamily
                }
                PreviewBindingFailureClass::StoreBackedRouteForbidden => {
                    PreviewFailureClass::StoreBackedRouteForbidden
                }
                PreviewBindingFailureClass::StaleOrInactivePreviewLifecycle => {
                    PreviewFailureClass::StaleOrInactivePreviewLifecycle
                }
                PreviewBindingFailureClass::PromotionLinkageMismatch => {
                    PreviewFailureClass::PromotionLinkageMismatch
                }
                other => panic!("unsupported preview certification runtime failure: {other:?}"),
            },
            counters: Some(counters.clone()),
            execution_counters: None,
            comparison_counters: None,
            preview_live_counters: None,
        }
    }

    pub(super) fn from_comparison_failure(error: &PreviewComparisonError) -> Self {
        Self {
            failure_class: match error.failure_class() {
                PreviewComparisonFailureClass::QueryDigestMismatch
                | PreviewComparisonFailureClass::ResultShapeMismatch
                | PreviewComparisonFailureClass::ResultFamilyMismatch
                | PreviewComparisonFailureClass::OrderingBasisMismatch
                | PreviewComparisonFailureClass::MaterializationBoundaryMismatch => {
                    PreviewFailureClass::PreviewShapeMismatchDenied
                }
                other => panic!("unsupported preview certification comparison failure: {other:?}"),
            },
            counters: None,
            execution_counters: None,
            comparison_counters: Some(error.counters().clone()),
            preview_live_counters: None,
        }
    }

    pub(super) fn from_workflow_failure(
        error: &crate::preview::PreviewWorkflowFoundationError,
    ) -> Self {
        Self {
            failure_class: match error.failure_class() {
                crate::preview::PreviewWorkflowFoundationFailureClass::EvidenceIdentityDerivationDenied(cause) => PreviewFailureClass::EvidenceIdentityDerivationDenied(*cause),
                crate::preview::PreviewWorkflowFoundationFailureClass::ReadOnlyPreviewWritebackFoundationForbidden => {
                    PreviewFailureClass::WorkflowFoundationAuthorityDenied
                }
                crate::preview::PreviewWorkflowFoundationFailureClass::OutOfScopeWorkflowFoundationRequest => {
                    panic!("out-of-scope workflow foundation denial is no longer expected in preview certification")
                }
            },
            counters: None,
            execution_counters: Some(error.counters().clone()),
            comparison_counters: None,
            preview_live_counters: None,
        }
    }

    pub(super) fn from_preview_live_failure(error: &PreviewLiveError) -> Self {
        Self {
            failure_class: match error.failure_class() {
                PreviewLiveFailureClass::PreviewLiveLifecycleDrifted => {
                    PreviewFailureClass::PreviewLiveDriftDenied
                }
                PreviewLiveFailureClass::PreviewLiveBroadFallbackForbidden => {
                    PreviewFailureClass::PreviewLiveBroadFallbackForbidden
                }
                other => panic!("unsupported preview-live certification failure: {other:?}"),
            },
            counters: None,
            execution_counters: None,
            comparison_counters: None,
            preview_live_counters: Some(error.counters().clone()),
        }
    }

    pub fn has_required_outputs(&self) -> bool {
        self.counters.is_some()
            || self.execution_counters.is_some()
            || self.comparison_counters.is_some()
            || self.preview_live_counters.is_some()
    }
}
