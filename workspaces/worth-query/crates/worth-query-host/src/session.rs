//! Reusable authenticated Host session facade (`home.session(me)`).
//!
//! Each method opens a fresh `WorthQueryRequestScope`, enforces this session's
//! configured `ExecutionRequestPolicy` through a metered lease, re-admits
//! principal and grants on the targeted branch, and executes under explicit
//! finite policy defaults. Nothing caches authority across calls.
use std::borrow::Borrow;
use std::num::NonZeroUsize;
use std::time::Instant;

use worth_execution::ExecutionResourceLease;
use worth_foundational::facade::{ExecutionBudget, ExecutionRequestPolicy};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryAuthenticatedExternalPrincipal,
    WorthQueryCancellationSource, WorthQueryCancellationToken, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeBinding,
    },
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_query_execution::facade::{
    application_contribution::{WorthQueryAdvancementDenial, WorthQueryAdvancementPhase},
    primary_graph::{
        WorthQueryApplicationCommitReceipt, WorthQueryApplicationUncommitted,
        WorthQueryRecoveredApplicationCommitReceipt,
    },
    product::WorthQueryProductBranch,
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationRequest, WorthQueryApplicationRequestExt,
    WorthQueryApplicationRequestMutationDenial,
};

use crate::application_home::{
    WorthQueryApplicationHome, WorthQueryCorrectionEntry, DEFAULT_HOST_EXECUTION_POLICY,
};

pub(crate) mod auto_key;
mod execution;
mod lease;
mod live_subscription;
mod managed_index;
mod query_execution;
pub use auto_key::WorthQueryAutoIdempotencyKey;
pub use live_subscription::{
    WorthQueryTypedLiveCloseDenial, WorthQueryTypedLiveRefreshDenial,
    WorthQueryTypedLiveSubscription,
};
pub use managed_index::{
    WorthQueryTypedManagedIndexRefreshDenial, WorthQueryTypedManagedIndexWatch,
};

const DEFAULT_LIVE_BUFFER_CAPACITY: usize = 64;
const DEFAULT_LIVE_MAXIMUM_RESULTS: usize = 1024;

pub(super) type QueryBinding<S, I> = <I as ApplicationQueryIntent<S>>::Binding;
pub(super) type QueryMarker<S, I> = <QueryBinding<S, I> as ApplicationQueryBinding<S>>::Query;
pub(super) type QueryRow<S, I> = <<QueryBinding<S, I> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
pub(super) type QueryScope<S, I> = <<QueryBinding<S, I> as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope;

/// A committed mutation result paired with its live commit receipt.
#[derive(Debug)]
pub struct WorthQuerySessionCommit<ResultValue> {
    pub(super) receipt: WorthQueryApplicationCommitReceipt,
    pub(super) result: ResultValue,
}

impl<ResultValue> WorthQuerySessionCommit<ResultValue> {
    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub const fn result(&self) -> &ResultValue {
        &self.result
    }

    pub fn into_result(self) -> ResultValue {
        self.result
    }

    pub fn into_parts(self) -> (WorthQueryApplicationCommitReceipt, ResultValue) {
        (self.receipt, self.result)
    }

    /// Captures a `'static` [`WorthQueryCorrectionEntry`] suitable for
    /// `home.undo(...)` or `home.undo_authored(...)` even after reopen.
    pub fn correction_entry(&self) -> WorthQueryCorrectionEntry {
        WorthQueryCorrectionEntry::from_receipt(&self.receipt)
    }
}

/// Every non-fresh-commit outcome from [`WorthQueryHostSession::run`] and
/// [`WorthQueryHostSession::run_with_key`], preserving typed denials and
/// uncommitted recovery custody without hiding any case.
#[derive(Debug)]
pub enum WorthQuerySessionRunDenial<DomainDenial> {
    Request(WorthQueryApplicationRequestMutationDenial),
    DomainDenied(DomainDenial),
    AlreadyCommitted(WorthQueryApplicationCommitReceipt),
    RecoveredCommitted(WorthQueryRecoveredApplicationCommitReceipt),
    IdempotencyIntentDrift,
    Cancelled,
    DeadlineExceeded,
    Uncommitted(WorthQueryApplicationUncommitted),
}

/// Reusable authenticated session over a [`WorthQueryApplicationHome`].
///
/// Construct via [`WorthQueryApplicationHome::session`]. Every call opens a
/// fresh [`WorthQueryRequestScope`], enforces this session's
/// [`ExecutionRequestPolicy`], and re-admits authority on the configured
/// branch with explicit bounded defaults.
pub struct WorthQueryHostSession<'home, Schema: ApplicationSchema, Program> {
    pub(super) home: &'home WorthQueryApplicationHome<Schema, Program>,
    pub(super) principal: WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(super) branch: WorthQueryProductBranch,
    pub(super) clock: WorthQueryAdmissionClock,
    pub(super) policy: ExecutionRequestPolicy,
    pub(super) deadline: Option<Instant>,
    pub(super) cancellation: Option<WorthQueryCancellationToken>,
    pub(super) live_buffer_capacity: usize,
    pub(super) live_maximum_results: Option<usize>,
    pub(super) query_result_limit: Option<NonZeroUsize>,
}

