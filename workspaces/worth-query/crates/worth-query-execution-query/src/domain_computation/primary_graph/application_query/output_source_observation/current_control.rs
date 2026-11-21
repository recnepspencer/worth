//! Current authorization and retry ownership for a producer's nested source reads.
use crate::domain_computation::primary_graph::application_contribution::producer::demand::denial::history_source::OutputDemandDenialHistoryExt;
use super::*;
use crate::domain_computation::primary_graph::{
    product_operation::document_history::{HistoryBorrowedReadStop, HistoryProducerInvocation},
    provider::WorthQueryApplicationBranchCommitLane,
    WorthQueryHistoryNavigationDenial, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind,
};

pub mod ordinary;
pub struct HistorySourceQueryControl<'a, S> {
    pub current: &'a SharedSelectedProductOperation<'a, S>,
    pub invocation: HistoryProducerInvocation<'a>,
    lane: Arc<WorthQueryApplicationBranchCommitLane>,
}

impl<'a, S: ApplicationSchema> SourceQueryReaderAccess<'a, S> {
    pub fn ordinary(
        application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
        shared: &'a SharedSelectedProductOperation<'a, S>,
        principal: &'a WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &'a WorthQueryRequestScope,
    ) -> Self {
        Self {
            application,
            shared,
            principal,
            request,
            history: None,
            current: None,
            predecessor: None,
            original: None,
            observation_refused: Default::default(),
            applied_participants: None,
            applied_source_publication: None,
            read_authorization: None,
            refused: Default::default(),
        }
    }
    pub fn record_control_refusal(
        &self,
        stop: SourceQueryObservationStop,
    ) -> SourceQueryObservationStop {
        if (self.history.is_some() || self.current.is_some()) && stop.is_control_refusal() {
            return self
                .refused
                .borrow_mut()
                .get_or_insert_with(|| stop.clone())
                .clone();
        }
        stop
    }
    pub fn new(
        application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
        shared: &'a SharedSelectedProductOperation<'a, S>,
        current: Option<&'a SharedSelectedProductOperation<'a, S>>,
        invocation: Option<HistoryProducerInvocation<'a>>,
        principal: &'a WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &'a WorthQueryRequestScope,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        let history = match (current, invocation) {
            (None, None) => None,
            (Some(current), Some(invocation))
                if std::ptr::eq(application, current.selected().application())
                    && current.selected().product().observation()
                        == invocation.current().observation()
                    && invocation.admits_request(request) =>
            {
                // Capture before any Native borrow: registry entry can allocate or wait.
                let lane = application
                    .runtime
                    .primary_provider
                    .application_branch_commit_lane_for_occurrence(
                        invocation.current().observation().lifecycle_incarnation(),
                    );
                Some(HistorySourceQueryControl {
                    current,
                    invocation,
                    lane,
                })
            }
            _ => {
                return Err(WorthQueryOutputDemandDenial::history_source(
                    WorthQueryHistoryNavigationDenial::ForeignDocument,
                ))
            }
        };
        let mut reader = Self::ordinary(application, shared, principal, request);
        reader.history = history;
        Ok(reader)
    }

    /// A handler cannot swallow a control refusal or turn it into domain denial.
    pub fn take_control_failure(&self) -> Option<WorthQueryOutputDemandDenial> {
        if let Some(cause) = self.observation_refused.borrow_mut().take() {
            if let super::predecessor::Denial::Control(stop) = cause {
                return Some(crate::domain_computation::primary_graph::invariant_projection::WorthQueryOutputAvailabilityDenial::source_query_stop(*stop).demand_stop());
            }
            let subject = format!("producer observation read: {cause}");
            let kind = match &cause {
                super::predecessor::Denial::Authorization(cause) => {
                    WorthQueryOutputDemandDenialKind::of_request_authorization(cause.kind())
                }
                _ => WorthQueryOutputDemandDenialKind::ObservationRead(Box::new(cause)),
            };
            return Some(WorthQueryOutputDemandDenial::new(kind, subject));
        }
        self.refused.borrow_mut().take().map(|stop| {
            crate::domain_computation::primary_graph::invariant_projection::WorthQueryOutputAvailabilityDenial::source_query_stop(stop).demand_stop()
        })
    }

    pub fn confirm_control<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &mut ApplicationQueryNativeAccess<'_>,
        value: T,
    ) -> Result<T, SourceQueryObservationStop> {
        let Some(history) = &self.history else {
            return self.confirm_current_control(execution, native, value);
        };
        let result = match native {
            ApplicationQueryNativeAccess::Acquire => history
                .invocation
                .confirm_in_execution(execution, self.application, value)
                .map_err(HistoryBorrowedReadStop::Navigation),
            ApplicationQueryNativeAccess::Borrowed(native) => history
                .invocation
                .confirm_in_borrowed(execution, self.application, &history.lane, native, value),
        };
        result.map_err(|stop| {
            SourceQueryObservationStop::CurrentControl(match stop {
                HistoryBorrowedReadStop::Busy => WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::CurrentControlBusy,
                    "history source query control",
                ),
                HistoryBorrowedReadStop::Navigation(stop) => {
                    WorthQueryOutputDemandDenial::history_source(stop)
                }
            })
        })
    }
}
