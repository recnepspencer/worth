//! End-to-end courts for Daymark Plan Wave 2 — PR 2B (`P2.2` / `G8`):
//! bounded, metered related-record absence/existence predicates (`NOT EXISTS`
//! / `EXISTS`), exact invalidation footprints (including out-of-scope blockers),
//! pinned/retained reads, metering/early fanout refusal, governance fail-closed
//! checks, and the 64-seed Differential Query DST + Reference Graph Oracle.
mod dst;
mod governance;
mod metering;
mod mutations;
mod queries;
mod schema;
mod world;

use mutations::MutateBoard;
use queries::*;
use world::{install, observe_ready_source, ready_with_source, run_mutate, Host, TaskSeed};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryApplicationRequestMutationDenial,
};
use worth_query_host::facade::primary_graph::{
    MutationHandlerExecutionDenial, WorthQueryApplicationAttemptDenialKind,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

pub(super) fn task(key: u64, project: u64, status: &str, rank: i64) -> TaskSeed {
    TaskSeed {
        key,
        project,
        title: format!("Task {key}"),
        notes: None,
        status: status.to_owned(),
        rank,
        due_days: None,
    }
}

fn daymark_board() -> Host {
    // Project 1:
    //   1 (open): no blockers -> READY
    //   2 (open): blocked by 3 (open) -> BLOCKED
    //   3 (open): no blockers -> READY (and blocks 2)
    //   4 (open): blocked by 5 (done) -> READY (blocker is closed!)
    //   5 (done): closed blocker
    // Project 2:
    //   6 (open): unrelated project task (can also act as an out-of-scope blocker!)
    let tasks = vec![
        task(1, 1, "open", 10),
        task(2, 1, "open", 5),
        task(3, 1, "open", 8),
        task(4, 1, "open", 3),
        task(5, 1, "done", 1),
        task(6, 2, "open", 9),
    ];
    install(&tasks, &[(2, 3), (4, 5)])
}

#[track_caller]
fn assert_stale(result: Result<(), WorthQueryApplicationRequestMutationDenial>) {
    let Err(WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(cause),
    )) = result
    else {
        panic!("expected SourceChanged invalidation, got: {result:?}");
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationAttemptDenialKind::SourceChanged
    );
}

