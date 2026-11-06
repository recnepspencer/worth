use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::counters::QuerySubscriptionDeclarationCounters;
use super::diagnostic::{QuerySubscriptionDiagnosticEvidence, QuerySubscriptionDiagnosticStage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionDeclarationDenialKind {
    UnsupportedMaskedSlice,
    UnsupportedGroupingSlice,
    UnsupportedBoundedMaterializationSlice,
    DeliveryIntentUnsupported,
    SliceBudgetExceeded,
    AllocationBudgetExceeded,
    AmbiguousSliceIntent,
}

impl QuerySubscriptionDeclarationDenialKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnsupportedMaskedSlice => "unsupported_masked_slice",
            Self::UnsupportedGroupingSlice => "unsupported_grouping_slice",
            Self::UnsupportedBoundedMaterializationSlice => {
                "unsupported_bounded_materialization_slice"
            }
            Self::DeliveryIntentUnsupported => "delivery_intent_unsupported",
            Self::SliceBudgetExceeded => "slice_budget_exceeded",
            Self::AllocationBudgetExceeded => "allocation_budget_exceeded",
            Self::AmbiguousSliceIntent => "ambiguous_slice_intent",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionDeclarationDenial {
    denial_kind: QuerySubscriptionDeclarationDenialKind,
    message: String,
    diagnostic: QuerySubscriptionDiagnosticEvidence,
    counters: QuerySubscriptionDeclarationCounters,
}

impl QuerySubscriptionDeclarationDenial {
    pub(super) fn new(
        denial_kind: QuerySubscriptionDeclarationDenialKind,
        message: impl Into<String>,
        diagnostic_stage: QuerySubscriptionDiagnosticStage,
        source_identity: &WorthQueryEvidenceIdentity,
        counters: QuerySubscriptionDeclarationCounters,
    ) -> Result<Self, IdentityDerivationDenial> {
        let message = message.into();
        let diagnostic = QuerySubscriptionDiagnosticEvidence::denied(
            diagnostic_stage,
            message.clone(),
            source_identity,
            &counters.evidence_identity()?,
        )?;
        Ok(Self {
            denial_kind,
            message,
            diagnostic,
            counters,
        })
    }

    pub fn denial_kind(&self) -> &QuerySubscriptionDeclarationDenialKind {
        &self.denial_kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn diagnostic(&self) -> &QuerySubscriptionDiagnosticEvidence {
        &self.diagnostic
    }

    pub fn counters(&self) -> &QuerySubscriptionDeclarationCounters {
        &self.counters
    }
}
