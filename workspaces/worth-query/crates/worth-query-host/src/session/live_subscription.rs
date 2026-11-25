//! Typed Host handle over certified whole-result live query subscriptions.
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationSchema,
        ApplicationStructuredValueBinding,
    },
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationLiveCloseOutcome, WorthQueryApplicationLiveOverflow,
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenialKind,
    WorthQueryApplicationWholeResultLiveOutcome, WorthQueryApplicationWholeResultLiveSnapshot,
    WorthQueryApprovedElevation, WorthQueryCommittedProductPublication,
    WorthQueryOperationAuthorizationDenial, WorthQueryProviderSessionDenialKind,
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationLiveLimits, WorthQueryApplicationLiveNextDenial,
    WorthQueryApplicationLiveOpenRequestDenial, WorthQueryApplicationRequest,
    WorthQueryApplicationWholeResultLiveSubscription,
};

use super::{QueryBinding, QueryMarker, QueryRow, QueryScope, WorthQueryHostSession};

/// Every non-`Pending`/non-`Delivered` failure or termination returned by
/// [`WorthQueryTypedLiveSubscription::refresh`].
#[derive(Debug)]
pub enum WorthQueryTypedLiveRefreshDenial {
    Next(WorthQueryApplicationLiveNextDenial),
    Overflow(WorthQueryApplicationLiveOverflow),
    AuthorizationDenied(Box<WorthQueryOperationAuthorizationDenial>),
    StalePrincipal,
    StaleScope,
    ProjectionDenied(WorthQueryApplicationProjectionDenialKind),
    ResultLimitExceeded,
    ExecutionRequest(WorthQueryProviderSessionDenialKind),
    Cancelled,
    DeadlineExceeded,
    Closed,
    Unavailable,
}

/// Refusal when closing a [`WorthQueryTypedLiveSubscription`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryTypedLiveCloseDenial {
    Unavailable,
}

/// Typed live whole-result subscription holding the latest complete snapshot.
///
/// Open via [`WorthQueryHostSession::subscribe`]. Inspect the latest projected
/// rows with [`Self::current`], poll for invalidated commits with
/// [`Self::refresh`] (which re-admits principal and grants fresh every time),
/// and close cleanly with [`Self::close`].
#[must_use = "close the live subscription before closing the Home"]
pub struct WorthQueryTypedLiveSubscription<'home, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
    inner: WorthQueryApplicationWholeResultLiveSubscription<'home, Schema, Intent>,
    current: WorthQueryApplicationWholeResultLiveSnapshot<
        QueryMarker<Schema, Intent>,
        QueryRow<Schema, Intent>,
    >,
    last_publication: Option<WorthQueryCommittedProductPublication>,
}

