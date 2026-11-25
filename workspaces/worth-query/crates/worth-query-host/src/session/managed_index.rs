//! Managed-index `session.watch(...)` and `session.ensure_index_ready(...)` helpers,
//! preserving session execution policies and result ceilings across open and refresh.
use std::num::NonZeroUsize;

use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_query::{
        ApplicationManagedIndexQueryIntent, ApplicationQueryBinding,
        ApplicationQueryScopeResolution,
    },
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationSchema,
        ApplicationStructuredValueBinding,
    },
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApprovedElevation,
    WorthQueryCommittedProductPublication, WorthQueryManagedIndex, WorthQueryManagedIndexDenial,
    WorthQueryManagedIndexWatch, WorthQueryManagedIndexWatchOutcome,
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationRequest, WorthQueryApplicationRequestQueryDenial,
};

use super::{QueryBinding, QueryMarker, QueryRow, QueryScope, WorthQueryHostSession};

/// Every non-`Pending`/non-`Delivered` refusal from
/// [`WorthQueryTypedManagedIndexWatch::refresh`].
#[derive(Debug)]
pub enum WorthQueryTypedManagedIndexRefreshDenial {
    Query(WorthQueryApplicationRequestQueryDenial),
    Overflow { missed_commit_batches: u64 },
    Unavailable,
    Closed,
}

/// Typed managed-index watch holding the latest projected rows, the configured
/// result ceiling, and the underlying [`WorthQueryManagedIndexWatch`].
#[must_use = "close the managed-index watch when finished"]
pub struct WorthQueryTypedManagedIndexWatch<Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationManagedIndexQueryIntent<Schema>,
    QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
    intent: Intent,
    watch: WorthQueryManagedIndexWatch<Schema, Intent::Index>,
    result_limit: Option<NonZeroUsize>,
    current: Option<Vec<QueryRow<Schema, Intent>>>,
    last_notification: Option<WorthQueryCommittedProductPublication>,
}

