//! Borrowed current-control conditions for one synchronous producer advancement.
use super::*;
use std::time::Instant;
use worth_runtime_world::facade::{PreparedWorldRelationalCutover, ProductBranchObservation};
mod borrowed_read;
mod read_confirmation;

pub(in crate::domain_computation::primary_graph) enum HistoryBorrowedReadStop {
    Busy,
    Navigation(WorthQueryHistoryNavigationDenial),
}

#[derive(Clone, Copy)]
pub struct HistoryProducerInvocation<'invocation> {
    current: &'invocation WorthQueryProductObservationLease,
    request: &'invocation AdmittedRequestCurrentness,
    principal: &'invocation crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding,
    selection: &'invocation HistoryAuthorityLifetime,
    callback: &'invocation HistoryAuthorityLifetime,
}

/// A single physical publication borrows both original admitted requests.
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct HistoryProducerCutover<'call> {
    invocation: HistoryProducerInvocation<'call>,
    producer_request: &'call AdmittedRequestCurrentness,
}

impl HistoryProducerCutover<'_> {
    pub(in crate::domain_computation::primary_graph) fn current(
        &self,
    ) -> &ProductBranchObservation {
        self.invocation.current.observation()
    }

    pub(in crate::domain_computation::primary_graph) fn perform(
        self,
        cutover: PreparedWorldRelationalCutover<'_, '_, '_, '_>,
        expected_target: &ProductBranchObservation,
    ) -> Result<(), WorthQueryHistoryNavigationDenial> {
        self.invocation
            .perform(cutover, expected_target, self.producer_request)
    }
}

/// The attempt carries an advancement's conditions to its commit through
/// this port and never names them.
impl crate::domain_computation::primary_graph::application_attempt::HistoryProducerCommit
    for HistoryProducerInvocation<'_>
{
    fn commit_prepared_application(
        &self,
        provider: &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        producer_request: &AdmittedRequestCurrentness,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        crate::domain_computation::WorthQueryProviderSessionCommitStop,
    > {
        provider.commit_prepared_application_in_history(
            session,
            self.bind_producer_request(producer_request),
            allocation_policy,
        )
    }
}

impl<'invocation> HistoryProducerInvocation<'invocation> {
    pub(in crate::domain_computation::primary_graph) fn bind_producer_request<'call>(
        self,
        producer_request: &'call AdmittedRequestCurrentness,
    ) -> HistoryProducerCutover<'call>
    where
        'invocation: 'call,
    {
        HistoryProducerCutover {
            invocation: self,
            producer_request,
        }
    }

    pub(super) fn new(
        current: &'invocation WorthQueryProductObservationLease,
        request: &'invocation AdmittedRequestCurrentness,
        principal: &'invocation crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding,
        selection: &'invocation HistoryAuthorityLifetime,
        callback: &'invocation HistoryAuthorityLifetime,
    ) -> Self {
        Self {
            current,
            request,
            principal,
            selection,
            callback,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn current(
        &self,
    ) -> &WorthQueryProductObservationLease {
        self.current
    }

    pub(in crate::domain_computation::primary_graph) fn request_lapse(
        &self,
    ) -> Option<crate::domain_computation::authorization::WorthQueryAdmissionLapse> {
        self.request.lapse()
    }

    pub(in crate::domain_computation::primary_graph) fn admits_principal(
        &self,
        principal: &crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding,
    ) -> bool {
        self.principal == principal
    }

    pub(in crate::domain_computation::primary_graph) fn admits_request(
        &self,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> bool {
        self.request.same_request(request)
    }

    /// The Native preparation is already finished. Only the original one-use
    /// World receiver may run under these revocable lifetime guards.
    pub(in crate::domain_computation::primary_graph) fn perform(
        &self,
        cutover: PreparedWorldRelationalCutover<'_, '_, '_, '_>,
        expected_target: &ProductBranchObservation,
        producer_request: &AdmittedRequestCurrentness,
    ) -> Result<(), WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        if cutover.current_read() != self.current.observation()
            || cutover.expected_target() != expected_target
            || !self.request.same_request(producer_request.request())
        {
            return Err(Denial::ForeignDocument);
        }
        let guarded = self.selection.while_live(|| {
            self.callback.while_live(|| {
                if let Some(lapse) = self.request.lapse().or_else(|| producer_request.lapse()) {
                    return Err(lapse);
                }
                let deadline: Instant = self
                    .request
                    .authentication_deadline()
                    .min(producer_request.authentication_deadline());
                cutover.perform_before(deadline);
                Ok(())
            })
        });
        match guarded {
            Ok(Ok(Ok(()))) => Ok(()),
            Ok(Ok(Err(lapse))) => Err(Denial::Authorization(
                WorthQueryOperationAuthorizationDenial::new(
                    lapse.denial_kind(),
                    "history producer request",
                ),
            )),
            _ => Err(Denial::HeadChanged),
        }
    }
}