impl<Schema: ApplicationSchema, Program> Clone for WorthQueryHostSession<'_, Schema, Program> {
    fn clone(&self) -> Self {
        Self {
            home: self.home,
            principal: self.principal.clone(),
            branch: self.branch,
            clock: self.clock.clone(),
            policy: self.policy,
            deadline: self.deadline,
            cancellation: self.cancellation.clone(),
            live_buffer_capacity: self.live_buffer_capacity,
            live_maximum_results: self.live_maximum_results,
            query_result_limit: self.query_result_limit,
        }
    }
}

impl<Schema: ApplicationSchema, Program> WorthQueryApplicationHome<Schema, Program> {
    /// Creates a reusable authenticated Host session bound to this Home's
    /// current active branch and explicit bounded execution defaults.
    pub fn session<'home>(
        &'home self,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
    ) -> WorthQueryHostSession<'home, Schema, Program> {
        let principal = principal.borrow().clone();
        let clock = principal.clock().clone();
        WorthQueryHostSession {
            home: self,
            principal,
            branch: self.active_branch(),
            clock,
            policy: DEFAULT_HOST_EXECUTION_POLICY,
            deadline: None,
            cancellation: None,
            live_buffer_capacity: DEFAULT_LIVE_BUFFER_CAPACITY,
            live_maximum_results: None,
            query_result_limit: None,
        }
    }
}

impl<'home, Schema: ApplicationSchema, Program> WorthQueryHostSession<'home, Schema, Program> {
    pub const fn home(&self) -> &'home WorthQueryApplicationHome<Schema, Program> {
        self.home
    }

    pub const fn branch(&self) -> WorthQueryProductBranch {
        self.branch
    }

    pub const fn policy(&self) -> ExecutionRequestPolicy {
        self.policy
    }

    pub const fn clock(&self) -> &WorthQueryAdmissionClock {
        &self.clock
    }

    pub const fn principal(&self) -> &WorthQueryAuthenticatedExternalPrincipal<Schema> {
        &self.principal
    }

    /// Replaces this session's bounded execution policy.
    pub const fn with_policy(mut self, policy: ExecutionRequestPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Replaces only the execution budget inside this session's policy.
    pub const fn with_budget(mut self, budget: ExecutionBudget) -> Self {
        self.policy =
            ExecutionRequestPolicy::new(self.policy.posture(), self.policy.determinism(), budget);
        self
    }

    /// Replaces the admission clock used when opening fresh request scopes.
    pub fn with_clock(mut self, clock: WorthQueryAdmissionClock) -> Self {
        self.clock = clock;
        self
    }

    /// Attaches a wall-clock deadline to fresh request scopes built by this session.
    pub const fn with_deadline(mut self, deadline: Option<Instant>) -> Self {
        self.deadline = deadline;
        self
    }

    /// Attaches a cancellation token to fresh request scopes built by this session.
    pub fn with_cancellation(mut self, cancellation: WorthQueryCancellationToken) -> Self {
        self.cancellation = Some(cancellation);
        self
    }

    /// Configures bounded ring buffer and result limits for live subscriptions.
    pub const fn with_live_limits(
        mut self,
        buffer_capacity: usize,
        maximum_results: usize,
    ) -> Self {
        self.live_buffer_capacity = buffer_capacity;
        self.live_maximum_results = Some(maximum_results);
        self
    }

    /// Narrows the per-query result limit ceiling for one-shot, live, and watch reads.
    pub const fn with_result_limit(mut self, maximum_results: NonZeroUsize) -> Self {
        self.query_result_limit = Some(maximum_results);
        self
    }

    /// Targets a specific product branch (such as a [`WorthQueryPreviewFork::branch`](crate::application_home::WorthQueryPreviewFork::branch)).
    pub const fn on_branch(mut self, branch: WorthQueryProductBranch) -> Self {
        self.branch = branch;
        self
    }

    /// Constructs a fresh request scope for one operation.
    pub fn fresh_scope(&self) -> WorthQueryRequestScope {
        WorthQueryRequestScope::new(
            self.deadline,
            self.cancellation
                .clone()
                .unwrap_or_else(|| WorthQueryCancellationSource::new().token()),
            self.clock.clone(),
        )
    }

    /// Borrows a fresh [`WorthQueryApplicationRequest`] bound to this session's
    /// Home, principal, and target branch.
    pub fn request<'a>(
        &'a self,
        scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryApplicationRequest<'home, 'a, 'a, Schema> {
        self.home
            .primary_graph()
            .request(&self.principal, scope)
            .on_branch(self.branch)
    }

    pub(super) fn open_lease(
        &self,
        scope: &WorthQueryRequestScope,
    ) -> Result<ExecutionResourceLease<'home>, WorthQueryAdvancementDenial> {
        lease::open_policy_lease(self.home.primary_graph(), self.policy, scope)
    }

    pub(super) fn with_funded_advancement<R>(
        &self,
        scope: &WorthQueryRequestScope,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        lease::with_policy_advancement(self.home.primary_graph(), self.policy, scope, body)
    }
}
