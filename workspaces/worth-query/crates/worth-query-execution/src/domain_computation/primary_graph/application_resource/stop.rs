//! Stops preserve the actual failing owner and any already allocated row.
use super::{
    admission::ResourceAdmissionDenial, provider::AttemptIssuanceDenial,
    storage::ResourceStorageDenial, value_capture::ResourceValueDenial,
};
use crate::domain_computation::primary_graph::admitted_capture::CaptureStop;
use worth_execution::WorkCeilingDenial;

/// Coarse refusal category without protected occurrence or Native details.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryResourceStopKind {
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

#[expect(
    dead_code,
    reason = "original typed refusal is retained opaquely; disclosure uses approved projections"
)]
pub(super) enum ResourceCause {
    Unavailable,
    Request(WorkCeilingDenial),
    Control(worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption),
    Advancement(crate::domain_computation::primary_graph::application_contribution::WorthQueryAdvancementDenial),
    Selection(crate::basis::WorthQueryProductBranchAdmissionDenial),
    Principal(crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial),
    Scope(crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial),
    ScopeCapture(Result<crate::domain_computation::WorthQueryProviderSessionDenialKind, crate::domain_computation::WorthQueryProviderSessionControlStopKind>),
    ParameterCapture(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial),
    Admission(ResourceAdmissionDenial),
    Storage(ResourceStorageDenial),
    Value(CaptureStop<ResourceValueDenial>),
    Clock(worth_query_installation::facade::WorthQueryNamedClockFailure),
    Native(worth_runtime_world::facade::RuntimeWorldOwnedAsyncRequestAdmissionDenial),
    Revalidation(worth_runtime_world::facade::RuntimeWorldOwnedAsyncManagedRevalidationDenial),
    Managed(worth_runtime_bridge::facade::BridgeOwnedAsyncManagedDenial),
    Completion(worth_runtime_bridge::facade::BridgeAsyncCompletionRejection),
    DeniedCompletion(worth_runtime_bridge::facade::BridgeOwnedAsyncCompletionAdmission),
    Provider(super::WorthQueryResourceProviderFailure),
    Ingress(super::WorthQueryResourceIngressDenial),
    Scratch(worth_execution::ExecutionRequestArrayDenial),
    Attempt(AttemptIssuanceDenial),
    RecoveryHandoffRequired,
    Recovery(crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial),
    RecoveryAffinity(ResourceRecoveryAffinity),
    EffectTerminal(crate::domain_computation::primary_graph::application_runtime::WorthQueryTerminalEffectRefusal),
    EffectOutbox(crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxReadDenial),
}

pub(super) enum ResourceRecoveryAffinity {
    Runtime,
    Schema,
    Branch,
    Scope,
    OutboxMissing,
}

/// A refused recovery close preserves any handle already returned by cleanup.
pub struct WorthQueryResourceRecoveryCloseStop {
    pub(super) recovery:
        Option<crate::domain_computation::application_aftermath::WorthQueryRecoveryHandle>,
    pub(super) denial: WorthQueryResourceActionDenial,
}

impl WorthQueryResourceRecoveryCloseStop {
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
    pub(super) cause: ResourceCause,
    pub(super) closed: Option<super::closed::WorthQueryClosedResource>,
}

