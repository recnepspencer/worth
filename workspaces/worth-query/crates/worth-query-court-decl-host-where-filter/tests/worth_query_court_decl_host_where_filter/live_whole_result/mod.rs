//! End-to-end courts for whole-result live subscriptions (`P2.3` / `G9`)
//! through the public `worth-query-decl` and `worth-query-host` facades:
//! `Today` negative filter evidence and `Ready` related-record evidence.
mod access;
mod budget_refresh;
mod dst;
mod governance_and_durability;
mod interleaving_and_custody;
mod mutations;
mod queries;
mod schema;
mod world;

use mutations::MutateLiveBoard;
use queries::{ReadyIntent, TodayIntent};
use world::{install, live_limits, run_mutate, TaskSeed, FUTURE_DUE_DAYS, TODAY_CUTOFF_DAYS};
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationLiveCloseOutcome, WorthQueryApplicationWholeResultLiveOutcome,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

pub(super) fn seed(key: u64, project: &str, status: &str, due_days: Option<i64>) -> TaskSeed {
    TaskSeed {
        key,
        project: project.to_owned(),
        title: format!("Task {key}"),
        notes: Some(format!("note-{key}")),
        status: status.to_owned(),
        due_days,
    }
}

pub(super) fn mutate_op(
    project: &str,
    op: u8,
    task: u64,
    other: u64,
    due_days: i64,
    text: &str,
) -> MutateLiveBoard {
    MutateLiveBoard {
        project: project.to_owned(),
        op,
        task,
        other,
        due_days,
        text: text.to_owned(),
    }
}

pub(super) fn keys_of<Q>(
    outcome: WorthQueryApplicationWholeResultLiveOutcome<Q, queries::TaskRow>,
) -> Vec<u64> {
    match outcome {
        WorthQueryApplicationWholeResultLiveOutcome::Delivered(update) => {
            update.snapshot().rows().iter().map(|r| r.key).collect()
        }
        other => panic!("expected Delivered, got {other:?}"),
    }
}

