use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::counters::QuerySubscriptionDeclarationCounters;
use super::diagnostic::{QuerySubscriptionDiagnosticEvidence, QuerySubscriptionDiagnosticStage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionBridgeLoweringDenialKind {
    BridgeFamilyUnsupported,
    BridgeSliceUnsupported,
    BridgeFallbackUnsupported,
    BasisBindingUnsupported,
    LoweringBudgetExceeded,
}

impl QuerySubscriptionBridgeLoweringDenialKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BridgeFamilyUnsupported => "bridge_family_unsupported",
            Self::BridgeSliceUnsupported => "bridge_slice_unsupported",
            Self::BridgeFallbackUnsupported => "bridge_fallback_unsupported",
            Self::BasisBindingUnsupported => "basis_binding_unsupported",
            Self::LoweringBudgetExceeded => "lowering_budget_exceeded",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionBridgeLoweringError {
    denial_kind: QuerySubscriptionBridgeLoweringDenialKind,
    message: String,
    diagnostic: QuerySubscriptionDiagnosticEvidence,
    counters: QuerySubscriptionDeclarationCounters,
}

impl QuerySubscriptionBridgeLoweringError {
    pub(super) fn new(
        denial_kind: QuerySubscriptionBridgeLoweringDenialKind,
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
            &counters.evidence_identity(),
        )?;
        Ok(Self {
            denial_kind,
            message,
            diagnostic,
            counters,
        })
    }

    pub fn denial_kind(&self) -> &QuerySubscriptionBridgeLoweringDenialKind {
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