impl WorthQueryResourceActionDenial {
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
        use ResourceCause as C;
        use WorthQueryResourceStopKind as K;
        match &self.cause {
            C::Unavailable => K::Unavailable,
            C::Request(_)
            | C::Control(_)
            | C::Advancement(_)
            | C::Value(CaptureStop::Request(_))
            | C::Admission(
                ResourceAdmissionDenial::Request(_) | ResourceAdmissionDenial::RequestControl(_),
            ) => K::Request,
            C::Storage(ResourceStorageDenial::CleanupPending) => K::Cleanup,
            C::Storage(ResourceStorageDenial::RecoveryHandoffRequired)
            | C::RecoveryHandoffRequired
            | C::Recovery(_)
            | C::RecoveryAffinity(_)
            | C::EffectTerminal(_)
            | C::EffectOutbox(_) => K::Recovery,
            C::Storage(_) => K::Capacity,
            C::Provider(_) | C::Ingress(_) => K::Provider,
            C::Completion(_) | C::DeniedCompletion(_) => K::Completion,
            C::Managed(_) => K::Cleanup,
            _ => K::Admission,
        }
    }

    pub fn execution_stop(&self) -> Option<&WorkCeilingDenial> {
        match &self.cause {
            ResourceCause::Request(cause) | ResourceCause::Value(CaptureStop::Request(cause)) => {
                Some(cause)
            }
            ResourceCause::Admission(ResourceAdmissionDenial::Request(cause)) => Some(cause),
            _ => None,
        }
    }

    /// The failing fresh action requirement, without protected row or Native detail.
    pub fn action_authorization_denial_kind(
        &self,
    ) -> Option<crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind>
    {
        match &self.cause {
            ResourceCause::Admission(ResourceAdmissionDenial::Authorization(cause)) => {
                Some(cause.kind())
            }
            _ => None,
        }
    }

    pub fn request_interruption(
        &self,
    ) -> Option<worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption>
    {
        match &self.cause {
            ResourceCause::Control(cause)
            | ResourceCause::Admission(ResourceAdmissionDenial::RequestControl(cause)) => {
                Some(*cause)
            }
            _ => None,
        }
    }

    pub fn advancement_denial(&self) -> Option<&crate::domain_computation::primary_graph::application_contribution::WorthQueryAdvancementDenial>{
        match &self.cause {
            ResourceCause::Advancement(cause) => Some(cause),
            _ => None,
        }
    }

    pub fn selection_denial(
        &self,
    ) -> Option<&crate::basis::WorthQueryProductBranchAdmissionDenial> {
        match &self.cause {
            ResourceCause::Selection(cause) => Some(cause),
            _ => None,
        }
    }

    pub fn recovery_denial(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial>
    {
        match &self.cause {
            ResourceCause::Recovery(cause) => Some(cause),
            _ => None,
        }
    }

    /// The actual current terminal/outcome owner's refusal, separate from a
    /// retained original outbox observation or provider timeout.
    pub fn effect_admission_denial(&self) -> Option<WorthQueryResourceEffectRefusal> {
        use crate::domain_computation::primary_graph::application_runtime::WorthQueryTerminalEffectRefusal as Cause;
        match &self.cause {
            ResourceCause::EffectTerminal(Cause::AlreadyCompleted) => {
                Some(WorthQueryResourceEffectRefusal::AlreadyCompleted)
            }
            ResourceCause::EffectTerminal(Cause::TerminalIndexUnavailable) => {
                Some(WorthQueryResourceEffectRefusal::TerminalIndexUnavailable)
            }
            _ => None,
        }
    }
}

/// A refused owned-effect start returns its allocated row or original recovery handle.
pub struct WorthQueryResourceOwnedEffectStartStop<Schema, Binding>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
    Binding:
        worth_query_declaration::facade::application_resource::ApplicationResourceBinding<Schema>,
{
    pub(super) resource: WorthQueryResourceStartStop<Schema, Binding>,
    pub(super) recovery:
        Option<crate::domain_computation::application_aftermath::WorthQueryRecoveryHandle>,
}

impl<Schema, Binding> WorthQueryResourceOwnedEffectStartStop<Schema, Binding>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
    Binding:
        worth_query_declaration::facade::application_resource::ApplicationResourceBinding<Schema>,
{
    pub fn into_parts(
        self,
    ) -> (
        Option<super::selector::WorthQueryApplicationResource<Schema, Binding>>,
        Option<crate::domain_computation::application_aftermath::WorthQueryRecoveryHandle>,
        WorthQueryResourceActionDenial,
    ) {
        (self.resource.handle, self.recovery, self.resource.denial)
    }
}

impl std::fmt::Debug for WorthQueryResourceActionDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryResourceActionDenial")
            .field("kind", &self.kind())
            .finish_non_exhaustive()
    }
}
impl From<ResourceCause> for WorthQueryResourceActionDenial {
    fn from(cause: ResourceCause) -> Self {
        Self {
            cause,
            closed: None,
        }
    }
}

/// A refused Start may already own real clock, Native or provider obligations.
/// Such a row is returned through the same move-only handle for fresh cleanup.
pub struct WorthQueryResourceStartStop<Schema, Binding>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
    Binding:
        worth_query_declaration::facade::application_resource::ApplicationResourceBinding<Schema>,
{
    pub(super) handle: Option<super::selector::WorthQueryApplicationResource<Schema, Binding>>,
    pub(super) denial: WorthQueryResourceActionDenial,
}

impl<Schema, Binding> WorthQueryResourceStartStop<Schema, Binding>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
    Binding:
        worth_query_declaration::facade::application_resource::ApplicationResourceBinding<Schema>,
{
    pub fn into_parts(
        self,
    ) -> (
        Option<super::selector::WorthQueryApplicationResource<Schema, Binding>>,
        WorthQueryResourceActionDenial,
    ) {
        (self.handle, self.denial)
    }
}
