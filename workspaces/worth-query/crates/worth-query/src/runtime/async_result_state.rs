use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_runtime_bridge::facade::{
    BridgeAsyncCompletionClass, BridgeAsyncCompletionDenialClass, BridgeAsyncCompletionState,
    BridgeAsyncCompletionSupersessionClass, BridgeAsyncForwardCausalityClass,
    BridgeMixedCauseAsyncResultCause, BridgeMixedCauseAsyncResultTransition,
};

use crate::evidence_identity::WorthQueryEvidenceIdentity;

use super::async_result_identity::{
    runtime_async_causality_from_bridge, runtime_async_causality_identity,
    runtime_async_causality_label_identity, runtime_async_result_state_identity,
};
#[cfg(test)]
pub(crate) use super::async_result_identity::{
    runtime_async_causality_from_label, runtime_async_checkpoint_label_identity,
};
use super::WorthQueryRuntimeStateKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryRuntimeAsyncResultStateKind {
    Pending,
    Current,
    Failed,
    Stale,
    Cancelled,
    Retried,
    Revalidating,
    Superseded,
    Denied,
    Unresolved,
}

impl WorthQueryRuntimeAsyncResultStateKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Current => "current",
            Self::Failed => "failed",
            Self::Stale => "stale",
            Self::Cancelled => "cancelled",
            Self::Retried => "retried",
            Self::Revalidating => "revalidating",
            Self::Superseded => "superseded",
            Self::Denied => "denied",
            Self::Unresolved => "unresolved",
        }
    }

    pub(crate) fn permits_basis_or_generation_drift(self) -> bool {
        matches!(
            self,
            Self::Stale | Self::Superseded | Self::Denied | Self::Unresolved
        )
    }

    pub(crate) fn state_kind(self) -> WorthQueryRuntimeStateKind {
        match self {
            Self::Pending => WorthQueryRuntimeStateKind::Pending,
            Self::Current => WorthQueryRuntimeStateKind::Ready,
            Self::Failed => WorthQueryRuntimeStateKind::Failed,
            Self::Stale => WorthQueryRuntimeStateKind::Stale,
            Self::Cancelled => WorthQueryRuntimeStateKind::Cancelled,
            Self::Retried => WorthQueryRuntimeStateKind::Retried,
            Self::Revalidating => WorthQueryRuntimeStateKind::Revalidating,
            Self::Superseded => WorthQueryRuntimeStateKind::Superseded,
            Self::Denied => WorthQueryRuntimeStateKind::Denied,
            Self::Unresolved => WorthQueryRuntimeStateKind::Unresolved,
        }
    }
}

impl std::fmt::Display for WorthQueryRuntimeAsyncResultStateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryRuntimeAsyncResultProjection {
    Pending {
        causality_identity: WorthQueryEvidenceIdentity,
    },
    CompletionState {
        state: BridgeAsyncCompletionState,
        causality_identity: WorthQueryEvidenceIdentity,
    },
    ForwardCausality {
        class: BridgeAsyncForwardCausalityClass,
        causality_identity: WorthQueryEvidenceIdentity,
    },
    ClassifiedSupersession {
        class: BridgeAsyncCompletionSupersessionClass,
        causality_identity: WorthQueryEvidenceIdentity,
    },
}

