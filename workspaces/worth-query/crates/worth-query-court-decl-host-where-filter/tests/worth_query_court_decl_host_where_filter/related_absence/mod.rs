//! End-to-end courts for Daymark Plan Wave 2 — PR 2B (`P2.2` / `G8`):
//! bounded, metered related-record absence/existence predicates (`NOT EXISTS`
//! / `EXISTS`), exact invalidation footprints, pinned/retained reads,
//! metering/budget exhaustion, governance fail-closed checks, and the 64-seed
//! Differential Query DST + Reference Graph Oracle.
mod dst;
mod governance;
mod mutations;
mod queries;
mod schema;
mod world;

use mutations::MutateBoard;
use queries::*;
use world::{
    funded_ready, install, observe_ready_source, ready_with_source, run_mutate, Host, TaskSeed,
};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryApplicationRequestMutationDenial,
    WorthQueryApplicationRequestQueryDenial,
};
use worth_query_host::facade::primary_graph::{
    MutationHandlerExecutionDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationOneShotDenialKind,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

fn task(key: u64, project: u64, status: &str, rank: i64) -> TaskSeed {
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
    //   6 (open): unrelated project task
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
fn daymark_ready_view_tracks_blocker_completion_reopen_and_edges() {
    let host = daymark_board();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);

    // Initial: 1, 3, 4 are ready; 2 is blocked by open task 3.
    let (ready, source0) = ready_with_source(&host, 1);
    assert_eq!(ready, [1, 3, 4]);
    observe_ready_source(&host, 1, &source0, "unchanged").expect("unchanged source stays current");

    // Unrelated project 2 mutation must NOT invalidate project 1's Ready source!
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
        .expect("project 2 edit leaves project 1 Ready source current");

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

    // Reopen blocker 5 (which blocks task 4) -> task 4 becomes BLOCKED, 5 enters READY, source1 stales!
    run_mutate(
        &host,
        MutateBoard {
            project: 1,
            op: 0,
            task: 5,
            other: 0,
            text: "open".into(),
        },
        3,
    );
    assert_stale(observe_ready_source(
        &host,
        1,
        &source1,
        "stale-blocker-reopen",
    ));
    let (ready2, source2) = ready_with_source(&host, 1);
    assert_eq!(ready2, [1, 2, 5]);

    // Add BlockedBy edge 1 -> 5 (open) -> task 1 becomes BLOCKED, source2 stales!
    run_mutate(
        &host,
        MutateBoard {
            project: 1,
            op: 2,
            task: 1,
            other: 5,
            text: String::new(),
        },
        4,
    );
    assert_stale(observe_ready_source(&host, 1, &source2, "stale-edge-add"));
    let (ready3, source3) = ready_with_source(&host, 1);
    assert_eq!(ready3, [2, 5]);

    // Remove BlockedBy edge 4 -> 5 -> task 4 becomes READY again, source3 stales!
    run_mutate(
        &host,
        MutateBoard {
            project: 1,
            op: 3,
            task: 4,
            other: 5,
            text: String::new(),
        },
        5,
    );
    assert_stale(observe_ready_source(
        &host,
        1,
        &source3,
        "stale-edge-remove",
    ));
    assert_eq!(ready_with_source(&host, 1).0, [2, 4, 5]);

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
    assert_eq!(blocked, [1]);

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

    // Task 2 still has a BlockedBy edge to 3 (done), so unlinked open tasks are [4, 5].
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
    assert_eq!(unlinked, [4, 5]);
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

#[test]
fn related_absence_is_metered_for_work_and_memory_exhaustion() {
    let mut tasks = Vec::new();
    let mut edges = Vec::new();
    for key in 1..=20_u64 {
        tasks.push(task(key, 1, "open", key as i64));
    }
    // Closed blockers 11..=20 plus one open blocker 21 (in project 2, so project 1 returns 0 rows)
    // mean every task 1..=10 inspects all closed blockers and stops on open blocker 21.
    for key in 11..=20_u64 {
        tasks[(key - 1) as usize].status = "done".into();
    }
    tasks.push(task(21, 2, "open", 21));
    for from in 1..=10_u64 {
        for to in 11..=21_u64 {
            edges.push((from, to));
        }
    }
    let host = install(&tasks, &edges);
    let source = WorthQueryCancellationSource::new();
    let scope =
        WorthQueryRequestScope::new(None, source.token(), WorthQueryAdmissionClock::system());

    // Generous budget succeeds and reports substantial work.
    let (rows, work_units) = funded_ready(&host, &scope, 64 * 1024 * 1024, 1 << 24, 1)
        .expect("generous budget succeeds");
    assert!(rows.is_empty());
    assert!(
        work_units > 100,
        "related narrowing must charge work: {work_units}"
    );

    // Binary search minimal work budget and confirm (min - 1) refuses with typed execution denial.
    let (mut lo, mut hi) = (1_u64, 1 << 24);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        match funded_ready(&host, &scope, 64 * 1024 * 1024, mid, 1) {
            Ok(_) => hi = mid,
            Err(_) => lo = mid + 1,
        }
    }
    match funded_ready(&host, &scope, 64 * 1024 * 1024, lo - 1, 1) {
        Err(WorthQueryApplicationRequestQueryDenial::Execution(denial)) => {
            assert!(
                matches!(
                    denial.kind(),
                    WorthQueryApplicationOneShotDenialKind::ExecutionRequest(_)
                        | WorthQueryApplicationOneShotDenialKind::WorkLimitExceeded
                ),
                "unexpected denial kind: {denial:?}"
            );
        }
        other => panic!("one work unit short must refuse typed: {other:?}"),
    }

    // Memory exhaustion: with 25 tasks x 35 edges (875 edges), related-predicate
    // edge & memo memory exceeds principal-resolution overhead, so (mem_lo - 1)
    // refuses inside query execution itself.
    let mut wide_tasks = Vec::new();
    let mut wide_edges = Vec::new();
    for key in 1..=25_u64 {
        wide_tasks.push(task(key, 1, "open", key as i64));
    }
    for key in 26..=59_u64 {
        wide_tasks.push(task(key, 1, "done", key as i64));
    }
    wide_tasks.push(task(60, 2, "open", 60));
    for from in 1..=25_u64 {
        for to in 26..=60_u64 {
            wide_edges.push((from, to));
        }
    }
    let wide_host = install(&wide_tasks, &wide_edges);
    let (mut mem_lo, mut mem_hi) = (4096_u64, 64 * 1024 * 1024);
    while mem_lo < mem_hi {
        let mid = mem_lo + (mem_hi - mem_lo) / 2;
        match funded_ready(&wide_host, &scope, mid, 1 << 24, 1) {
            Ok(_) => mem_hi = mid,
            Err(_) => mem_lo = mid + 1,
        }
    }
    assert!(
        mem_lo > 24 * 1024,
        "related narrowing over 875 edges must reserve measurable memory: {mem_lo}"
    );
    match funded_ready(&wide_host, &scope, mem_lo - 1, 1 << 24, 1) {
        Err(WorthQueryApplicationRequestQueryDenial::Execution(_)) => {}
        other => panic!("one byte short of minimal memory must refuse inside execution: {other:?}"),
    }
}
