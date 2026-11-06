use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::counters::QuerySubscriptionDeclarationCounters;
use super::diagnostic::{QuerySubscriptionDiagnosticEvidence, QuerySubscriptionDiagnosticStage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionFamilySelectionFailureClass {
    ViewFamilyLiveFamilyMismatch,
    InvalidAdmissionDimensions,
    WorkBudgetExceeded,
    AllocationBudgetExceeded,
    UnknownSubscriptionCost,
    UnsupportedViewFamily,
    UnsupportedTemporalLiveShape,
    UnsupportedAsyncLiveShape,
    FutureLiveBasisUnsupported,
    HostObserverInferenceForbidden,
    RawCdcFallbackForbidden,
    GenericSubscriptionKindForbidden,
    RelationshipProofAdmissionDrift,
}

impl QuerySubscriptionFamilySelectionFailureClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ViewFamilyLiveFamilyMismatch => "view_family_live_family_mismatch",
            Self::InvalidAdmissionDimensions => "invalid_admission_dimensions",
            Self::WorkBudgetExceeded => "work_budget_exceeded",
            Self::AllocationBudgetExceeded => "allocation_budget_exceeded",
            Self::UnknownSubscriptionCost => "unknown_subscription_cost",
            Self::UnsupportedViewFamily => "unsupported_view_family",
            Self::UnsupportedTemporalLiveShape => "unsupported_temporal_live_shape",
            Self::UnsupportedAsyncLiveShape => "unsupported_async_live_shape",
            Self::FutureLiveBasisUnsupported => "future_live_basis_unsupported",
            Self::HostObserverInferenceForbidden => "host_observer_inference_forbidden",
            Self::RawCdcFallbackForbidden => "raw_cdc_fallback_forbidden",
            Self::GenericSubscriptionKindForbidden => "generic_subscription_kind_forbidden",
            Self::RelationshipProofAdmissionDrift => "relationship_proof_admission_drift",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionFamilySelectionError {
    failure_class: QuerySubscriptionFamilySelectionFailureClass,
    message: String,
    diagnostic: Result<QuerySubscriptionDiagnosticEvidence, IdentityDerivationDenial>,
    counters: QuerySubscriptionDeclarationCounters,
}

impl QuerySubscriptionFamilySelectionError {
    pub(super) fn new(
        failure_class: QuerySubscriptionFamilySelectionFailureClass,
        message: impl Into<String>,
        diagnostic_stage: QuerySubscriptionDiagnosticStage,
        source_identity: &WorthQueryEvidenceIdentity,
        counters: QuerySubscriptionDeclarationCounters,
    ) -> Self {
        let message = message.into();
        let diagnostic = (|| {
            Ok(QuerySubscriptionDiagnosticEvidence::denied(
                diagnostic_stage,
                message.clone(),
                source_identity,
                &counters.evidence_identity()?,
            )?)
        })();
        Self {
            failure_class,
            message,
            diagnostic,
            counters,
        }
    }

    pub fn failure_class(&self) -> &QuerySubscriptionFamilySelectionFailureClass {
        &self.failure_class
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn diagnostic(
        &self,
    ) -> Result<&QuerySubscriptionDiagnosticEvidence, IdentityDerivationDenial> {
        Ok(self.diagnostic.as_ref().map_err(|cause| *cause)?)
    }

    pub fn counters(&self) -> &QuerySubscriptionDeclarationCounters {
        &self.counters
    }
}
