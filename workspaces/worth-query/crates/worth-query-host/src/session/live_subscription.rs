//! Typed Host handle over certified whole-result live query subscriptions.
//!
//! Any authority, overflow, limit, cancellation, or transport refusal
//! invalidates the cached snapshot and publication state immediately so stale
//! or pre-revocation rows are never exposed as current. An expired session
//! proof refuses one poll without ending the subscription: the snapshot is
//! withheld until a renewed session's poll confirms or replaces it.
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationSchema,
};
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationLiveCloseOutcome, WorthQueryApplicationLiveOverflow,
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenialKind,
    WorthQueryApplicationWholeResultLiveOutcome, WorthQueryApplicationWholeResultLiveSnapshot,
    WorthQueryCommittedProductPublication, WorthQueryOperationAuthorizationDenial,
    WorthQueryPrincipalResolutionDenialKind as Principal,
    WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal,
    WorthQueryProviderSessionDenialKind,
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationLiveNextDenial, WorthQueryApplicationLiveOpenRequestDenial,
    WorthQueryApplicationRequest, WorthQueryApplicationWholeResultLiveSubscription as Native,
};

use super::live_slot::LiveSlot;
use super::{QueryBinding, QueryMarker, QueryRow, WorthQueryHostSession};
use crate::application_home::owned_holds::OwnedRuntimeHold;

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
    /// The refreshing session's proof expired. The subscription stays open;
    /// refreshing with a renewed (or self-renewing) session resumes it.
    SessionExpired,
}

/// Refusal when closing a [`WorthQueryTypedLiveSubscription`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryTypedLiveCloseDenial {
    Unavailable,
}

type Snapshot<Schema, Intent> = WorthQueryApplicationWholeResultLiveSnapshot<
    QueryMarker<Schema, Intent>,
    QueryRow<Schema, Intent>,
>;
type Outcome<Schema, Intent> = WorthQueryApplicationWholeResultLiveOutcome<
    QueryMarker<Schema, Intent>,
    QueryRow<Schema, Intent>,
>;
type Polled<Schema, Intent> = Result<
    Result<Outcome<Schema, Intent>, WorthQueryApplicationLiveNextDenial>,
    WorthQueryAdvancementDenial,
>;

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
    inner: LiveSlot<'home, Schema, Intent>,
    current: Option<Snapshot<Schema, Intent>>,
    /// The snapshot an expired-session poll withheld, pending confirmation.
    withheld: Option<Snapshot<Schema, Intent>>,
    /// A borrowed subscription's latest publication; an owned one keeps it in
    /// its slot so closing the Home can drop the receipt.
    last_publication: Option<WorthQueryCommittedProductPublication>,
}