#[test]
fn today_whole_result_wakes_on_due_date_entry_exit_and_ignores_unrelated_edits() {
    let host = install(
        &[
            seed(1, "p1", "open", Some(TODAY_CUTOFF_DAYS)),
            seed(2, "p1", "open", Some(FUTURE_DUE_DAYS)),
            seed(3, "p1", "open", None),
            seed(4, "p1", "done", Some(TODAY_CUTOFF_DAYS)),
            seed(5, "p2", "open", Some(FUTURE_DUE_DAYS)),
        ],
        &[],
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let mut sub = host
        .runtime()
        .request(&principal, &scope)
        .query(TodayIntent {
            project: "p1".into(),
            cutoff_days: TODAY_CUTOFF_DAYS,
        })
        .subscribe_whole_result(live_limits())
        .expect("today whole-result subscription opens");
    assert_eq!(
        sub.initial()
            .unwrap()
            .rows()
            .iter()
            .map(|r| r.key)
            .collect::<Vec<_>>(),
        vec![1]
    );
    assert!(matches!(
        sub.next(&host.runtime().request(&principal, &scope))
            .unwrap(),
        WorthQueryApplicationWholeResultLiveOutcome::Pending
    ));

    // 1. Negative evidence: Task 3 has Due = None -> Some(today) => MUST wake & enter!
    run_mutate(&host, mutate_op("p1", 1, 3, 0, TODAY_CUTOFF_DAYS, ""), 1);
    assert_eq!(
        keys_of(
            sub.next(&host.runtime().request(&principal, &scope))
                .unwrap()
        ),
        vec![1, 3]
    );

    // 2. Negative evidence: Task 2 has Due = Some(future) -> Some(today) => MUST wake & enter!
    run_mutate(&host, mutate_op("p1", 1, 2, 0, TODAY_CUTOFF_DAYS, ""), 2);
    assert_eq!(
        keys_of(
            sub.next(&host.runtime().request(&principal, &scope))
                .unwrap()
        ),
        vec![1, 2, 3]
    );

    // 3. Unrelated Project-2 edit MUST NOT wake Project-1 subscription!
    run_mutate(&host, mutate_op("p2", 1, 5, 0, TODAY_CUTOFF_DAYS, ""), 3);
    assert!(matches!(
        sub.next(&host.runtime().request(&principal, &scope))
            .unwrap(),
        WorthQueryApplicationWholeResultLiveOutcome::Pending
    ));

    // 4. Unexamined `Notes` aspect field edit on Task 1 MUST NOT wake!
    run_mutate(&host, mutate_op("p1", 5, 1, 0, 0, "updated-notes"), 4);
    assert!(matches!(
        sub.next(&host.runtime().request(&principal, &scope))
            .unwrap(),
        WorthQueryApplicationWholeResultLiveOutcome::Pending
    ));

    // 5. Positive evidence: projected `Title` edit on returned Task 1 MUST wake!
    run_mutate(&host, mutate_op("p1", 4, 1, 0, 0, "Renamed 1"), 5);
    let update = match sub
        .next(&host.runtime().request(&principal, &scope))
        .unwrap()
    {
        WorthQueryApplicationWholeResultLiveOutcome::Delivered(u) => u,
        other => panic!("expected Delivered on title edit, got {other:?}"),
    };
    assert_eq!(update.snapshot().rows()[0].title, "Renamed 1");

    // 6. Due date cleared (`Some(today) -> None`) => Task 3 leaves Today!
    run_mutate(&host, mutate_op("p1", 1, 3, 0, -1, ""), 6);
    assert_eq!(
        keys_of(
            sub.next(&host.runtime().request(&principal, &scope))
                .unwrap()
        ),
        vec![1, 2]
    );

    // 7. Due date pushed to future (`Some(today) -> Some(future)`) => Task 2 leaves Today!
    run_mutate(&host, mutate_op("p1", 1, 2, 0, FUTURE_DUE_DAYS, ""), 7);
    assert_eq!(
        keys_of(
            sub.next(&host.runtime().request(&principal, &scope))
                .unwrap()
        ),
        vec![1]
    );

    // 8. Negative status evidence: Task 4 flips done -> open (already due today) => enters Today!
    run_mutate(&host, mutate_op("p1", 0, 4, 0, 0, "open"), 8);
    assert_eq!(
        keys_of(
            sub.next(&host.runtime().request(&principal, &scope))
                .unwrap()
        ),
        vec![1, 4]
    );

    assert!(matches!(
        sub.close(),
        WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
}

#[test]
fn ready_whole_result_wakes_on_blocker_completion_reopen_and_edge_changes() {
    let host = install(
        &[
            seed(1, "p1", "open", None),
            seed(2, "p1", "open", None),
            seed(3, "p1", "open", None),
            seed(4, "p2", "open", None),
        ],
        &[(1, 2)],
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let mut sub = host
        .runtime()
        .request(&principal, &scope)
        .query(ReadyIntent {
            project: "p1".into(),
        })
        .subscribe_whole_result(live_limits())
        .expect("ready whole-result subscription opens");
    assert_eq!(
        sub.initial()
            .unwrap()
            .rows()
            .iter()
            .map(|r| r.key)
            .collect::<Vec<_>>(),
        vec![2, 3]
    );

    // 1. Completing blocker Task 2 wakes Ready: Task 2 leaves (done), Task 1 enters (unblocked)!
    run_mutate(&host, mutate_op("p1", 0, 2, 0, 0, "done"), 1);
    assert_eq!(
        keys_of(
            sub.next(&host.runtime().request(&principal, &scope))
                .unwrap()
        ),
        vec![1, 3]
    );

    // 2. Reopening blocker Task 2 wakes Ready: Task 1 leaves (re-blocked), Task 2 enters!
    run_mutate(&host, mutate_op("p1", 0, 2, 0, 0, "open"), 2);
    assert_eq!(
        keys_of(
            sub.next(&host.runtime().request(&principal, &scope))
                .unwrap()
        ),
        vec![2, 3]
    );

    // 3. Removing BlockedBy edge (1 -> 2) wakes Ready: Task 1 enters!
    run_mutate(&host, mutate_op("p1", 3, 1, 2, 0, ""), 3);
    assert_eq!(
        keys_of(
            sub.next(&host.runtime().request(&principal, &scope))
                .unwrap()
        ),
        vec![1, 2, 3]
    );

    // 4. Adding self-loop BlockedBy (3 -> 3) wakes Ready: Task 3 leaves!
    run_mutate(&host, mutate_op("p1", 2, 3, 3, 0, ""), 4);
    assert_eq!(
        keys_of(
            sub.next(&host.runtime().request(&principal, &scope))
                .unwrap()
        ),
        vec![1, 2]
    );

    // 5. Unrelated Project-2 edit & unexamined Notes edit do NOT wake Ready!
    run_mutate(&host, mutate_op("p2", 0, 4, 0, 0, "done"), 5);
    run_mutate(&host, mutate_op("p1", 5, 1, 0, 0, "note"), 6);
    assert!(matches!(
        sub.next(&host.runtime().request(&principal, &scope))
            .unwrap(),
        WorthQueryApplicationWholeResultLiveOutcome::Pending
    ));

    // 6. Cross-project blocker edge (1 -> 4 where 4 is done, then reopen 4): wakes!
    run_mutate(&host, mutate_op("p1", 2, 1, 4, 0, ""), 7);
    assert_eq!(
        keys_of(
            sub.next(&host.runtime().request(&principal, &scope))
                .unwrap()
        ),
        vec![1, 2]
    );
    run_mutate(&host, mutate_op("p2", 0, 4, 0, 0, "open"), 8);
    assert_eq!(
        keys_of(
            sub.next(&host.runtime().request(&principal, &scope))
                .unwrap()
        ),
        vec![2]
    );

    assert!(matches!(
        sub.close(),
        WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
}
