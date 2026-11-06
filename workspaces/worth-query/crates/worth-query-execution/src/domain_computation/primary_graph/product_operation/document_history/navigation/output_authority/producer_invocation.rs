//! Borrowed current-control conditions for one synchronous producer advancement.
use super::*;
use std::time::Instant;
use worth_runtime_world::facade::{PreparedWorldRelationalCutover, ProductBranchObservation};
mod borrowed_read;
mod read_confirmation;
mod source_presence;

pub(in crate::domain_computation::primary_graph) enum HistoryBorrowedReadStop {
    Busy,
    Navigation(WorthQueryHistoryNavigationDenial),
}

#[derive(Clone, Copy)]
pub struct HistoryProducerInvocation<'invocation> {
    current: &'invocation WorthQueryProductObservationLease,
    request: &'invocation AdmittedRequestCurrentness,
    principal: &'invocation crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    selection: &'invocation HistoryAuthorityLifetime,
    callback: &'invocation HistoryAuthorityLifetime,
    selected_graph: Option<&'invocation worth_relational::facade::mvcc::RelationalBranchObservation>,
    restored: &'invocation ProductBranchObservation,
    family: &'static str,
}

/// A single physical publication borrows both original admitted requests.
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct HistoryProducerCutover<'call> {
    invocation: HistoryProducerInvocation<'call>,
    producer_request: &'call AdmittedRequestCurrentness,
}

impl HistoryProducerCutover<'_> {
    pub(in crate::domain_computation) fn current(&self) -> &ProductBranchObservation {
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

impl<'invocation> HistoryProducerInvocation<'invocation> {
    /// Diagnostic subject from an installed typed demand; authority is unchanged.
    pub(in crate::domain_computation::primary_graph) fn for_family(
        self,
        family: &'static str,
    ) -> Self {
        Self { family, ..self }
    }

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
        principal: &'invocation crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        selection: &'invocation HistoryAuthorityLifetime,
        callback: &'invocation HistoryAuthorityLifetime,
        selected_graph: Option<
            &'invocation worth_relational::facade::mvcc::RelationalBranchObservation,
        >,
        restored: &'invocation ProductBranchObservation,
        family: &'static str,
    ) -> Self {
        Self {
            current,
            request,
            principal,
            selection,
            callback,
            selected_graph,
            restored,
            family,
        }
    }

    pub(in crate::domain_computation) fn current(&self) -> &WorthQueryProductObservationLease {
        self.current
    }

    pub(in crate::domain_computation::primary_graph) fn request_lapse(
        &self,
    ) -> Option<crate::domain_computation::authorization::WorthQueryAdmissionLapse> {
        self.request.lapse()
    }

    pub(in crate::domain_computation) fn admits_principal(
        &self,
        principal: &crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    ) -> bool {
        self.principal == principal
    }

    pub(in crate::domain_computation) fn admits_request(
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