impl<'home, Schema, Intent> WorthQueryTypedLiveSubscription<'home, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
    pub(super) fn from_opened(mut inner: Native<'home, Schema, Intent>) -> Self {
        let current = inner
            .take_initial()
            .expect("opened whole-result live subscription always captures initial snapshot");
        Self::from_slot(LiveSlot::Direct(inner), current)
    }

    pub(super) fn from_slot(
        inner: LiveSlot<'home, Schema, Intent>,
        current: Snapshot<Schema, Intent>,
    ) -> Self {
        Self {
            inner,
            current: Some(current),
            withheld: None,
            last_publication: None,
        }
    }

    fn invalidate(&mut self) {
        self.current = None;
        self.withheld = None;
        self.set_publication(None);
    }

    fn set_publication(&mut self, publication: Option<WorthQueryCommittedProductPublication>) {
        match &self.inner {
            LiveSlot::Direct(_) => self.last_publication = publication,
            LiveSlot::Held(held) => *held.publication() = publication,
        }
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
    pub const fn current_snapshot(&self) -> Option<&Snapshot<Schema, Intent>> {
        self.current.as_ref()
    }

    /// The committed product publication that produced the latest refresh, if
    /// current and a refresh has been delivered since initial open. A held
    /// clone keeps the Home busy until dropped.
    pub fn last_publication(&self) -> Option<WorthQueryCommittedProductPublication> {
        match &self.inner {
            LiveSlot::Direct(_) => self.last_publication.clone(),
            LiveSlot::Held(held) => held.publication().clone(),
        }
    }

    /// Polls for updates under `session`'s configured execution policy, fresh
    /// request scope, and fresh principal/grant admission.
    ///
    /// Returns `Ok(None)` when no unseen commit invalidated the observed
    /// footprint (`Pending`), `Ok(Some(rows))` when a new snapshot was
    /// delivered or a withheld snapshot was confirmed, or `Err(...)` while
    /// invalidating [`Self::current`] and [`Self::current_snapshot`] so stale
    /// or pre-revocation rows cannot leak. `Err(SessionExpired)` keeps the
    /// subscription open for a renewed session's next refresh.
    pub fn refresh<Program>(
        &mut self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<Option<&[QueryRow<Schema, Intent>]>, WorthQueryTypedLiveRefreshDenial> {
        let scope = session.fresh_scope();
        let principal = session.call_principal();
        let fresh_request = session.request_as(&principal, &scope);
        let poll = |native: &mut Native<'_, Schema, Intent>| -> Polled<Schema, Intent> {
            session.with_funded_advancement(&scope, |phase| {
                native.next_in_advancement(&phase, &fresh_request)
            })
        };
        let polled = match &mut self.inner {
            LiveSlot::Direct(native) => Some(poll(native)),
            LiveSlot::Held(held) => held.native().as_mut().map(poll),
        };
        let Some(polled) = polled else {
            return self.released();
        };
        match polled {
            Ok(Ok(outcome)) => self.apply_outcome(outcome),
            Ok(Err(denial)) => self.refuse_next(denial),
            Err(advancement) => self.refuse_next(
                WorthQueryApplicationLiveNextDenial::ExecutionRequest(advancement),
            ),
        }
    }

    /// Polls for updates using an explicit fresh [`WorthQueryApplicationRequest`].
    pub fn refresh_with_request(
        &mut self,
        fresh_request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<Option<&[QueryRow<Schema, Intent>]>, WorthQueryTypedLiveRefreshDenial> {
        let polled = match &mut self.inner {
            LiveSlot::Direct(native) => Some(native.next(fresh_request)),
            LiveSlot::Held(held) => held
                .native()
                .as_mut()
                .map(|native| native.next(fresh_request)),
        };
        let Some(polled) = polled else {
            return self.released();
        };
        match polled {
            Ok(outcome) => self.apply_outcome(outcome),
            Err(denial) => self.refuse_next(denial),
        }
    }

    /// The Home closed (or its owned session detached): close any native lease
    /// and receipt still held, so a retirement on another thread can finish.
    pub(super) fn released(
        &mut self,
    ) -> Result<Option<&[QueryRow<Schema, Intent>]>, WorthQueryTypedLiveRefreshDenial> {
        if let LiveSlot::Held(held) = &self.inner {
            OwnedRuntimeHold::release(held.as_ref());
        }
        self.invalidate();
        Err(WorthQueryTypedLiveRefreshDenial::Closed)
    }

    /// An expired proof withholds the snapshot and keeps the subscription;
    /// every other poll refusal invalidates it.
    fn refuse_next(
        &mut self,
        denial: WorthQueryApplicationLiveNextDenial,
    ) -> Result<Option<&[QueryRow<Schema, Intent>]>, WorthQueryTypedLiveRefreshDenial> {
        if let WorthQueryApplicationLiveNextDenial::PrincipalResolution(resolution) = &denial
            && resolution.kind() == Principal::Refused(PrincipalRefusal::ExpiredAuthentication)
        {
            self.withheld = self.current.take().or(self.withheld.take());
            self.set_publication(None);
            return Err(WorthQueryTypedLiveRefreshDenial::SessionExpired);
        }
        self.invalidate();
        Err(WorthQueryTypedLiveRefreshDenial::Next(denial))
    }

    fn apply_outcome(
        &mut self,
        outcome: Outcome<Schema, Intent>,
    ) -> Result<Option<&[QueryRow<Schema, Intent>]>, WorthQueryTypedLiveRefreshDenial> {
        use WorthQueryApplicationWholeResultLiveOutcome as Outcome;
        use WorthQueryTypedLiveRefreshDenial as Denial;
        match outcome {
            // A freshly admitted poll that found no unseen commit confirms a
            // snapshot an expired-session poll withheld.
            Outcome::Pending => match self.withheld.take() {
                Some(withheld) => {
                    self.current = Some(withheld);
                    Ok(self.current.as_ref().map(|s| s.rows()))
                }
                None => Ok(None),
            },
            Outcome::Delivered(update) => {
                let (publication, snapshot) = update.into_parts();
                self.set_publication(Some(publication));
                self.current = Some(snapshot);
                self.withheld = None;
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
    /// terminated or unavailable. An owned subscription its Home already
    /// released is closed.
    pub fn close(self) -> Result<(), WorthQueryTypedLiveCloseDenial> {
        match self.close_native() {
            Some(WorthQueryApplicationLiveCloseOutcome::Unavailable) => {
                Err(WorthQueryTypedLiveCloseDenial::Unavailable)
            }
            Some(WorthQueryApplicationLiveCloseOutcome::Completed(_)) | None => Ok(()),
        }
    }

    /// Closes this live subscription and returns the raw native close outcome
    /// (`Unavailable` for an owned subscription its Home already released).
    pub fn close_with_outcome(self) -> WorthQueryApplicationLiveCloseOutcome {
        self.close_native()
            .unwrap_or(WorthQueryApplicationLiveCloseOutcome::Unavailable)
    }

    fn close_native(self) -> Option<WorthQueryApplicationLiveCloseOutcome> {
        match self.inner {
            LiveSlot::Direct(native) => Some(native.close()),
            LiveSlot::Held(held) => held.native().take().map(Native::close),
        }
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
}
