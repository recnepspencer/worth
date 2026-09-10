use super::{
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolCounters,
    WorthQueryProviderSessionRecoveryPosture, WorthQueryProviderSessionSettlementDeferred,
};

/// Provider-authored text describing a completed physical transition.
///
/// This value is deliberately descriptive only. It is never parsed and cannot
/// register, locate, commit, abort, clean up, or readmit a provider session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProviderTerminalDescription(std::sync::Arc<str>);

impl WorthQueryProviderTerminalDescription {
    pub fn new(description: impl Into<std::sync::Arc<str>>) -> Result<Self, &'static str> {
        let description = description.into();
        if description.trim().is_empty() || description.trim() != description.as_ref() {
            return Err("provider terminal description must be non-empty and trimmed");
        }
        Ok(Self(description))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub struct WorthQueryClosedProviderSessionDisposition {
    provider_description: WorthQueryProviderTerminalDescription,
    counters: WorthQueryProviderSessionProtocolCounters,
    terminal_binding: super::WorthQueryProviderSessionTerminalBinding,
}

#[derive(Debug)]
pub enum WorthQuerySessionCommitOrAbortOutcome {
    ProductStale(crate::domain_computation::WorthQueryProductStaleApplication),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    NoEffect(worth_runtime_world::facade::NoEffectCompositePublication),
    Committed(WorthQueryClosedProviderSessionDisposition),
    Aborted(WorthQueryClosedProviderSessionDisposition),
    CommitDeferred(super::WorthQueryProviderSessionCommitDeferred),
    CommitControlStopped(super::WorthQueryProviderSessionCommitControlStopped),
    CommitSettlementDeferred(WorthQueryProviderSessionSettlementDeferred),
    CommitRecoveryRequired(WorthQueryProviderSessionFailure),
    AbortRecoveryRequired(WorthQueryProviderSessionFailure),
}

impl WorthQuerySessionCommitOrAbortOutcome {
    pub fn recovery_posture(&self) -> WorthQueryProviderSessionRecoveryPosture {
        match self {
            Self::Committed(_)
            | Self::ProductStale(_)
            | Self::NoEffect(_)
            | Self::Aborted(_)
            | Self::CommitDeferred(_)
            | Self::CommitControlStopped(_) => WorthQueryProviderSessionRecoveryPosture::Closed,
            Self::CommitSettlementDeferred(_)
            | Self::ProductUnpublished(_)
            | Self::CommitRecoveryRequired(_)
            | Self::AbortRecoveryRequired(_) => {
                WorthQueryProviderSessionRecoveryPosture::RecoveryRequired
            }
        }
    }

    pub fn failure(&self) -> Option<&WorthQueryProviderSessionFailure> {
        match self {
            Self::CommitRecoveryRequired(failure) | Self::AbortRecoveryRequired(failure) => {
                Some(failure)
            }
            Self::Committed(_)
            | Self::ProductStale(_)
            | Self::NoEffect(_)
            | Self::ProductUnpublished(_)
            | Self::Aborted(_)
            | Self::CommitDeferred(_)
            | Self::CommitControlStopped(_)
            | Self::CommitSettlementDeferred(_) => None,
        }
    }

    pub fn settlement_deferred(&self) -> Option<&WorthQueryProviderSessionSettlementDeferred> {
        match self {
            Self::CommitSettlementDeferred(deferred) => Some(deferred),
            Self::Committed(_)
            | Self::ProductStale(_)
            | Self::NoEffect(_)
            | Self::ProductUnpublished(_)
            | Self::Aborted(_)
            | Self::CommitDeferred(_)
            | Self::CommitControlStopped(_)
            | Self::CommitRecoveryRequired(_)
            | Self::AbortRecoveryRequired(_) => None,
        }
    }

    pub fn control_stopped(&self) -> Option<&super::WorthQueryProviderSessionCommitControlStopped> {
        match self {
            Self::CommitControlStopped(stopped) => Some(stopped),
            _ => None,
        }
    }
}

impl WorthQueryClosedProviderSessionDisposition {
    pub(in crate::domain_computation::provider_session::protocol) fn close(
        provider_description: WorthQueryProviderTerminalDescription,
        counters: WorthQueryProviderSessionProtocolCounters,
        terminal_binding: super::WorthQueryProviderSessionTerminalBinding,
    ) -> Self {
        Self {
            provider_description,
            counters,
            terminal_binding,
        }
    }

    pub fn provider_description(&self) -> &WorthQueryProviderTerminalDescription {
        &self.provider_description
    }

    pub fn counters(&self) -> WorthQueryProviderSessionProtocolCounters {
        self.counters
    }

    pub(in crate::domain_computation) const fn terminal_binding(
        &self,
    ) -> &super::WorthQueryProviderSessionTerminalBinding {
        &self.terminal_binding
    }
}
