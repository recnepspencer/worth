//! Forks and switches over seeded truth, judged against full computation.
use super::differential::{
    alphabet::{Change, Kind, Lcg, Model},
    prefix,
};
use super::*;
use worth_query_host::facade::application_contribution::{
    WorthQueryPartitionedComputationFullCause as Cause, WorthQueryPartitionedComputationRun as Run,
};
use worth_query_host::facade::product::WorthQueryProductBranch;

fn fork<const MODE: u8>(
    app: &Application<false, TOTALS_WORK, 1, MODE>,
    branch: WorthQueryProductBranch,
) -> WorthQueryProductBranch {
    app.branches()
        .fork(branch)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .unwrap()
}

fn run<const MODE: u8>(
    app: &Application<false, TOTALS_WORK, 1, MODE>,
    branch: WorthQueryProductBranch,
) -> Vec<OracleRun> {
    let (scope, principal) = authenticate(app);
    demand(&app.request(&principal, &scope).on_branch(branch), app).1
}

fn apply<const MODE: u8>(
    app: &Application<false, TOTALS_WORK, 1, MODE>,
    branch: WorthQueryProductBranch,
    changes: Vec<Change>,
    command: &mut u64,
) {
    let (scope, principal) = authenticate(app);
    let request = app.request(&principal, &scope).on_branch(branch);
    for change in changes {
        *command += 1;
        match change {
            Change::Entry(change) => edit(&request, app, change, *command),
            Change::Ordinate(y) => adjust(&request, app, y, *command),
        }
    }
}

fn judge(kept: &[OracleRun], fresh: &[OracleRun], expected: Option<Run>) {
    assert_eq!(
        kept.iter().map(|r| &r.outcome).collect::<Vec<_>>(),
        fresh.iter().map(|r| &r.outcome).collect::<Vec<_>>(),
        "bits, typed outcomes, charged work and named work boundary"
    );
    if kept.is_empty() {
        assert!(
            expected.is_none(),
            "an expected first fork computation must run"
        );
        return;
    }
    assert_published_state(kept, fresh);
    if let Some(expected) = expected {
        assert_eq!(kept[0].runs, [expected]);
    }
    assert!(
        fresh
            .iter()
            .flat_map(|r| &r.runs)
            .all(|r| *r == Run::Full(Cause::Unretained)),
        "reference always computes fresh"
    );
}

fn close_ancestor<const MODE: u8>(
    app: &Application<false, TOTALS_WORK, 1, MODE>,
    branch: WorthQueryProductBranch,
) -> worth_query_host::facade::product::WorthQueryApplicationProductBranchCleanup {
    use worth_query_host::facade::product::{
        WorthQueryApplicationProductBranchCleanupDenial as Cleanup,
        WorthQueryApplicationProductBranchCloseDenial as Close,
        WorthQueryProductBranchOwnerCleanupDenial as Product,
    };
    let Err(Close::OwnerCleanupPending(pending)) = app.on_branch(branch).close() else {
        panic!("the ancestor closes with its live descendant retaining Native history");
    };
    assert!(matches!(
        pending.denial(),
        Cleanup::Product(Product::WorldHistoryStillRetained)
    ));
    pending.into_cleanup()
}

fn clone_change(change: &Change) -> Change {
    match change {
        Change::Entry(edit) => Change::Entry(edit.clone()),
        Change::Ordinate(y) => Change::Ordinate(*y),
    }
}

thread_local! { static REINSTALL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
pub(super) fn reinstallation_requested() -> bool {
    REINSTALL.with(std::cell::Cell::get)
}

fn with_own_write<T>(
    write: Option<super::super::region_output::OwnWrite>,
    run: impl FnOnce() -> T,
) -> T {
    use super::super::region_output::arm_own_write;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            arm_own_write(None);
        }
    }
    let _reset = Reset;
    arm_own_write(write);
    run()
}

mod basis_drift;
mod eviction;
mod seeded;
mod tree_identity;
