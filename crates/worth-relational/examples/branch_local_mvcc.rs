//! Branch-local MVCC through the public Relational owner services.
//!
//! The progression is: seed `main` through the ordinary shared-receiver
//! convenience path, fork a second branch through the fork service, then
//! publish on both unrelated branches at once with each worker holding its own
//! clone of the owner services.
//!
//! This example is executed, not merely compiled, by
//! `cargo test -p worth-relational`. The prose model it demonstrates is
//! `BRANCH_LOCAL_MVCC.md`.

use std::thread;

use worth_relational::facade::{
    branch::{AdmittedRelationalBranchBasis, RelationalBranchIdentity, RelationalForkPort},
    history::BranchId,
    mvcc::{
        BranchBoundRelationalTransaction, CommitResult, PerformedRelationalCommit,
        RelationalPreparationPort, RelationalPublicationDeferred, RelationalPublicationOutcome,
        RelationalPublicationPort, RelationalSettlementPort, RelationalTransactionIntent,
        WorkerIntentBatch,
    },
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::RelationalSchemaRegistry,
};

/// Caller-owned bound on fresh publication attempts for one branch.
///
/// The global patch-position reservation is a bounded, nonblocking mechanical
/// reservation, so losing it occasionally is expected and losing it many times
/// consecutively is a defect. Exhausting this budget fails by name; it never
/// spins and never waits.
const PUBLICATION_ATTEMPT_BUDGET: u32 = 8;

/// The forked branch this example publishes to concurrently with `main`.
const REPORTS_BRANCH: &str = "reports";

/// The four independently borrowable Relational owner services.
///
/// Each is obtained from a *shared* borrow of the runtime and is
/// `Clone + Send + Sync`, so every branch worker holds its own without
/// excluding another worker and without wrapping the runtime in a mutex. That
/// is the point of the owner-service split: the runtime is not the unit of
/// exclusion, the branch reference cell is.
#[derive(Clone)]
struct OwnerServices {
    preparation: RelationalPreparationPort,
    publication: RelationalPublicationPort,
    settlement: RelationalSettlementPort,
    forking: RelationalForkPort,
}

impl OwnerServices {
    fn obtain(runtime: &RelationalRuntime) -> Self {
        Self {
            preparation: runtime.preparation_port(),
            publication: runtime.publication_port(),
            settlement: runtime.settlement_port(),
            forking: runtime.fork_port(),
        }
    }
}

fn main() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .build();
    let services = OwnerServices::obtain(&runtime);
    let main_branch = runtime.main_branch_identity();

    // The ordinary convenience path takes a shared runtime receiver and runs
    // the same prepare/publish/settle progression through the same services.
    // It runs before any worker exists, so no mechanical reservation can
    // contend with it; it also gives `main` the commit a fork source requires.
    let seeded = ordinary_convenience_commit(&runtime, &main_branch);
    release_commit_snapshot(&runtime, &seeded);

    // Fork is a separate owner transition that shares an immutable root.
    let reports = fork_reports_branch(&runtime, &services);

    // Two unrelated branch cells move at once through one runtime.
    let branches = [main_branch, reports];
    for committed in publish_concurrently(&runtime, &services, &branches) {
        release_commit_snapshot(&runtime, &committed);
    }
}

/// Commit through the ordinary convenience path with an explicit allocation policy.
///
/// This is the ordinary path for callers that do not need the explicit
/// linearization outcome. It is not a second authority: it delegates to the
/// same preparation, publication, and settlement services used below.
fn ordinary_convenience_commit(
    runtime: &RelationalRuntime,
    branch: &RelationalBranchIdentity,
) -> CommitResult {
    let basis = observe(runtime, branch);
    fresh_transaction(runtime, &basis, "branch-local-mvcc-seed")
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("an uncontended convenience commit settles through its owner")
}

/// Fork `reports` from the current `main` root through the fork service.
///
/// `observe_fork_source` issues a linear, fork-only source token and
/// `fork_branch` consumes it exactly once. The fork shares the exact immutable
/// source root and canonical commit artifact; it copies no authoritative truth
/// and never infers its source from a branch label.
fn fork_reports_branch(
    runtime: &RelationalRuntime,
    services: &OwnerServices,
) -> RelationalBranchIdentity {
    let source_branch = runtime.main_branch_identity().branch_id().clone();
    let (_descriptor, source) = services
        .forking
        .observe_fork_source(&source_branch)
        .expect("a committed source branch issues one fork-only source basis");
    services
        .forking
        .fork_branch(BranchId(REPORTS_BRANCH.to_owned()), source)
        .expect("an unused target name accepts the owner-sealed fork source")
        .target_identity()
        .clone()
}