impl WorthQueryRuntimeAsyncResultProjection {
    pub(super) fn kind(&self) -> WorthQueryRuntimeAsyncResultStateKind {
        match self {
            Self::Pending { .. } => WorthQueryRuntimeAsyncResultStateKind::Pending,
            Self::CompletionState { state, .. } => match state {
                BridgeAsyncCompletionState::Admitted(BridgeAsyncCompletionClass::Fulfilled) => {
                    WorthQueryRuntimeAsyncResultStateKind::Current
                }
                BridgeAsyncCompletionState::Admitted(
                    BridgeAsyncCompletionClass::EffectsIndeterminate,
                ) => WorthQueryRuntimeAsyncResultStateKind::Unresolved,
                BridgeAsyncCompletionState::Denied(BridgeAsyncCompletionDenialClass::Rejected)
                | BridgeAsyncCompletionState::Denied(BridgeAsyncCompletionDenialClass::TimedOut) => {
                    WorthQueryRuntimeAsyncResultStateKind::Failed
                }
                BridgeAsyncCompletionState::Denied(BridgeAsyncCompletionDenialClass::Cancelled) => {
                    WorthQueryRuntimeAsyncResultStateKind::Cancelled
                }
                BridgeAsyncCompletionState::Denied(
                    BridgeAsyncCompletionDenialClass::StaleDenied,
                ) => WorthQueryRuntimeAsyncResultStateKind::Stale,
                BridgeAsyncCompletionState::Denied(
                    BridgeAsyncCompletionDenialClass::Superseded,
                ) => WorthQueryRuntimeAsyncResultStateKind::Superseded,
                BridgeAsyncCompletionState::Denied(
                    BridgeAsyncCompletionDenialClass::SignalLifecycleDenied,
                ) => WorthQueryRuntimeAsyncResultStateKind::Denied,
            },
            Self::ForwardCausality { class, .. } => match class {
                BridgeAsyncForwardCausalityClass::RetryAfterTimeout
                | BridgeAsyncForwardCausalityClass::RetryAfterCancellation => {
                    WorthQueryRuntimeAsyncResultStateKind::Retried
                }
                BridgeAsyncForwardCausalityClass::RevalidationAfterTruthBasisDrift
                | BridgeAsyncForwardCausalityClass::RevalidationAfterPreviewBasisDrift
                | BridgeAsyncForwardCausalityClass::RevalidationAfterSubscriptionInstanceDrift => {
                    WorthQueryRuntimeAsyncResultStateKind::Revalidating
                }
            },
            Self::ClassifiedSupersession { class, .. } => match class {
                BridgeAsyncCompletionSupersessionClass::TruthBasisSuperseded
                | BridgeAsyncCompletionSupersessionClass::BranchDrifted
                | BridgeAsyncCompletionSupersessionClass::PreviewBasisDrifted
                | BridgeAsyncCompletionSupersessionClass::SubscriptionInstanceSuperseded => {
                    WorthQueryRuntimeAsyncResultStateKind::Stale
                }
                BridgeAsyncCompletionSupersessionClass::PreviewDiscarded
                | BridgeAsyncCompletionSupersessionClass::SignalGenerationSuperseded => {
                    WorthQueryRuntimeAsyncResultStateKind::Superseded
                }
            },
        }
    }

    pub(super) fn causality_identity(&self) -> &WorthQueryEvidenceIdentity {
        match self {
            Self::Pending { causality_identity } => causality_identity,
            Self::CompletionState {
                causality_identity, ..
            } => causality_identity,
            Self::ForwardCausality {
                causality_identity, ..
            } => causality_identity,
            Self::ClassifiedSupersession {
                causality_identity, ..
            } => causality_identity,
        }
    }

    pub(super) fn from_bridge_transition(
        transition: &BridgeMixedCauseAsyncResultTransition,
    ) -> Result<Self, IdentityDerivationDenial> {
        let causality_identity = runtime_async_causality_from_bridge(
            transition.source_identity(),
            transition.source_digest(),
        )?;
        Ok(match transition.cause() {
            BridgeMixedCauseAsyncResultCause::Completion(state) => Self::CompletionState {
                state,
                causality_identity,
            },
            BridgeMixedCauseAsyncResultCause::ClassifiedDenied { supersession, .. } => {
                Self::ClassifiedSupersession {
                    class: supersession,
                    causality_identity,
                }
            }
            BridgeMixedCauseAsyncResultCause::Retry(class)
            | BridgeMixedCauseAsyncResultCause::Revalidation(class) => Self::ForwardCausality {
                class,
                causality_identity,
            },
        })
    }

