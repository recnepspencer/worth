//! Stops preserve the actual failing owner and any already allocated row.
use worth_execution::WorkCeilingDenial;

/// Coarse refusal category without protected occurrence or Native details.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryResourceStopKind {
    StaleIncarnation,
    Unavailable,
    Request,
    Admission,
    Capacity,
    Provider,
    Completion,
    Cleanup,
    Recovery,
}

/// A projection of the genuine current terminal/outcome owner's refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryResourceEffectRefusal {
    AlreadyCompleted,
    TerminalIndexUnavailable,
}

/// Sealed upper-layer cause carrier behind `WorthQueryResourceActionDenial`.
pub trait ResourceCausePort: Send + Sync {
    fn kind(&self) -> WorthQueryResourceStopKind;
    fn execution_stop(&self) -> Option<&WorkCeilingDenial>;
    fn action_authorization_denial_kind(
        &self,
    ) -> Option<crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind>;
    fn request_interruption(
        &self,
    ) -> Option<worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption>;
    fn advancement_denial(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::application_contribution::WorthQueryAdvancementDenial>;
    fn selection_denial(&self) -> Option<&crate::basis::WorthQueryProductBranchAdmissionDenial>;
    fn recovery_denial(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial>;
    fn effect_admission_denial(&self) -> Option<WorthQueryResourceEffectRefusal>;
}

/// A refused recovery close preserves any handle already returned by cleanup.
pub struct WorthQueryResourceRecoveryCloseStop {
    pub(crate) recovery:
        Option<crate::domain_computation::application_aftermath::WorthQueryRecoveryHandle>,
    pub(crate) denial: WorthQueryResourceActionDenial,
}

impl WorthQueryResourceRecoveryCloseStop {
    pub fn from_parts(
        recovery: Option<
            crate::domain_computation::application_aftermath::WorthQueryRecoveryHandle,
        >,
        denial: WorthQueryResourceActionDenial,
    ) -> Self {
        Self { recovery, denial }
    }

    pub fn into_parts(
        self,
    ) -> (
        Option<crate::domain_computation::application_aftermath::WorthQueryRecoveryHandle>,
        WorthQueryResourceActionDenial,
    ) {
        (self.recovery, self.denial)
    }
}

/// The exact original owner cause is retained. No display identity is promoted
/// to a replacement request or cleanup capability.
pub struct WorthQueryResourceActionDenial {
    cause: Box<dyn ResourceCausePort>,
    closed: Option<super::closed::WorthQueryClosedResource>,
}

impl WorthQueryResourceActionDenial {
    pub fn from_cause_port(
        cause: Box<dyn ResourceCausePort>,
        closed: Option<super::closed::WorthQueryClosedResource>,
    ) -> Self {
        Self { cause, closed }
    }

    pub fn with_closed(mut self, closed: Option<super::closed::WorthQueryClosedResource>) -> Self {
        self.closed = closed;
        self
    }

    pub fn into_closed(self) -> (Option<super::closed::WorthQueryClosedResource>, Self) {
        let Self { cause, closed } = self;
        (
            closed,
            Self {
                cause,
                closed: None,
            },
        )
    }

    pub fn kind(&self) -> WorthQueryResourceStopKind {
        self.cause.kind()
    }

    pub fn execution_stop(&self) -> Option<&WorkCeilingDenial> {
        self.cause.execution_stop()
    }

    /// The failing fresh action requirement, without protected row or Native detail.
    pub fn action_authorization_denial_kind(
        &self,
    ) -> Option<crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind>
    {
        self.cause.action_authorization_denial_kind()
    }

    pub fn request_interruption(
        &self,
    ) -> Option<worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption>
    {
        self.cause.request_interruption()
    }

    pub fn advancement_denial(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::application_contribution::WorthQueryAdvancementDenial>
{
        self.cause.advancement_denial()
    }

    pub fn selection_denial(
        &self,
    ) -> Option<&crate::basis::WorthQueryProductBranchAdmissionDenial> {
        self.cause.selection_denial()
    }

    pub fn recovery_denial(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial>
    {
        self.cause.recovery_denial()
    }

    /// The actual current terminal/outcome owner's refusal, separate from a
    /// retained original outbox observation or provider timeout.
    pub fn effect_admission_denial(&self) -> Option<WorthQueryResourceEffectRefusal> {
        self.cause.effect_admission_denial()
    }
}

impl std::fmt::Debug for WorthQueryResourceActionDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryResourceActionDenial")
            .field("kind", &self.kind())
            .finish_non_exhaustive()
    }
}
