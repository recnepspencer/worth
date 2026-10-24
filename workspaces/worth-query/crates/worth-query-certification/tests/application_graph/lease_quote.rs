//! Learns the lease a call needs to reach a later owner.
//!
//! One caller lease funds a call's admission and every charge after it. A
//! court proving a later owner's refusal sizes the lease from the earlier
//! owners' own refusals (MENTALITY §7): each adds `requested - admitted`
//! until the call reaches the owner under proof.
//!
//! The lease keeps a full declared budget and holds all but its funded room.
//! A commit call runs on a [`tracked`] child that declares no memory of its
//! own: owners that size a declared share from a lease's budget (a native
//! query packet's result, a commit preparation's scratch) then charge the
//! bytes they actually keep, so every quote is an actual charge.
//!
//! A mutation call resolves its principal through a native query whose
//! transient bound passes any later handler owner's quote, so no room fixed
//! before the call isolates that owner. Its court instead takes a decision
//! hold ([`keep_at_decision`]) once the handler decides, leaving the lease
//! exactly the learned keep for the owners after the decision.
use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};
use worth_query_host::facade::{
    application_contribution::WorthQueryManagedComputationResourceDenial as Resource,
    primary_graph::{
        WorthQueryApplicationCommitDenial as CommitDenial,
        WorthQueryApplicationCommitDenialKind as CommitKind,
        WorthQueryOperationAuthorizationDenialKind as Authorization,
        WorthQueryProviderSessionDenialKind as Session,
    },
    runtime::{
        CancellationToken, ExecutionAllocationDenial, ExecutionAllocationDenialKind,
        ExecutionResourceLease, LeaseDenial, LeaseRequest, MemoryLimitDenial,
    },
};

/// The budget every learned lease declares; its unfunded part stays held.
pub(crate) const DECLARED_BUDGET: u64 = 16 * 1024 * 1024;

/// Runs `call` while `lease` has exactly `room` of its declared budget free,
/// then releases the held remainder.
pub(crate) fn with_room<T>(
    lease: &ExecutionResourceLease<'_>,
    room: u64,
    call: impl FnOnce() -> T,
) -> T {
    let held = lease
        .reserve_memory(
            DECLARED_BUDGET
                .checked_sub(room)
                .expect("the learned room fits the budget"),
        )
        .expect("a fresh lease holds its unfunded budget");
    let outcome = call();
    drop(held);
    outcome
}

/// A child of `parent` that declares no memory: its charges are the actual
/// bytes each owner keeps, admitted by `parent`'s room. One more controlled
/// level then sits between an owner's worker and `parent`.
pub(crate) fn tracked<'a>(parent: &ExecutionResourceLease<'a>) -> ExecutionResourceLease<'a> {
    let policy = parent.policy();
    parent
        .child(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                policy.posture(),
                policy.determinism(),
                ExecutionBudget::new(policy.max_workers(), None, policy.work_ceiling()),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .expect("a child without a memory declaration fits its parent")
}

/// A decision hold leaving `lease` exactly `keep` bytes of room once the
/// handler decides, for `with_decision_hold`. The holder is a controlled
/// child, so the hold charges `lease` and an owner refusing inside the
/// call still quotes `lease` as the ancestor whose room ran out.
pub(crate) fn keep_at_decision(
    lease: &ExecutionResourceLease<'static>,
    keep: u64,
) -> impl FnOnce() -> Box<dyn std::any::Any> + 'static {
    let holder = lease.controlled_child(CancellationToken::new(), None);
    move || {
        let room = holder
            .available_memory_bytes()
            .expect("the court's lease declares memory");
        let held = holder
            .reserve_memory(
                room.checked_sub(keep)
                    .expect("the learned keep fits the room"),
            )
            .expect("the observed room admits the hold");
        Box::new((held, holder))
    }
}

/// What an owner still lacked when it refused for lease memory.
pub(crate) fn allocation_shortfall(cause: &ExecutionAllocationDenial) -> Option<u64> {
    match cause.kind() {
        ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(
            MemoryLimitDenial::Capacity {
                requested,
                admitted,
                ..
            },
        )) => Some(requested - admitted),
        _ => None,
    }
}

/// A bound, not a count: learning ends when the call reaches its owner. A
/// lease leak would never converge and stops here with every quote it saw.
const LEARNING_RUNS: usize = 64;

/// Runs `attempt(funded)` from `funded`, adding each earlier owner's
/// shortfall, until `shortfall` finds none; returns that run and its lease.
pub(crate) fn learn<T>(
    mut funded: u64,
    mut attempt: impl FnMut(u64) -> T,
    shortfall: impl Fn(&T) -> Option<u64>,
) -> (T, u64) {
    let mut quotes = Vec::new();
    for _ in 0..LEARNING_RUNS {
        let outcome = attempt(funded);
        match shortfall(&outcome) {
            Some(missing) => {
                quotes.push(missing);
                funded += missing;
            }
            None => return (outcome, funded),
        }
    }
    panic!("the call never reached its owner; earlier owners quoted {quotes:?}")
}

fn session_shortfall(kind: &Session) -> Option<u64> {
    match kind {
        Session::ExecutionResource {
            denial:
                Resource::MemoryLimit {
                    requested,
                    admitted,
                    ..
                },
            ..
        } => Some(requested - admitted),
        _ => None,
    }
}

/// What a commit call still lacked when an owner before its read set refused
/// for lease memory: its admission, provider plan or request authority.
pub(crate) fn commit_shortfall(denial: &CommitDenial) -> Option<u64> {
    if let Some(Authorization::Stopped(Ok(kind))) = denial.authorization_denial_kind() {
        return session_shortfall(&kind);
    }
    if denial.decision_read_set_failure().is_some()
        || denial.invariant_execution_failure().is_some()
    {
        return None;
    }
    match denial.kind() {
        CommitKind::ExecutionResource {
            denial:
                Resource::MemoryLimit {
                    requested,
                    admitted,
                    ..
                },
            ..
        } => Some(requested - admitted),
        _ => None,
    }
}
