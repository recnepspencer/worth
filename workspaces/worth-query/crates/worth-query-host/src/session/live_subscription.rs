//! Typed Host handle over certified whole-result live query subscriptions.
//!
//! Any authority, overflow, limit, cancellation, or transport refusal
//! invalidates the cached snapshot and publication state immediately so stale
//! or pre-revocation rows are never exposed as current.
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
    WorthQueryApplicationLiveNextDenial, WorthQueryApplicationLiveOpenRequestDenial,
    WorthQueryApplicationRequest, WorthQueryApplicationWholeResultLiveSubscription,
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

/// Typed live whole-result subscription holding the latest current snapshot.
///
/// Open via [`WorthQueryHostSession::subscribe`]. Inspect the current projected
/// rows with [`Self::current`] (returns `None` once invalidated by any refusal),
/// poll for invalidated commits with [`Self::refresh`] (which enforces the
/// session's policy and re-admits principal and grants fresh every time), and
/// close cleanly with [`Self::close`].
#[must_use = "close the live subscription before closing the Home"]
pub struct WorthQueryTypedLiveSubscription<'home, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
    inner: WorthQueryApplicationWholeResultLiveSubscription<'home, Schema, Intent>,
    current: Option<
        WorthQueryApplicationWholeResultLiveSnapshot<
            QueryMarker<Schema, Intent>,
            QueryRow<Schema, Intent>,
        >,
    >,
    last_publication: Option<WorthQueryCommittedProductPublication>,
}

impl<'home, Schema, Intent> WorthQueryTypedLiveSubscription<'home, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
    pub(super) fn from_opened(
        mut inner: WorthQueryApplicationWholeResultLiveSubscription<'home, Schema, Intent>,
    ) -> Self {
        let current = inner
            .take_initial()
            .expect("opened whole-result live subscription always captures initial snapshot");
        Self {
            inner,
            current: Some(current),
            last_publication: None,
        }
    }

    fn invalidate(&mut self) {
        self.current = None;
        self.last_publication = None;
    }

    /// Returns `true` while this subscription holds a valid current snapshot.
    pub const fn is_current(&self) -> bool {
        self.current.is_some()
    }

    /// The current complete projected rows for this subscription, or `None`
    /// after an authority, overflow, limit, cancellation, or transport refusal
    /// invalidated currentness.
    pub fn current(&self) -> Option<&[QueryRow<Schema, Intent>]> {
        self.current.as_ref().map(|snapshot| snapshot.rows())
    }

    /// Borrows the complete latest snapshot while current, or `None` once
    /// invalidated.
    pub const fn current_snapshot(
        &self,
    ) -> Option<
        &WorthQueryApplicationWholeResultLiveSnapshot<
            QueryMarker<Schema, Intent>,
            QueryRow<Schema, Intent>,
        >,
    > {
        self.current.as_ref()
    }

    /// The committed product publication that produced the latest refresh, if
    /// current and a refresh has been delivered since initial open.
    pub const fn last_publication(&self) -> Option<&WorthQueryCommittedProductPublication> {
        self.last_publication.as_ref()
    }

    /// Polls for updates under `session`'s configured execution policy, fresh
    /// request scope, and fresh principal/grant admission.
    ///
    /// Returns `Ok(None)` when no unseen commit invalidated the observed
    /// footprint (`Pending`), `Ok(Some(rows))` when a new snapshot was
    /// delivered, or `Err(...)` while invalidating [`Self::current`] and
    /// [`Self::current_snapshot`] so stale or pre-revocation rows cannot leak.
    pub fn refresh<Program>(
        &mut self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<Option<&[QueryRow<Schema, Intent>]>, WorthQueryTypedLiveRefreshDenial> {
        let scope = session.fresh_scope();
        let fresh_request = session.request(&scope);
        let outcome = match session.with_funded_advancement(&scope, |phase| {
            self.inner.next_in_advancement(&phase, &fresh_request)
        }) {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(next_denial)) => {
                self.invalidate();
                return Err(WorthQueryTypedLiveRefreshDenial::Next(next_denial));
            }
            Err(advancement_denial) => {
                self.invalidate();
                return Err(WorthQueryTypedLiveRefreshDenial::Next(
                    WorthQueryApplicationLiveNextDenial::ExecutionRequest(advancement_denial),
                ));
            }
        };
        self.apply_outcome(outcome)
    }

    /// Polls for updates using an explicit fresh [`WorthQueryApplicationRequest`].
    pub fn refresh_with_request(
        &mut self,
        fresh_request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<Option<&[QueryRow<Schema, Intent>]>, WorthQueryTypedLiveRefreshDenial> {
        let outcome = match self.inner.next(fresh_request) {
            Ok(outcome) => outcome,
            Err(next_denial) => {
                self.invalidate();
                return Err(WorthQueryTypedLiveRefreshDenial::Next(next_denial));
            }
        };
        self.apply_outcome(outcome)
    }

    fn apply_outcome(
        &mut self,
        outcome: WorthQueryApplicationWholeResultLiveOutcome<
            QueryMarker<Schema, Intent>,
            QueryRow<Schema, Intent>,
        >,
    ) -> Result<Option<&[QueryRow<Schema, Intent>]>, WorthQueryTypedLiveRefreshDenial> {
        use WorthQueryApplicationWholeResultLiveOutcome as Outcome;
        use WorthQueryTypedLiveRefreshDenial as Denial;
        match outcome {
            Outcome::Pending => Ok(None),
            Outcome::Delivered(update) => {
                let (publication, snapshot) = update.into_parts();
                self.last_publication = Some(publication);
                self.current = Some(snapshot);
                Ok(self.current.as_ref().map(|s| s.rows()))
            }
            other => {
                self.invalidate();
                Err(match other {
                    Outcome::Overflow(overflow) => Denial::Overflow(overflow),
                    Outcome::AuthorizationDenied(denial) => Denial::AuthorizationDenied(denial),
                    Outcome::StalePrincipal => Denial::StalePrincipal,
                    Outcome::StaleScope => Denial::StaleScope,
                    Outcome::ProjectionDenied(kind) => Denial::ProjectionDenied(kind),
                    Outcome::ResultLimitExceeded => Denial::ResultLimitExceeded,
                    Outcome::ExecutionRequest(kind) => Denial::ExecutionRequest(kind),
                    Outcome::Cancelled => Denial::Cancelled,
                    Outcome::DeadlineExceeded => Denial::DeadlineExceeded,
                    Outcome::Closed => Denial::Closed,
                    Outcome::Unavailable => Denial::Unavailable,
                    Outcome::Pending | Outcome::Delivered(_) => unreachable!(),
                })
            }
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
        self.open_whole_result(intent)
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
        self.with_funded_advancement(&scope, |phase| {
            query_req.subscribe_whole_result_governed_in_advancement(
                &phase, capability, operation, input, limits,
            )
        })
        .map_err(WorthQueryApplicationLiveOpenRequestDenial::ExecutionRequest)?
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
        self.with_funded_advancement(&scope, |phase| {
            query_req.subscribe_whole_result_approved_in_advancement(
                &phase, approved, capability, operation, input, limits,
            )
        })
        .map_err(WorthQueryApplicationLiveOpenRequestDenial::ExecutionRequest)?
        .map(WorthQueryTypedLiveSubscription::from_opened)
    }
}
