//! Issue #423: long Directory sessions close with a checkpoint and reopen.
use super::*;
use host_observation::requests;
use std::sync::{atomic::AtomicUsize, Arc};
use worth_query_host::facade::WorthQueryApplicationHomeLocation as Location;
use worth_store_application::{
    ApplicationHomeCheckpointLimits as Limits, ApplicationHomeCloseOutcome,
};

const KEY_BASE: u64 = 423_000;

#[test]
fn repeated_automatic_checkpoints_then_close_reopen_from_disk() {
    // Eight entries in any journal schedule upkeep; each run and the close
    // capture own this allowance, which the whole session exceeds together.
    let limits = Limits::new(
        std::num::NonZeroU64::new(8).unwrap(),
        std::num::NonZeroU64::new(64 << 20).unwrap(),
    )
    .with_maintenance_allowance(
        std::num::NonZeroU64::new(64 << 20).unwrap(),
        std::num::NonZeroU64::new(256 << 20).unwrap(),
    );
    let automatic = session(limits, 160, true);
    assert!(automatic >= 15, "only {automatic} automatic checkpoints");
}

#[test]
fn long_single_session_closes_with_a_checkpoint_and_reopens_from_disk() {
    let automatic = session(Limits::default(), 300, true);
    assert!(
        automatic >= 3,
        "default cadence must space physical upkeep across 300 commits (saw {automatic})"
    );
}

/// Commits `commits` balances in one session, closes, reopens and returns the
/// automatic checkpoints observed when `observe` inspects every commit.
fn session(limits: Limits, commits: u64, observe: bool) -> usize {
    let path = tempfile::tempdir().unwrap();
    let request = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let open = || {
        host_observation::opening::open(
            Location::Directory(path.path().to_path_buf()),
            opening::build(),
            Arc::new(AtomicUsize::new(0)),
            Arc::new(AtomicUsize::new(0)),
            None,
            limits,
            &execution,
        )
    };
    let home = open();
    let covered = |home: &host_observation::OracleHome| {
        let inspection = home.inspect_checkpoint(&execution).unwrap();
        assert!(inspection.pending.is_none(), "{inspection:?}");
        assert!(!inspection.due, "upkeep fell behind: {inspection:?}");
        inspection.journals.map(|journal| journal.covered)
    };
    let mut prior = observe.then(|| covered(&home));
    let mut automatic = 0;
    for index in 0..commits {
        let outcome = requests::set(home.runtime(), KEY_BASE + index, index + 1);
        assert!(outcome.receipt().is_some(), "commit {index} refused");
        drop(outcome);
        if let Some(before) = prior {
            let after = covered(&home);
            automatic += usize::from(after != before);
            prior = Some(after);
        }
    }
    assert_eq!(requests::balance(home.runtime()), commits);
    let head = home.inspect_checkpoint(&execution).unwrap().journals;
    let closed = home
        .close(&execution)
        .unwrap_or_else(|refused| panic!("close: {refused:?}"));
    match closed.outcome() {
        ApplicationHomeCloseOutcome::Closed { checkpoint, .. } => {
            let checkpoint = checkpoint.as_ref().expect("close checkpoint");
            assert!(checkpoint.covered().0 >= head[0].head);
            assert!(checkpoint.covered_transitions().0 >= head[1].head);
        }
        ApplicationHomeCloseOutcome::CloseRefused { .. } => panic!("quiescent Home must close"),
    }
    closed.reclaim();

    let home = open();
    assert_eq!(requests::balance(home.runtime()), commits);
    let reopened = home.inspect_checkpoint(&execution).unwrap();
    for (journal, before) in reopened.journals.iter().zip(&head) {
        assert!(
            journal.covered >= before.head,
            "{journal:?} lost close coverage"
        );
    }
    let outcome = requests::set(home.runtime(), KEY_BASE + commits, commits + 1);
    assert!(
        outcome.receipt().is_some(),
        "reopened Home refused a commit"
    );
    drop(outcome);
    assert_eq!(requests::balance(home.runtime()), commits + 1);
    assert!(home.close(&execution).unwrap().is_closed());
    automatic
}