    pub(super) fn stale_before_bridge_revalidation(
        transition: &BridgeMixedCauseAsyncResultTransition,
    ) -> Result<Self, IdentityDerivationDenial> {
        let class = match transition.cause() {
            BridgeMixedCauseAsyncResultCause::Revalidation(
                BridgeAsyncForwardCausalityClass::RevalidationAfterTruthBasisDrift,
            ) => BridgeAsyncCompletionSupersessionClass::TruthBasisSuperseded,
            BridgeMixedCauseAsyncResultCause::Revalidation(
                BridgeAsyncForwardCausalityClass::RevalidationAfterPreviewBasisDrift,
            ) => BridgeAsyncCompletionSupersessionClass::PreviewBasisDrifted,
            BridgeMixedCauseAsyncResultCause::Revalidation(
                BridgeAsyncForwardCausalityClass::RevalidationAfterSubscriptionInstanceDrift,
            ) => BridgeAsyncCompletionSupersessionClass::SubscriptionInstanceSuperseded,
            _ => {
                unreachable!("stale precursor is created only for Bridge revalidation lineage")
            }
        };
        Ok(Self::ClassifiedSupersession {
            class,
            causality_identity: runtime_async_causality_from_bridge(
                transition.source_identity(),
                transition.source_digest(),
            )?,
        })
    }
    pub(crate) fn pending(causality_label: &str) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::Pending {
            causality_identity: runtime_async_causality_identity(
                &runtime_async_causality_label_identity(causality_label)?,
            )?,
        })
    }

    #[cfg(test)]
    pub(crate) fn completion_state(
        state: BridgeAsyncCompletionState,
        causality_label: &str,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::CompletionState {
            state,
            causality_identity: runtime_async_causality_identity(
                &runtime_async_causality_label_identity(causality_label)?,
            )?,
        })
    }

    #[cfg(test)]
    pub(crate) fn forward_causality(
        class: BridgeAsyncForwardCausalityClass,
        causality_label: &str,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::ForwardCausality {
            class,
            causality_identity: runtime_async_causality_identity(
                &runtime_async_causality_label_identity(causality_label)?,
            )?,
        })
    }

    #[cfg(test)]
    pub(crate) fn supersession(causality_label: &str) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::ClassifiedSupersession {
            class: BridgeAsyncCompletionSupersessionClass::SignalGenerationSuperseded,
            causality_identity: runtime_async_causality_identity(
                &runtime_async_causality_label_identity(causality_label)?,
            )?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimeAsyncResultState {
    kind: WorthQueryRuntimeAsyncResultStateKind,
    causality_identity: WorthQueryEvidenceIdentity,
    basis_identity: WorthQueryEvidenceIdentity,
    checkpoint_identity: WorthQueryEvidenceIdentity,
    result_state_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryRuntimeAsyncResultState {
    pub fn new(
        kind: WorthQueryRuntimeAsyncResultStateKind,
        causality_identity: &WorthQueryEvidenceIdentity,
        basis_identity: &WorthQueryEvidenceIdentity,
        checkpoint_identity: &WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        let causality_identity = causality_identity.clone();
        let basis_identity = basis_identity.clone();
        let checkpoint_identity = checkpoint_identity.clone();
        let result_state_identity = runtime_async_result_state_identity(
            kind,
            &causality_identity,
            &basis_identity,
            &checkpoint_identity,
        )?;
        Ok(Self {
            kind,
            causality_identity,
            basis_identity,
            checkpoint_identity,
            result_state_identity,
        })
    }

    pub fn kind(&self) -> WorthQueryRuntimeAsyncResultStateKind {
        self.kind
    }

    pub fn causality_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.causality_identity
    }

    pub fn causality_for_reporting(&self) -> &str {
        self.causality_identity.as_str()
    }

    pub fn basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_identity
    }

    pub fn basis_for_reporting(&self) -> &str {
        self.basis_identity.as_str()
    }

    pub fn checkpoint_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.checkpoint_identity
    }

    pub fn checkpoint_for_reporting(&self) -> &str {
        self.checkpoint_identity.as_str()
    }

    pub fn result_state_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.result_state_identity
    }

    pub fn result_state_for_reporting(&self) -> &str {
        self.result_state_identity.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effects_indeterminate_completion_maps_only_to_unresolved(
    ) -> Result<(), IdentityDerivationDenial> {
        let projection = WorthQueryRuntimeAsyncResultProjection::completion_state(
            BridgeAsyncCompletionState::Admitted(BridgeAsyncCompletionClass::EffectsIndeterminate),
            "owner-effects-indeterminate",
        )?;

        assert_eq!(
            projection.kind(),
            WorthQueryRuntimeAsyncResultStateKind::Unresolved
        );
        assert!(projection.kind().permits_basis_or_generation_drift());

        Ok(())
    }
}