/// Publish every branch simultaneously, one worker each.
///
/// Each worker takes its own clone of the owner services. A worker that fails
/// is not summarized away: its panic payload is resumed on this thread so the
/// worker's own named diagnostic is what the reader sees.
fn publish_concurrently(
    runtime: &RelationalRuntime,
    services: &OwnerServices,
    branches: &[RelationalBranchIdentity],
) -> Vec<CommitResult> {
    thread::scope(|scope| {
        let workers: Vec<_> = branches
            .iter()
            .map(|branch| {
                let services = services.clone();
                scope.spawn(move || publish_one_branch(runtime, &services, branch))
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| match worker.join() {
                Ok(committed) => committed,
                Err(worker_panic) => std::panic::resume_unwind(worker_panic),
            })
            .collect()
    })
}

/// Move one branch reference, retrying only what the owner contract says to
/// retry.
///
/// The branch is observed once. A contended patch-position reservation is a
/// mechanical no-movement outcome that does not invalidate the caller's
/// observation, so a retry reuses this still-admitted basis and pays for no
/// second observation. Every other terminal outcome is reported, not retried.
fn publish_one_branch(
    runtime: &RelationalRuntime,
    services: &OwnerServices,
    branch: &RelationalBranchIdentity,
) -> CommitResult {
    let basis = observe(runtime, branch);

    for attempt in 1..=PUBLICATION_ATTEMPT_BUDGET {
        let transaction = fresh_transaction(runtime, &basis, "branch-local-mvcc-worker");
        // Preparation is fallible before any effect, and one of the ways it
        // refuses is bounded: it reserves this candidate's published-snapshot
        // handle here, so an exhausted budget is reported as
        // `PublicationDeferred { deferred: PublishedSnapshotCapacityExhausted
        // { .. } }` and no candidate is created. This example's default
        // budgets make that unreachable, so a refusal here is a defect report
        // rather than a claim that preparation always succeeds.
        let candidate = services
            .preparation
            .prepare_branch_transaction(
                transaction,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect(concat!(
                "preparation refused before producing a candidate; the branch ",
                "did not move, and this example models neither a bounded ",
                "preparation refusal nor a validation denial",
            ));

        match services.publication.compare_and_publish(candidate) {
            RelationalPublicationOutcome::Performed(performed) => {
                return settle(services, performed);
            }
            RelationalPublicationOutcome::Deferred(
                RelationalPublicationDeferred::PatchPositionReservationContended,
            ) => {
                // The candidate was consumed and its prepared residue released.
                // The documented next action is a fresh transaction and a fresh
                // preparation from the same still-admitted basis, which is
                // exactly what the next turn of this loop does. The owner never
                // retries, rebases, or returns a reusable candidate.
                continue;
            }
            outcome => panic!(
                "attempt {attempt} on branch {branch:?} reached a terminal \
                 no-movement outcome this example does not model: {outcome:?}"
            ),
        }
    }

    panic!(
        "the global patch-position reservation contended \
         {PUBLICATION_ATTEMPT_BUDGET} times consecutively for branch \
         {branch:?}; that reservation is bounded and nonblocking, so this is a \
         defect rather than a busy runtime"
    );
}

/// Settle a performed publication through the settlement service.
///
/// The branch reference has already moved. Dropping performed evidence without
/// settling it is not success, so this obligation belongs to whichever worker
/// won linearization.
fn settle(services: &OwnerServices, performed: PerformedRelationalCommit) -> CommitResult {
    services
        .settlement
        .settle_performed_publication(performed)
        .expect("a performed publication settles through its owner")
}

/// Observe and admit one exact basis for a branch this runtime owns.
fn observe(
    runtime: &RelationalRuntime,
    branch: &RelationalBranchIdentity,
) -> AdmittedRelationalBranchBasis {
    let (_descriptor, basis) = runtime
        .observe_branch(branch)
        .expect("an owner-issued identity observes its own branch");
    basis
}

/// Begin one detached, branch-bound transaction from an exact admitted basis.
///
/// The transaction owns its overlay, footprint, and retained basis, and holds
/// no borrow of the runtime for its lifetime. The batch is empty on purpose:
/// this example demonstrates authority and publication, not domain schema.
fn fresh_transaction(
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    batch_name: &str,
) -> BranchBoundRelationalTransaction {
    let mut transaction = runtime
        .begin_branch_transaction(basis, RelationalTransactionIntent::ordinary())
        .expect("an exact admitted basis admits a branch transaction");
    transaction
        .push_batch(
            WorkerIntentBatch::new(batch_name),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("one empty batch stages within the declared transaction budget");
    transaction
}

/// Release the pinned snapshot every settled commit hands back.
fn release_commit_snapshot(runtime: &RelationalRuntime, committed: &CommitResult) {
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .expect("a commit snapshot releases exactly once");
}
