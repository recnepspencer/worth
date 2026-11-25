//! Branches forked from an upgraded head before its successor managed index
//! readies share that vacant head. Each branch attaches its own Source image,
//! so readying on one never makes the other refuse `ForeignSource`, in either
//! order, and the active branch keeps answering after the preview handle is gone.

use worth_query_host::facade::{
    primary_graph::WorthQueryManagedIndexLifecycle,
    product::WorthQueryProductBranch,
    program_transition::{worth_query_default_build_id, WorthQueryTransitionOpenOutcome},
};
use worth_query_testkit_host::facade::host::{
    application_home as fixture, application_runtime_surface as identity, checkpoint_capture,
};

use super::driver::request_scope;
use super::journey::seed_v2_tasks;
use super::managed_index::{open_indexed, IndexedHome};
use super::v3_indexed_program::FindTasksByValuePrefix;

/// Readies the prefix index on `branch` (the active branch when `None`) and
/// returns the task keys whose `V2Value` starts with "mill".
fn mill_keys(home: &IndexedHome, branch: Option<WorthQueryProductBranch>) -> Vec<String> {
    let scope = request_scope();
    let principal = identity::inspection_authentication::authenticate(
        home.runtime().runtime().installed_schema(),
        &scope,
    );
    let session = home.session(&principal);
    let session = match branch {
        Some(branch) => session.on_branch(branch),
        None => session,
    };
    let query = || FindTasksByValuePrefix {
        project: "p-main".to_owned(),
        prefix: "mill".to_owned(),
    };
    let index = session
        .ensure_index_ready(query())
        .expect("each branch sharing the vacant head readies its own index");
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    let keys = session
        .query(query())
        .expect("indexed query answers on its own branch")
        .into_iter()
        .map(|row| row.task_key)
        .collect();
    let _ = index.close();
    keys
}

fn fork_before_ready(preview_first: bool) {
    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let tasks = [
        ("t1", "Mill housing"),
        ("t2", "Drill flange"),
        ("t3", "mill base"),
    ];
    seed_v2_tasks(&location, build, execution, &tasks);
    let (home, outcome) = open_indexed(&location, build, execution);
    assert!(matches!(
        outcome,
        WorthQueryTransitionOpenOutcome::UpgradedInPlace { .. }
    ));

    let expected = ["t1".to_owned(), "t3".to_owned()];
    let preview = home
        .fork_preview("preview")
        .expect("preview forks the vacant head");
    if preview_first {
        assert_eq!(mill_keys(&home, Some(preview.branch())), expected);
        assert_eq!(mill_keys(&home, None), expected);
    } else {
        assert_eq!(mill_keys(&home, None), expected);
        assert_eq!(mill_keys(&home, Some(preview.branch())), expected);
    }
    // This fixture's world resources refuse a preview close (even right after
    // the fork) with `CapacityExhausted`, so the handle is released instead.
    drop(preview);
    assert_eq!(mill_keys(&home, None), expected);
    home.close(&execution).unwrap().reclaim();
}

#[test]
fn preview_forked_before_ready_readies_first_and_active_branch_still_answers() {
    fork_before_ready(true);
}

#[test]
fn active_branch_readies_first_and_preview_forked_before_ready_still_answers() {
    fork_before_ready(false);
}
