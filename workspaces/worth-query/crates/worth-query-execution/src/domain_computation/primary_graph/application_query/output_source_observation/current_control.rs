//! Current authorization and retry ownership for a producer's nested source reads.
use super::*;
use {
    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial,
    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind,
    crate::domain_computation::primary_graph::product_operation::document_history::HistoryBorrowedReadStop,
    crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation,
    crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitLane,
    crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial,
};
use crate::domain_computation::primary_graph::vocabulary::application_query::one_shot::WorthQueryAdmittedOneShotStop;
use crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenialKind;
use crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind;

pub(super) struct HistorySourceQueryControl<'a, S> {
    pub(super) current: &'a SharedSelectedProductOperation<'a, S>,
    pub(super) invocation: HistoryProducerInvocation<'a>,
    lane: Arc<WorthQueryApplicationBranchCommitLane>,
}

impl SourceQueryObservationStop {
    /// Authorized model-result outcomes retain the handler's existing semantics.
    pub(super) fn is_control_refusal(&self) -> bool {
        use crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenialKind as Read;
        use crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind as Admission;
        use SourceQueryObservationStop as Stop;
        match self {
            Stop::CurrentControl(_)
            | Stop::ManagedIndex(_)
            | Stop::Principal(_)
            | Stop::Scope(_)
            | Stop::Basis(_) => true,
            Stop::Admission(stop) | Stop::Preparation { cause: stop, .. } => matches!(
                stop.kind(),
                Admission::Authorization(_)
                    | Admission::StalePrincipal
                    | Admission::StaleScope
                    | Admission::ForeignPrincipal
                    | Admission::ForeignScope
                    | Admission::StaleBasis
                    | Admission::Cancelled
                    | Admission::DeadlineExceeded
                    | Admission::ExecutionRequest(_)
                    | Admission::ActiveSnapshotCapacityExhausted { .. }
                    | Admission::SnapshotIdentityExhausted
                    | Admission::RetentionCapacityExhausted
                    | Admission::RetentionIdentityExhausted
                    | Admission::ReadmissionPreparationMemoryExhausted
                    | Admission::WorkLimitExceeded
                    | Admission::CanonicalWorkDenied
                    | Admission::GraphWorkAdmissionUnavailable
                    | Admission::DisclosureGovernanceRequired
                    | Admission::DisclosureAuthorizationMismatch
                    | Admission::InternalComputationDenied
            ),
            Stop::Read(WorthQueryAdmittedOneShotStop::Execution(stop)) => matches!(
                stop.kind(),
                Read::Authorization(_)
                    | Read::StalePrincipal
                    | Read::StaleScope
                    | Read::Cancelled
                    | Read::DeadlineExceeded
                    | Read::ExecutionRequest(_)
                    | Read::ActiveSnapshotCapacityExhausted { .. }
                    | Read::SnapshotIdentityExhausted
                    | Read::RetentionCapacityExhausted
                    | Read::RetentionIdentityExhausted
                    | Read::WorkLimitExceeded
                    | Read::ResultBufferLimitExceeded
                    | Read::SourceIdentityExhausted
                    | Read::SourceObservationAllocation { .. }
            ),
            Stop::Read(_) => true,
            Stop::SourceFacts(stop) => {
                stop.execution_denial().is_some() || stop.retention_interruption().is_some()
            }
            Stop::Missing | Stop::Ambiguous | Stop::ForeignClosure | Stop::Installation(_) => false,
        }
    }
}

impl<'a, S: ApplicationSchema> SourceQueryReaderAccess<'a, S> {
    pub(in crate::domain_computation::primary_graph) fn ordinary(
        application: &'a dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<S>,
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
            refused: Default::default(),
        }
    }
    pub(super) fn record_history_refusal(
        &self,
        stop: SourceQueryObservationStop,
    ) -> SourceQueryObservationStop {
        if self.history.is_some() && stop.is_control_refusal() {
            return self
                .refused
                .borrow_mut()
                .get_or_insert_with(|| stop.clone())
                .clone();
        }
        stop
    }
    pub(in crate::domain_computation::primary_graph) fn new(
        application: &'a dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<S>,
        shared: &'a SharedSelectedProductOperation<'a, S>,
        current: Option<&'a SharedSelectedProductOperation<'a, S>>,
        invocation: Option<HistoryProducerInvocation<'a>>,
        principal: &'a WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &'a WorthQueryRequestScope,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        let history = match (current, invocation) {
            (None, None) => None,
            (Some(current), Some(invocation))
                if std::ptr::addr_eq(application, current.selected().application())
                    && current.selected().product().observation()
                        == invocation.current().observation()
                    && invocation.admits_request(request) =>
            {
                // Capture before any Native borrow: registry entry can allocate or wait.
                let lane = application
                    .primary_provider()
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
    pub(in crate::domain_computation::primary_graph) fn take_history_failure(
        &self,
    ) -> Option<WorthQueryOutputDemandDenial> {
        self.refused.borrow_mut().take().map(|stop| {
            crate::domain_computation::primary_graph::invariant_projection::WorthQueryOutputAvailabilityDenial::source_query_stop(stop).demand_stop()
        })
    }

    pub(super) fn confirm_control<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &mut ApplicationQueryNativeAccess<'_>,
        value: T,
    ) -> Result<T, SourceQueryObservationStop> {
        let Some(history) = &self.history else {
            return Ok(value);
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

mod court_doors_for_test;
