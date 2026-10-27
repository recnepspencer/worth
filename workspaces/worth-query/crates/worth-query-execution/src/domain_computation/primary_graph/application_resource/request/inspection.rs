//! Every visible facet uses its own installed purpose before exact row lookup.
mod source_for_test;
mod values;
use super::super::{
    action,
    occurrence::{ResourceCandidateQueue, ResourceLifecycle},
    selector::WorthQueryApplicationResourceReceipt,
    stop::WorthQueryResourceActionDenial,
    storage::{ResourceRowLease, ResourceStorage},
};
use super::*;
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::WorthQueryResourceAccessPurpose as Purpose;

/// Endpoint posture, not a replacement Native request lifecycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryResourceState {
    AdmissionPending,
    AwaitingResult,
    ResultCommitted,
    DomainFailureCommitted,
    RetryPending,
    TimedOut,
    EffectRecoveryPending,
    NativeReleased,
}

/// Cleanup details exposed only through the installed CleanupDetail purpose.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryResourceCleanupDetail {
    pub provider_closed: bool,
    pub queue_drained: bool,
    pub native_released: bool,
}

impl<Schema: ApplicationSchema> WorthQueryApplicationResourcesRequest<'_, '_, '_, Schema> {
    pub(in crate::domain_computation::primary_graph::application_resource::request) fn inspect<
        Binding,
        T,
    >(
        &self,
        selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
        purpose: Purpose,
        pending: impl FnOnce(
            &InstalledResourceBinding<Schema, Binding>,
            EntityId,
            EntityId,
        ) -> Result<
            Option<WorthQueryPendingApplicationQueryGovernance>,
            ResourceCause,
        >,
        inspect: impl FnOnce(&ExecutionRequest<'_, '_>, &ResourceRowLease) -> Result<T, ResourceCause>,
    ) -> Result<T, WorthQueryResourceActionDenial>
    where
        Binding: ApplicationResourceBinding<Schema>,
    {
        self.open(|execution| {
            let installed = self.installed::<Binding>()?;
            action::with_existing(
                self.runtime,
                execution,
                self.external,
                self.request,
                &installed,
                selector,
                purpose,
                |principal, scope| pending(&installed, principal, scope),
                ResourceStorage::take,
                |row, _| inspect(execution, row),
                |row, result| {
                    drop(row);
                    Ok(result)
                },
            )
        })
        .map_err(Self::denial)
    }

    pub fn result_available<Binding: ApplicationResourceBinding<Schema>>(
        &self,
        selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
    ) -> Result<bool, WorthQueryResourceActionDenial> {
        self.inspect(
            selector,
            Purpose::Availability,
            |_, _, _| Ok(None),
            |_, row| Ok(row.occurrence().result.is_some()),
        )
    }

    pub fn result_available_governed<Binding, Capability, Operation, Input>(
        &self,
        selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<bool, WorthQueryResourceActionDenial>
    where
        Binding: ApplicationResourceBinding<Schema>,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = AccessScope<Schema, Binding>>,
    {
        self.inspect(
            selector,
            Purpose::Availability,
            |installed, principal, scope| self.pending(installed, principal, scope, capability),
            |_, row| Ok(row.occurrence().result.is_some()),
        )
    }

    pub fn state<Binding: ApplicationResourceBinding<Schema>>(
        &self,
        selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
    ) -> Result<WorthQueryResourceState, WorthQueryResourceActionDenial> {
        self.inspect(
            selector,
            Purpose::State,
            |_, _, _| Ok(None),
            |_, row| Ok(state(row)),
        )
    }

    pub fn state_governed<Binding, Capability, Operation, Input>(
        &self,
        selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<WorthQueryResourceState, WorthQueryResourceActionDenial>
    where
        Binding: ApplicationResourceBinding<Schema>,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = AccessScope<Schema, Binding>>,
    {
        self.inspect(
            selector,
            Purpose::State,
            |installed, principal, scope| self.pending(installed, principal, scope, capability),
            |_, row| Ok(state(row)),
        )
    }

    pub fn cleanup_detail<Binding: ApplicationResourceBinding<Schema>>(
        &self,
        selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
    ) -> Result<WorthQueryResourceCleanupDetail, WorthQueryResourceActionDenial> {
        self.inspect(
            selector,
            Purpose::CleanupDetail,
            |_, _, _| Ok(None),
            |_, row| Ok(cleanup(row)),
        )
    }

    pub fn cleanup_detail_governed<Binding, Capability, Operation, Input>(
        &self,
        selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<WorthQueryResourceCleanupDetail, WorthQueryResourceActionDenial>
    where
        Binding: ApplicationResourceBinding<Schema>,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = AccessScope<Schema, Binding>>,
    {
        self.inspect(
            selector,
            Purpose::CleanupDetail,
            |installed, principal, scope| self.pending(installed, principal, scope, capability),
            |_, row| Ok(cleanup(row)),
        )
    }
}

fn state(row: &ResourceRowLease) -> WorthQueryResourceState {
    if row
        .occurrence()
        .candidate
        .as_ref()
        .and_then(|candidate| candidate.admission.as_ref())
        .and_then(|admission| admission.report().admitted_completion())
        .is_some_and(|admitted| {
            admitted.completion_class()
                == worth_runtime_bridge::facade::BridgeAsyncCompletionClass::EffectsIndeterminate
        })
    {
        return WorthQueryResourceState::EffectRecoveryPending;
    }
    if let Some(result) = row.occurrence().result.as_ref() {
        return if result.kind == super::super::WorthQueryResourceFrameKind::DomainFailure {
            WorthQueryResourceState::DomainFailureCommitted
        } else {
            WorthQueryResourceState::ResultCommitted
        };
    }
    if let ResourceLifecycle::Live { owner, .. } = &row.occurrence().lifecycle {
        if owner.has_pending_retry() {
            return WorthQueryResourceState::RetryPending;
        }
        if row
            .occurrence()
            .timeout
            .as_ref()
            .and_then(|timeout| timeout.report().timed_out_request())
            .is_some_and(|timeout| timeout.handle() == owner.request_handle())
        {
            return WorthQueryResourceState::TimedOut;
        }
    }
    match row.occurrence().lifecycle {
        ResourceLifecycle::Reserved { .. } => WorthQueryResourceState::AdmissionPending,
        ResourceLifecycle::Live { .. } => WorthQueryResourceState::AwaitingResult,
        ResourceLifecycle::Released(_) => WorthQueryResourceState::NativeReleased,
    }
}

fn cleanup(row: &ResourceRowLease) -> WorthQueryResourceCleanupDetail {
    WorthQueryResourceCleanupDetail {
        provider_closed: row.occurrence().provider_closed,
        queue_drained: row.occurrence().candidate.as_ref().is_none_or(|candidate| {
            !matches!(candidate.queue, ResourceCandidateQueue::Admitted(_))
        }),
        native_released: !matches!(row.occurrence().lifecycle, ResourceLifecycle::Live { .. }),
    }
}
