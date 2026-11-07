use crate::domain_capabilities::denials::{
    WorthQueryDomainCapabilityProgressionDenial, WorthQueryDomainCapabilityProgressionFailure,
    WorthQueryDomainCapabilityRebindRequired, WorthQueryDomainCapabilityStale,
};
use crate::domain_capabilities::{
    WorthQueryDomainCapabilityTargetKind, WorthQueryDomainCapabilityTransitionOutcome,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDomainCapabilityOutcomeKind {
    Materialized,
    Denied,
    Stale,
    RebindRequired,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryDomainCapabilityMaterializationError {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Denied(WorthQueryDomainCapabilityProgressionDenial),
    Stale(WorthQueryDomainCapabilityStale),
    RebindRequired(WorthQueryDomainCapabilityRebindRequired),
    Failed(WorthQueryDomainCapabilityProgressionFailure),
}

impl WorthQueryDomainCapabilityMaterializationError {
    pub fn kind(&self) -> WorthQueryDomainCapabilityOutcomeKind {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => {
                WorthQueryDomainCapabilityOutcomeKind::Failed
            }
            Self::Denied(_) => WorthQueryDomainCapabilityOutcomeKind::Denied,
            Self::Stale(_) => WorthQueryDomainCapabilityOutcomeKind::Stale,
            Self::RebindRequired(_) => WorthQueryDomainCapabilityOutcomeKind::RebindRequired,
            Self::Failed(_) => WorthQueryDomainCapabilityOutcomeKind::Failed,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCheckedDomainCapabilityOutcome<T> {
    category: &'static str,
    target_kind: WorthQueryDomainCapabilityTargetKind,
    semantic_posture: &'static str,
    inner: WorthQueryDomainCapabilityTransitionOutcome<T>,
}

impl<T> WorthQueryCheckedDomainCapabilityOutcome<T> {
    pub(crate) fn from_transition_outcome(
        category: &'static str,
        target_kind: WorthQueryDomainCapabilityTargetKind,
        semantic_posture: &'static str,
        inner: WorthQueryDomainCapabilityTransitionOutcome<T>,
    ) -> Self {
        Self {
            category,
            target_kind,
            semantic_posture,
            inner,
        }
    }

    pub fn category(&self) -> &str {
        self.category
    }

    pub fn target_kind(&self) -> WorthQueryDomainCapabilityTargetKind {
        self.target_kind
    }

    pub fn semantic_posture(&self) -> &str {
        self.semantic_posture
    }

    pub fn kind(&self) -> WorthQueryDomainCapabilityOutcomeKind {
        match self.inner {
            worth_proof::TransitionOutcome::Success(_) => {
                WorthQueryDomainCapabilityOutcomeKind::Materialized
            }
            worth_proof::TransitionOutcome::Denied(_) => {
                WorthQueryDomainCapabilityOutcomeKind::Denied
            }
            worth_proof::TransitionOutcome::Stale(_) => {
                WorthQueryDomainCapabilityOutcomeKind::Stale
            }
            worth_proof::TransitionOutcome::RebindRequired(_) => {
                WorthQueryDomainCapabilityOutcomeKind::RebindRequired
            }
            worth_proof::TransitionOutcome::Failed(_) => {
                WorthQueryDomainCapabilityOutcomeKind::Failed
            }
            worth_proof::TransitionOutcome::Deferred(never) => match never {},
        }
    }

    pub fn transition_outcome(&self) -> &WorthQueryDomainCapabilityTransitionOutcome<T> {
        &self.inner
    }

    pub fn into_transition_outcome(self) -> WorthQueryDomainCapabilityTransitionOutcome<T> {
        self.inner
    }

    pub fn materialized(&self) -> Option<&T> {
        match &self.inner {
            worth_proof::TransitionOutcome::Success(value) => Some(value),
            _ => None,
        }
    }

    pub fn denial(&self) -> Option<&WorthQueryDomainCapabilityProgressionDenial> {
        match &self.inner {
            worth_proof::TransitionOutcome::Denied(value) => Some(value),
            _ => None,
        }
    }

    pub fn stale(&self) -> Option<&WorthQueryDomainCapabilityStale> {
        match &self.inner {
            worth_proof::TransitionOutcome::Stale(value) => Some(value),
            _ => None,
        }
    }

    pub fn rebind_required(&self) -> Option<&WorthQueryDomainCapabilityRebindRequired> {
        match &self.inner {
            worth_proof::TransitionOutcome::RebindRequired(value) => Some(value),
            _ => None,
        }
    }

    pub fn failure(&self) -> Option<&WorthQueryDomainCapabilityProgressionFailure> {
        match &self.inner {
            worth_proof::TransitionOutcome::Failed(value) => Some(value),
            _ => None,
        }
    }

    pub fn into_result(self) -> Result<T, WorthQueryDomainCapabilityMaterializationError> {
        match self.inner {
            worth_proof::TransitionOutcome::Success(value) => Ok(value),
            worth_proof::TransitionOutcome::Denied(value) => Err(
                WorthQueryDomainCapabilityMaterializationError::Denied(value),
            ),
            worth_proof::TransitionOutcome::Stale(value) => {
                Err(WorthQueryDomainCapabilityMaterializationError::Stale(value))
            }
            worth_proof::TransitionOutcome::RebindRequired(value) => {
                Err(WorthQueryDomainCapabilityMaterializationError::RebindRequired(value))
            }
            worth_proof::TransitionOutcome::Failed(value) => Err(
                WorthQueryDomainCapabilityMaterializationError::Failed(value),
            ),
            worth_proof::TransitionOutcome::Deferred(never) => match never {},
        }
    }
}

impl From<IdentityDerivationDenial> for WorthQueryDomainCapabilityMaterializationError {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}