impl<'home, Schema, Intent> WorthQueryTypedLiveSubscription<'home, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
    fn from_opened(
        mut inner: WorthQueryApplicationWholeResultLiveSubscription<'home, Schema, Intent>,
    ) -> Self {
        let current = inner
            .take_initial()
            .expect("opened whole-result live subscription always captures initial snapshot");
        Self {
            inner,
            current,
            last_publication: None,
        }
    }

    /// The current complete projected rows for this subscription.
    pub fn current(&self) -> &[QueryRow<Schema, Intent>] {
        self.current.rows()
    }

    /// Borrows the complete latest snapshot (rows, observed sources, receipt).
    pub const fn current_snapshot(
        &self,
    ) -> &WorthQueryApplicationWholeResultLiveSnapshot<
        QueryMarker<Schema, Intent>,
        QueryRow<Schema, Intent>,
    > {
        &self.current
    }

    /// The committed product publication that produced the latest refresh, if
    /// any refresh has been delivered since initial open.
    pub const fn last_publication(&self) -> Option<&WorthQueryCommittedProductPublication> {
        self.last_publication.as_ref()
    }

    /// Polls for updates using a fresh request scope and fresh principal/grant
    /// admission from `session`.
    ///
    /// Returns `Ok(None)` when no unseen commit invalidated the observed
    /// footprint (`Pending`), `Ok(Some(rows))` when a new snapshot was
    /// delivered (updating [`Self::current`]), or `Err(...)` for every
    /// authorization, overflow, limit, cancellation, or transport failure.
    pub fn refresh<Program>(
        &mut self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<Option<&[QueryRow<Schema, Intent>]>, WorthQueryTypedLiveRefreshDenial> {
        let scope = session.fresh_scope();
        let fresh_request = session.request(&scope);
        self.refresh_with_request(&fresh_request)
    }

    /// Polls for updates using an explicit fresh [`WorthQueryApplicationRequest`].
    pub fn refresh_with_request(
        &mut self,
        fresh_request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<Option<&[QueryRow<Schema, Intent>]>, WorthQueryTypedLiveRefreshDenial> {
        use WorthQueryApplicationWholeResultLiveOutcome as Outcome;
        use WorthQueryTypedLiveRefreshDenial as Denial;
        match self.inner.next(fresh_request).map_err(Denial::Next)? {
            Outcome::Pending => Ok(None),
            Outcome::Delivered(update) => {
                let (publication, snapshot) = update.into_parts();
                self.last_publication = Some(publication);
                self.current = snapshot;
                Ok(Some(self.current.rows()))
            }
            Outcome::Overflow(overflow) => Err(Denial::Overflow(overflow)),
            Outcome::AuthorizationDenied(denial) => Err(Denial::AuthorizationDenied(denial)),
            Outcome::StalePrincipal => Err(Denial::StalePrincipal),
            Outcome::StaleScope => Err(Denial::StaleScope),
            Outcome::ProjectionDenied(kind) => Err(Denial::ProjectionDenied(kind)),
            Outcome::ResultLimitExceeded => Err(Denial::ResultLimitExceeded),
            Outcome::ExecutionRequest(kind) => Err(Denial::ExecutionRequest(kind)),
            Outcome::Cancelled => Err(Denial::Cancelled),
            Outcome::DeadlineExceeded => Err(Denial::DeadlineExceeded),
            Outcome::Closed => Err(Denial::Closed),
            Outcome::Unavailable => Err(Denial::Unavailable),
        }
    }

    /// Closes this live subscription, refusing honestly if it was already
    /// terminated or unavailable.
    pub fn close(self) -> Result<(), WorthQueryTypedLiveCloseDenial> {
        match self.inner.close() {
            WorthQueryApplicationLiveCloseOutcome::Completed(_completion) => Ok(()),
            WorthQueryApplicationLiveCloseOutcome::Unavailable => {
                Err(WorthQueryTypedLiveCloseDenial::Unavailable)
            }
        }
    }

    /// Closes this live subscription and returns the raw native close outcome.
    pub fn close_with_outcome(self) -> WorthQueryApplicationLiveCloseOutcome {
        self.inner.close()
    }
}

impl<'home, Schema, Program> WorthQueryHostSession<'home, Schema, Program>
where
    Schema: ApplicationSchema,
{
    fn resolved_live_limits<Intent>(&self) -> WorthQueryApplicationLiveLimits
    where
        Intent: ApplicationQueryIntent<Schema>,
    {
        let maximum_results = match self.live_maximum_results {
            Some(explicit) => explicit,
            None => {
                let installed_max = self
                    .home
                    .primary_graph()
                    .installed_schema()
                    .installed_query_binding::<QueryBinding<Schema, Intent>>()
                    .map(|binding| {
                        self.home
                            .primary_graph()
                            .resolve_application_query_limits(binding.limits())
                            .maximum_results()
                            .get()
                    })
                    .unwrap_or(super::DEFAULT_LIVE_MAXIMUM_RESULTS);
                match self.query_result_limit {
                    Some(narrowed) => narrowed.get().min(installed_max),
                    None => installed_max,
                }
            }
        };
        WorthQueryApplicationLiveLimits::bounded(self.live_buffer_capacity, maximum_results)
    }

    /// Opens a typed whole-result live subscription for a query declared with
    /// `#[worth_query(..., live = whole_result)]`.
    pub fn subscribe<Intent>(
        &self,
        intent: Intent,
    ) -> Result<
        WorthQueryTypedLiveSubscription<'home, Schema, Intent>,
        WorthQueryApplicationLiveOpenRequestDenial,
    >
    where
        Intent: ApplicationQueryIntent<Schema>,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
        let scope = self.fresh_scope();
        let request = self.request(&scope);
        let query_req = request.query(intent);
        let query_req = match self.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        let limits = self.resolved_live_limits::<Intent>();
        query_req
            .subscribe_whole_result(limits)
            .map(WorthQueryTypedLiveSubscription::from_opened)
    }

    /// Opens a capability-governed typed whole-result live subscription.
    pub fn subscribe_governed<Intent, Capability, Operation, Input>(
        &self,
        intent: Intent,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<
        WorthQueryTypedLiveSubscription<'home, Schema, Intent>,
        WorthQueryApplicationLiveOpenRequestDenial,
    >
    where
        Intent: ApplicationQueryIntent<Schema>,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = QueryScope<Schema, Intent>>
            + 'static,
{
        let scope = self.fresh_scope();
        let request = self.request(&scope);
        let query_req = request.query(intent);
        let query_req = match self.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        let limits = self.resolved_live_limits::<Intent>();
        query_req
            .subscribe_whole_result_governed(capability, operation, input, limits)
            .map(WorthQueryTypedLiveSubscription::from_opened)
    }

    /// Opens an approved-elevation typed whole-result live subscription.
    pub fn subscribe_approved<Intent, Capability, Operation, Input>(
        &self,
        intent: Intent,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<
        WorthQueryTypedLiveSubscription<'home, Schema, Intent>,
        WorthQueryApplicationLiveOpenRequestDenial,
    >
    where
        Intent: ApplicationQueryIntent<Schema>,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = QueryScope<Schema, Intent>>
            + 'static,
{
        let scope = self.fresh_scope();
        let request = self.request(&scope);
        let query_req = request.query(intent);
        let query_req = match self.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        let limits = self.resolved_live_limits::<Intent>();
        query_req
            .subscribe_whole_result_approved(approved, capability, operation, input, limits)
            .map(WorthQueryTypedLiveSubscription::from_opened)
    }
}