impl<Schema, Intent> WorthQueryTypedManagedIndexWatch<Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationManagedIndexQueryIntent<Schema> + Clone,
    <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
    fn invalidate(&mut self) {
        self.current = None;
        self.last_notification = None;
    }

    /// Returns `true` while this watch holds a current snapshot.
    pub const fn is_current(&self) -> bool {
        self.current.is_some()
    }

    /// The latest projected rows from the managed-index watch, or `None` after
    /// a refusal invalidated currentness.
    pub fn current(&self) -> Option<&[QueryRow<Schema, Intent>]> {
        self.current.as_deref()
    }

    /// The latest notification publication that triggered a refresh, if current.
    pub const fn last_notification(&self) -> Option<&WorthQueryCommittedProductPublication> {
        self.last_notification.as_ref()
    }

    /// Refreshes the watch under `session`'s configured execution policy,
    /// preserved result ceiling, fresh request scope, and fresh principal admission.
    pub fn refresh<Program>(
        &mut self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<Option<&[QueryRow<Schema, Intent>]>, WorthQueryTypedManagedIndexRefreshDenial> {
        use WorthQueryTypedManagedIndexRefreshDenial as Denial;
        let scope = session.fresh_scope();
        let lease = match session.open_lease(&scope) {
            Ok(lease) => lease,
            Err(advancement_denial) => {
                self.invalidate();
                return Err(Denial::Query(
                    WorthQueryApplicationRequestQueryDenial::ExecutionRequest(advancement_denial),
                ));
            }
        };
        let fresh_request = session.request(&scope);
        let query_req = fresh_request.query(self.intent.clone());
        let query_req = match self.result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        let outcome = match query_req.next_watch_with_allocation(
            &mut self.watch,
            worth_execution::ExecutionAllocationPolicy::Execution(&lease),
        ) {
            Ok(outcome) => outcome,
            Err(query_denial) => {
                self.invalidate();
                return Err(Denial::Query(query_denial));
            }
        };
        self.apply_outcome(outcome)
    }

    /// Refreshes the watch using an explicit fresh [`WorthQueryApplicationRequest`],
    /// preserving the result ceiling captured when the watch opened.
    pub fn refresh_with_request(
        &mut self,
        fresh_request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<Option<&[QueryRow<Schema, Intent>]>, WorthQueryTypedManagedIndexRefreshDenial> {
        use WorthQueryTypedManagedIndexRefreshDenial as Denial;
        let query_req = fresh_request.query(self.intent.clone());
        let query_req = match self.result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        let outcome = match query_req.next_watch(&mut self.watch) {
            Ok(outcome) => outcome,
            Err(query_denial) => {
                self.invalidate();
                return Err(Denial::Query(query_denial));
            }
        };
        self.apply_outcome(outcome)
    }

    fn apply_outcome(
        &mut self,
        outcome: WorthQueryManagedIndexWatchOutcome<
            QueryMarker<Schema, Intent>,
            QueryRow<Schema, Intent>,
        >,
    ) -> Result<Option<&[QueryRow<Schema, Intent>]>, WorthQueryTypedManagedIndexRefreshDenial> {
        use WorthQueryManagedIndexWatchOutcome as Outcome;
        use WorthQueryTypedManagedIndexRefreshDenial as Denial;
        match outcome {
            Outcome::Pending => Ok(None),
            Outcome::Delivered(update) => {
                self.last_notification = Some(update.notification().clone());
                self.current = Some(update.into_read().into_rows());
                Ok(self.current.as_deref())
            }
            Outcome::Overflow {
                missed_commit_batches,
            } => {
                self.invalidate();
                Err(Denial::Overflow {
                    missed_commit_batches,
                })
            }
            Outcome::Unavailable => {
                self.invalidate();
                Err(Denial::Unavailable)
            }
            Outcome::Closed => {
                self.invalidate();
                Err(Denial::Closed)
            }
        }
    }

    /// Closes the managed-index watch and releases its predicate registration.
    pub fn close(self) -> Result<(), WorthQueryManagedIndexDenial> {
        self.watch.close()
    }
}

impl<Schema, Program> WorthQueryHostSession<'_, Schema, Program>
where
    Schema: ApplicationSchema,
{
    /// Opens an inverse-routed live watch over a managed-index query declared
    /// with `#[worth_query(..., managed_index = Idx, lanes = OneShotAndLive)]`.
    pub fn watch<Intent>(
        &self,
        intent: Intent,
    ) -> Result<
        WorthQueryTypedManagedIndexWatch<Schema, Intent>,
        WorthQueryApplicationRequestQueryDenial,
    >
    where
        Intent: ApplicationManagedIndexQueryIntent<Schema> + Clone,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
        let scope = self.fresh_scope();
        let request = self.request(&scope);
        let query_req = request.query(intent.clone());
        let query_req = match self.query_result_limit {
            Some(limit) => query_req.limit_results(limit),
            None => query_req,
        };
        let (initial, watch) = self
            .with_funded_advancement(&scope, |phase| {
                query_req.watch_index_in_advancement::<Intent::Index>(&phase)
            })
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)??;
        Ok(WorthQueryTypedManagedIndexWatch {
            intent,
            watch,
            result_limit: self.query_result_limit,
            current: Some(initial.into_rows()),
            last_notification: None,
        })
    }

    /// Ensures the managed index backing `parent_intent` is `Ready`, rebuilding
    /// ONLY if its status is `Created` (such as after cold reopen).
    pub fn ensure_index_ready<
        Intent: super::query_intent::WorthQuerySessionIndexedQuery<Schema>,
    >(
        &self,
        parent_intent: Intent,
    ) -> Result<
        WorthQueryManagedIndex<Schema, Intent::Index>,
        WorthQueryApplicationRequestQueryDenial,
    > {
        parent_intent.ensure_ready_in_session(self)
    }

    /// Ensures the managed index backing a `Default`-constructible parent query
    /// intent is `Ready`, rebuilding ONLY if its status is `Created`.
    pub fn ensure_default_index_ready<Intent>(
        &self,
    ) -> Result<WorthQueryManagedIndex<Schema, Intent::Index>, WorthQueryApplicationRequestQueryDenial>
    where
        Intent: ApplicationManagedIndexQueryIntent<Schema> + Default,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
{
        self.ensure_index_ready(Intent::default())
    }

    /// Ensures a capability-governed managed index is `Ready` for `parent_intent`,
    /// rebuilding ONLY if its status is `Created`.
    pub fn ensure_index_ready_governed<Intent, Capability, Operation, Input>(
        &self,
        parent_intent: Intent,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<WorthQueryManagedIndex<Schema, Intent::Index>, WorthQueryApplicationRequestQueryDenial>
    where
        Intent: ApplicationManagedIndexQueryIntent<Schema>,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = QueryScope<Schema, Intent>>
            + 'static,
{
        let scope = self.fresh_scope();
        let request = self.request(&scope);
        self.with_funded_advancement(&scope, |phase| {
            request
                .query(parent_intent)
                .ensure_index_ready_governed_in_advancement::<Intent::Index, _, _, _>(
                    &phase, capability, operation, input,
                )
        })
        .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }

    /// Ensures an approved-elevation managed index is `Ready` for `parent_intent`,
    /// rebuilding ONLY if its status is `Created`.
    pub fn ensure_index_ready_approved<Intent, Capability, Operation, Input>(
        &self,
        parent_intent: Intent,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<WorthQueryManagedIndex<Schema, Intent::Index>, WorthQueryApplicationRequestQueryDenial>
    where
        Intent: ApplicationManagedIndexQueryIntent<Schema>,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = QueryScope<Schema, Intent>>
            + 'static,
{
        let scope = self.fresh_scope();
        let request = self.request(&scope);
        self.with_funded_advancement(&scope, |phase| {
            request
                .query(parent_intent)
                .ensure_index_ready_approved_in_advancement::<Intent::Index, _, _, _>(
                    &phase, approved, capability, operation, input,
                )
        })
        .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }
}
