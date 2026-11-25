//! Managed-index `session.watch(...)` and `session.ensure_index_ready(...)` helpers.
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

/// Typed managed-index watch holding the latest projected rows and the
/// underlying [`WorthQueryManagedIndexWatch`].
#[must_use = "close the managed-index watch when finished"]
pub struct WorthQueryTypedManagedIndexWatch<Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationManagedIndexQueryIntent<Schema>,
    QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
    intent: Intent,
    watch: WorthQueryManagedIndexWatch<Schema, Intent::Index>,
    current: Vec<QueryRow<Schema, Intent>>,
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
    /// The latest projected rows from the managed-index watch.
    pub fn current(&self) -> &[QueryRow<Schema, Intent>] {
        &self.current
    }

    /// The latest notification publication that triggered a refresh, if any.
    pub const fn last_notification(&self) -> Option<&WorthQueryCommittedProductPublication> {
        self.last_notification.as_ref()
    }

    /// Refreshes the watch under a fresh request scope and fresh principal
    /// admission from `session`.
    pub fn refresh<Program>(
        &mut self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
    ) -> Result<Option<&[QueryRow<Schema, Intent>]>, WorthQueryTypedManagedIndexRefreshDenial> {
        let scope = session.fresh_scope();
        let fresh_request = session.request(&scope);
        self.refresh_with_request(&fresh_request)
    }

    /// Refreshes the watch using an explicit fresh [`WorthQueryApplicationRequest`].
    pub fn refresh_with_request(
        &mut self,
        fresh_request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<Option<&[QueryRow<Schema, Intent>]>, WorthQueryTypedManagedIndexRefreshDenial> {
        use WorthQueryManagedIndexWatchOutcome as Outcome;
        use WorthQueryTypedManagedIndexRefreshDenial as Denial;
        let outcome = fresh_request
            .query(self.intent.clone())
            .next_watch(&mut self.watch)
            .map_err(Denial::Query)?;
        match outcome {
            Outcome::Pending => Ok(None),
            Outcome::Delivered(update) => {
                self.last_notification = Some(update.notification().clone());
                self.current = update.into_read().into_rows();
                Ok(Some(&self.current))
            }
            Outcome::Overflow {
                missed_commit_batches,
            } => Err(Denial::Overflow {
                missed_commit_batches,
            }),
            Outcome::Unavailable => Err(Denial::Unavailable),
            Outcome::Closed => Err(Denial::Closed),
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
        let (initial, watch) = query_req.watch_index::<Intent::Index>()?;
        Ok(WorthQueryTypedManagedIndexWatch {
            intent,
            watch,
            current: initial.into_rows(),
            last_notification: None,
        })
    }

    /// Ensures the managed index backing `parent_intent` is `Ready`, rebuilding
    /// ONLY if its status is `Created` (such as after cold reopen).
    pub fn ensure_index_ready<Intent>(
        &self,
        parent_intent: Intent,
    ) -> Result<WorthQueryManagedIndex<Schema, Intent::Index>, WorthQueryApplicationRequestQueryDenial>
    where
        Intent: ApplicationManagedIndexQueryIntent<Schema>,
        <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <QueryBinding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
{
        let scope = self.fresh_scope();
        let request = self.request(&scope);
        request
            .query(parent_intent)
            .ensure_index_ready::<Intent::Index>()
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
        request
            .query(parent_intent)
            .ensure_index_ready_governed::<Intent::Index, _, _, _>(capability, operation, input)
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
        request
            .query(parent_intent)
            .ensure_index_ready_approved::<Intent::Index, _, _, _>(
                approved, capability, operation, input,
            )
    }
}
