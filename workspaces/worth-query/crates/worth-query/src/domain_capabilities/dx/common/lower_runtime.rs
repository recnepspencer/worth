use crate::domain_capabilities::authoring::{
    WorthQueryExplanationContributionAuthoring, WorthQuerySupportContributionAuthoring,
};
use crate::domain_capabilities::canonical_runtime::{
    materialize_lower_runtime_support_traceability_artifact,
    materialize_query_causal_inspection_artifact, materialize_query_causal_inspection_review,
    WorthQueryExplanationInspectionReview,
    WorthQueryLowerRuntimeBoundarySupportTraceabilityArtifact,
};
use crate::domain_capabilities::dx::checked::{
    WorthQueryCheckedDomainCapabilityOutcome, WorthQueryDomainCapabilityMaterializationError,
};
use crate::domain_capabilities::eligibility::{
    admit_eligible_domain_capability_contribution,
    evaluate_requested_domain_capability_contribution,
    prepare_admitted_domain_capability_contribution_for_materialization,
};
use crate::domain_capabilities::payloads::{
    WorthQueryExplanationContributionPosture, WorthQuerySupportContributionPosture,
};
use crate::domain_capabilities::{
    WorthQueryDomainCapabilityTargetKind, WorthQueryInstalledLowerRuntimeContributionTarget,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::QueryCausalInspectionArtifact;

use super::lower_runtime_explanation_request::{
    WorthQueryLowerRuntimeExplanationRequest, WorthQueryLowerRuntimeExplanationRequestKind,
};
use super::shared::{materialize_common_lane, qualify_semantic_code};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeDomainContributionSurface {
    pub(crate) target: WorthQueryInstalledLowerRuntimeContributionTarget,
}

impl WorthQueryLowerRuntimeDomainContributionSurface {
    pub fn supports_boundary_traceability(
        self,
        semantic_code: impl Into<String>,
    ) -> WorthQueryLowerRuntimeSupportDraft {
        WorthQueryLowerRuntimeSupportDraft {
            target: self.target,
            semantic_code: semantic_code.into(),
        }
    }

    pub fn requires_cross_runtime_context(
        self,
        semantic_code: impl Into<String>,
        request: WorthQueryLowerRuntimeExplanationRequest,
    ) -> WorthQueryLowerRuntimeExplanationDraft {
        WorthQueryLowerRuntimeExplanationDraft::new(self.target, semantic_code, request.kind())
    }

    pub fn explains_cross_runtime_fallback(
        self,
        semantic_code: impl Into<String>,
        request: WorthQueryLowerRuntimeExplanationRequest,
    ) -> WorthQueryLowerRuntimeExplanationDraft {
        WorthQueryLowerRuntimeExplanationDraft::new(self.target, semantic_code, request.kind())
    }

    pub fn explains_store_backed_replay_gap(
        self,
        semantic_code: impl Into<String>,
        request: WorthQueryLowerRuntimeExplanationRequest,
    ) -> WorthQueryLowerRuntimeExplanationDraft {
        WorthQueryLowerRuntimeExplanationDraft::new(self.target, semantic_code, request.kind())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeSupportDraft {
    target: WorthQueryInstalledLowerRuntimeContributionTarget,
    semantic_code: String,
}

impl WorthQueryLowerRuntimeSupportDraft {
    pub fn because(self, detail: impl Into<String>) -> WorthQueryLowerRuntimeSupportContribution {
        WorthQueryLowerRuntimeSupportContribution {
            target: self.target,
            semantic_code: self.semantic_code,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeSupportContribution {
    target: WorthQueryInstalledLowerRuntimeContributionTarget,
    semantic_code: String,
    detail: String,
}

impl WorthQueryLowerRuntimeSupportContribution {
    pub fn try_materialize(
        self,
    ) -> WorthQueryCheckedDomainCapabilityOutcome<
        WorthQueryLowerRuntimeBoundarySupportTraceabilityArtifact,
    > {
        let target = self.target.clone();
        let requested = WorthQuerySupportContributionAuthoring::narrowed_support(
            qualify_semantic_code(self.target.authority(), &self.semantic_code),
            self.detail,
        )
        .bind_to_installed_target(self.target);

        materialize_common_lane(
            "support-traceability",
            WorthQueryDomainCapabilityTargetKind::LowerRuntimeBoundaryEnvelope,
            WorthQuerySupportContributionPosture::NarrowedSupport.as_str(),
            requested,
            evaluate_requested_domain_capability_contribution,
            admit_eligible_domain_capability_contribution,
            |admitted| {
                prepare_admitted_domain_capability_contribution_for_materialization(
                    admitted, target,
                )
            },
            materialize_lower_runtime_support_traceability_artifact,
        )
    }

    pub fn materialize(
        self,
    ) -> Result<
        WorthQueryLowerRuntimeBoundarySupportTraceabilityArtifact,
        WorthQueryDomainCapabilityMaterializationError,
    > {
        self.try_materialize().into_result()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeExplanationDraft {
    target: WorthQueryInstalledLowerRuntimeContributionTarget,
    semantic_code: String,
    kind: WorthQueryLowerRuntimeExplanationRequestKind,
}

impl WorthQueryLowerRuntimeExplanationDraft {
    fn new(
        target: WorthQueryInstalledLowerRuntimeContributionTarget,
        semantic_code: impl Into<String>,
        kind: WorthQueryLowerRuntimeExplanationRequestKind,
    ) -> Self {
        Self {
            target,
            semantic_code: semantic_code.into(),
            kind,
        }
    }

    pub fn because(
        self,
        detail: impl Into<String>,
    ) -> WorthQueryLowerRuntimeExplanationContribution {
        WorthQueryLowerRuntimeExplanationContribution {
            target: self.target,
            semantic_code: self.semantic_code,
            detail: detail.into(),
            kind: self.kind,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeExplanationContribution {
    target: WorthQueryInstalledLowerRuntimeContributionTarget,
    semantic_code: String,
    detail: String,
    kind: WorthQueryLowerRuntimeExplanationRequestKind,
}

impl WorthQueryLowerRuntimeExplanationContribution {
    pub fn try_review(
        self,
    ) -> Result<
        WorthQueryCheckedDomainCapabilityOutcome<WorthQueryExplanationInspectionReview>,
        IdentityDerivationDenial,
    > {
        let posture = self.posture();
        let target = self.target.clone();
        let requested = self.into_requested()?;

        Ok(materialize_common_lane(
            "explanation-inspection",
            WorthQueryDomainCapabilityTargetKind::LowerRuntimeBoundaryEnvelope,
            posture.as_str(),
            requested,
            evaluate_requested_domain_capability_contribution,
            admit_eligible_domain_capability_contribution,
            |admitted| {
                prepare_admitted_domain_capability_contribution_for_materialization(
                    admitted, target,
                )
            },
            materialize_query_causal_inspection_review,
        ))
    }

    pub fn review(
        self,
    ) -> Result<WorthQueryExplanationInspectionReview, WorthQueryDomainCapabilityMaterializationError>
    {
        self.try_review().into_result()
    }

    pub fn try_materialize_artifact(
        self,
    ) -> Result<
        WorthQueryCheckedDomainCapabilityOutcome<QueryCausalInspectionArtifact>,
        IdentityDerivationDenial,
    > {
        let posture = self.posture();
        let target = self.target.clone();
        let requested = self.into_requested()?;

        Ok(materialize_common_lane(
            "explanation-inspection",
            WorthQueryDomainCapabilityTargetKind::LowerRuntimeBoundaryEnvelope,
            posture.as_str(),
            requested,
            evaluate_requested_domain_capability_contribution,
            admit_eligible_domain_capability_contribution,
            |admitted| {
                prepare_admitted_domain_capability_contribution_for_materialization(
                    admitted, target,
                )
            },
            materialize_query_causal_inspection_artifact,
        ))
    }

    pub fn materialize_artifact(
        self,
    ) -> Result<QueryCausalInspectionArtifact, WorthQueryDomainCapabilityMaterializationError> {
        self.try_materialize_artifact().into_result()
    }

    fn posture(&self) -> WorthQueryExplanationContributionPosture {
        match self.kind {
            WorthQueryLowerRuntimeExplanationRequestKind::CrossRuntimeContext { .. } => {
                WorthQueryExplanationContributionPosture::RequiresContext
            }
            WorthQueryLowerRuntimeExplanationRequestKind::CrossRuntimeFallback { .. } => {
                WorthQueryExplanationContributionPosture::ExplainsFallback
            }
            WorthQueryLowerRuntimeExplanationRequestKind::StoreBackedReplayGap { .. } => {
                WorthQueryExplanationContributionPosture::ExplainsAmbiguity
            }
        }
    }

    fn into_requested(
        self,
    ) -> Result<
        crate::domain_capabilities::WorthQueryRequestedExplanationContribution<
            WorthQueryInstalledLowerRuntimeContributionTarget,
        >,
        IdentityDerivationDenial,
    > {
        let semantic_code = qualify_semantic_code(self.target.authority(), &self.semantic_code);
        Ok(match self.kind {
            WorthQueryLowerRuntimeExplanationRequestKind::CrossRuntimeContext {
                reference_set,
                target,
                requested_evidence_families,
                bridge_envelope,
                redaction_policy,
                materialization_policy,
            } => WorthQueryExplanationContributionAuthoring::cross_runtime_causal_context(
                semantic_code,
                self.detail,
                reference_set,
                target,
                requested_evidence_families,
                bridge_envelope,
                redaction_policy,
                materialization_policy,
            )?
            .bind_to_installed_target(self.target),
            WorthQueryLowerRuntimeExplanationRequestKind::CrossRuntimeFallback {
                reference_set,
                target,
                requested_evidence_families,
                bridge_envelope,
                redaction_policy,
                materialization_policy,
            } => WorthQueryExplanationContributionAuthoring::cross_runtime_fallback_explanation(
                semantic_code,
                self.detail,
                reference_set,
                target,
                requested_evidence_families,
                bridge_envelope,
                redaction_policy,
                materialization_policy,
            )?
            .bind_to_installed_target(self.target),
            WorthQueryLowerRuntimeExplanationRequestKind::StoreBackedReplayGap {
                reference_set,
                target,
                requested_evidence_families,
                redaction_policy,
                materialization_policy,
            } => WorthQueryExplanationContributionAuthoring::store_backed_replay_gap_explanation(
                semantic_code,
                self.detail,
                reference_set,
                target,
                requested_evidence_families,
                redaction_policy,
                materialization_policy,
            )?
            .bind_to_installed_target(self.target),
        })
    }
}