#[test]
fn daymark_ready_view_tracks_blocker_completion_reopen_edges_and_out_of_scope_blockers() {
    let host = daymark_board();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);

    // Initial: 1, 3, 4 are ready; 2 is blocked by open task 3.
    let (ready, source0) = ready_with_source(&host, 1);
    assert_eq!(ready, [1, 3, 4]);
    observe_ready_source(&host, 1, &source0, "unchanged").expect("unchanged source stays current");

    // Unrelated project 2 mutation (not linked to any project 1 task) leaves source0 current!
    run_mutate(
        &host,
        MutateBoard {
            project: 2,
            op: 0,
            task: 6,
            other: 0,
            text: "done".into(),
        },
        1,
    );
    observe_ready_source(&host, 1, &source0, "unrelated-proj")
        .expect("unlinked project 2 edit leaves project 1 Ready source current");

    // Complete blocker 3 -> task 2 becomes READY, task 3 leaves READY, and source0 stales!
    run_mutate(
        &host,
        MutateBoard {
            project: 1,
            op: 0,
            task: 3,
            other: 0,
            text: "done".into(),
        },
        2,
    );
    assert_stale(observe_ready_source(
        &host,
        1,
        &source0,
        "stale-blocker-done",
    ));
    let (ready1, source1) = ready_with_source(&host, 1);
    assert_eq!(ready1, [1, 2, 4]);

    // Link task 1 (project 1) -> task 6 (project 2, currently done): task 1 stays READY,
    // and now task 6's `Status` is in project 1's Ready footprint even though task 6 is
    // OUTSIDE project 1's root scope!
    run_mutate(
        &host,
        MutateBoard {
            project: 1,
            op: 2,
            task: 1,
            other: 6,
            text: String::new(),
        },
        3,
    );
    assert_stale(observe_ready_source(
        &host,
        1,
        &source1,
        "stale-cross-edge-add",
    ));
    let (ready_cross, source_cross) = ready_with_source(&host, 1);
    assert_eq!(ready_cross, [1, 2, 4]);

    // Reopen out-of-scope blocker 6 (project 2) -> task 1 becomes BLOCKED and source_cross STALES!
    run_mutate(
        &host,
        MutateBoard {
            project: 2,
            op: 0,
            task: 6,
            other: 0,
            text: "open".into(),
        },
        4,
    );
    assert_stale(observe_ready_source(
        &host,
        1,
        &source_cross,
        "stale-out-of-scope-blocker-reopen",
    ));
    let (ready2, source2) = ready_with_source(&host, 1);
    assert_eq!(ready2, [2, 4]);

    // Remove BlockedBy edge 1 -> 6 -> task 1 becomes READY again, source2 stales!
    run_mutate(
        &host,
        MutateBoard {
            project: 1,
            op: 3,
            task: 1,
            other: 6,
            text: String::new(),
        },
        5,
    );
    assert_stale(observe_ready_source(
        &host,
        1,
        &source2,
        "stale-edge-remove",
    ));
    assert_eq!(ready_with_source(&host, 1).0, [1, 2, 4]);

    // Reopen blocker 5 (which blocks task 4) -> task 4 becomes BLOCKED, 5 enters READY!
    run_mutate(
        &host,
        MutateBoard {
            project: 1,
            op: 0,
            task: 5,
            other: 0,
            text: "open".into(),
        },
        6,
    );
    assert_eq!(ready_with_source(&host, 1).0, [1, 2, 5]);

    // Blocked (Exists Forward), Blocking (Exists Reverse), and Unlinked (Absent unfiltered) agree:
    let blocked: Vec<u64> = host
        .runtime()
        .request(&principal, &scope)
        .query(BlockedIntent(ProjectScope { project: 1 }))
        .execute()
        .unwrap()
        .rows()
        .iter()
        .map(|r| r.key)
        .collect();
    assert_eq!(blocked, [4]);

    let blocking: Vec<u64> = host
        .runtime()
        .request(&principal, &scope)
        .query(BlockingIntent(ProjectScope { project: 1 }))
        .execute()
        .unwrap()
        .rows()
        .iter()
        .map(|r| r.key)
        .collect();
    assert_eq!(blocking, [5]);

    // Task 2 still has BlockedBy -> 3 (done), so unlinked open tasks are [1, 5].
    let unlinked: Vec<u64> = host
        .runtime()
        .request(&principal, &scope)
        .query(UnlinkedIntent(ProjectScope { project: 1 }))
        .execute()
        .unwrap()
        .rows()
        .iter()
        .map(|r| r.key)
        .collect();
    assert_eq!(unlinked, [1, 5]);
}

#[test]
fn pinned_read_preserves_historical_related_absence_after_blocker_changes() {
    let host = daymark_board();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let pinned = host
        .runtime()
        .request(&principal, &scope)
        .retain_current_read()
        .expect("retain basis succeeds");

    // Mutate blocker 3 to done and link 1 -> 4.
    run_mutate(
        &host,
        MutateBoard {
            project: 1,
            op: 0,
            task: 3,
            other: 0,
            text: "done".into(),
        },
        1,
    );
    run_mutate(
        &host,
        MutateBoard {
            project: 1,
            op: 2,
            task: 1,
            other: 4,
            text: String::new(),
        },
        2,
    );

    // Current view changed to [2, 4], while pinned view still returns [1, 3, 4].
    assert_eq!(ready_with_source(&host, 1).0, [2, 4]);
    let pinned_rows: Vec<u64> = host
        .runtime()
        .request(&principal, &scope)
        .at(&pinned)
        .query(ReadyIntent(ProjectScope { project: 1 }))
        .execute()
        .expect("pinned read succeeds")
        .rows()
        .iter()
        .map(|r| r.key)
        .collect();
    assert_eq!(pinned_rows, [1, 3, 4]);
}
