//! Reusable authenticated Host session facade (`home.session(me)`).
//!
//! Each method opens a fresh `WorthQueryRequestScope`, re-admits principal and
//! grants on the targeted branch, and executes under explicit finite policy
//! defaults. Nothing caches authority across calls.
use std::borrow::Borrow;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

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

mod execution;
mod live_subscription;
mod managed_index;
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

/// Restart-safe auto idempotency key generation for [`WorthQueryHostSession::run`].
///
/// Mixes a process-startup wall-clock seed with a monotonic atomic counter so
/// keys generated across separate launches of a persistent Directory Home do
/// not collide. Call [`WorthQueryHostSession::run_with_key`] whenever you need
/// deterministic caller-chosen replay keys.
pub trait WorthQueryAutoIdempotencyKey: Sized {
    fn next_restart_safe_key() -> Self;
}

fn next_restart_safe_pair() -> (u64, u64) {
    static SEQUENCE: AtomicU64 = AtomicU64::new(1);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos() as u64);
    let seq = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    (nanos, seq)
}

impl WorthQueryAutoIdempotencyKey for u64 {
    fn next_restart_safe_key() -> Self {
        let (nanos, seq) = next_restart_safe_pair();
        let mixed = (nanos << 20) ^ seq.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        if mixed == 0 {
            1
        } else {
            mixed
        }
    }
}

impl WorthQueryAutoIdempotencyKey for i64 {
    fn next_restart_safe_key() -> Self {
        (u64::next_restart_safe_key() & (i64::MAX as u64)).max(1) as i64
    }
}

impl WorthQueryAutoIdempotencyKey for u128 {
    fn next_restart_safe_key() -> Self {
        let (nanos, seq) = next_restart_safe_pair();
        ((u128::from(nanos)) << 64) | u128::from(seq)
    }
}

impl WorthQueryAutoIdempotencyKey for String {
    fn next_restart_safe_key() -> Self {
        let (nanos, seq) = next_restart_safe_pair();
        format!("worth-session-{nanos:016x}-{seq:016x}")
    }
}

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
/// fresh [`WorthQueryRequestScope`] and re-admits authority on the configured
/// branch with explicit bounded defaults.
#[derive(Clone)]
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

    /// Narrows the per-query result limit ceiling for one-shot and live reads.
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
}
