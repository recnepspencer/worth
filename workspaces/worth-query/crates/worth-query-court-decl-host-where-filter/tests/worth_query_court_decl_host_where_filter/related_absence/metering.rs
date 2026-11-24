//! Work metering, memory exhaustion, and bounded early-refusal fanout courts
//! for related-record absence/existence predicates.
use super::task;
use super::world::{funded_ready, install};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial;
use worth_query_host::facade::primary_graph::WorthQueryApplicationOneShotDenialKind;

fn fresh_scope() -> (WorthQueryCancellationSource, WorthQueryRequestScope) {
    let source = WorthQueryCancellationSource::new();
    let scope =
        WorthQueryRequestScope::new(None, source.token(), WorthQueryAdmissionClock::system());
    (source, scope)
}

#[test]
fn related_absence_is_metered_for_work_and_memory_exhaustion() {
    let mut tasks = Vec::new();
    let mut edges = Vec::new();
    for key in 1..=20_u64 {
        tasks.push(task(key, 1, "open", key as i64));
    }
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
    let (_source, scope) = fresh_scope();

    let (rows, work_units) = funded_ready(&host, &scope, 64 * 1024 * 1024, 1 << 24, 1)
        .expect("generous budget succeeds");
    assert!(rows.is_empty());
    assert!(
        work_units > 100,
        "related narrowing must charge work: {work_units}"
    );

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

fn minimal_work_budget(host: &super::world::Host, scope: &WorthQueryRequestScope) -> u64 {
    let (mut lo, mut hi) = (1_u64, 1 << 22);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        match funded_ready(host, scope, 64 * 1024 * 1024, mid, 1) {
            Ok(_) => hi = mid,
            Err(_) => lo = mid + 1,
        }
    }
    lo
}

fn fanout_host(fanout: u64) -> super::world::Host {
    let mut tasks = Vec::with_capacity((fanout + 1) as usize);
    let mut edges = Vec::with_capacity(fanout as usize);
    tasks.push(task(1, 1, "open", 1));
    for key in 2..=(fanout + 1) {
        tasks.push(task(key, 2, "done", key as i64));
        edges.push((1, key));
    }
    install(&tasks, &edges)
}

#[test]
fn high_fanout_single_root_stops_native_frontier_scan_early_when_budget_exhausts() {
    // 1 open root task in project 1 with 400 (and 800) `BlockedBy` edges to
    // out-of-scope closed tasks in project 2. A tight budget `base + 150`
    // (enough for principal + scope + root selection, far below scanning all
    // 400 or 800 edges) must refuse INSIDE native `BlockedBy` frontier
    // traversal before reading, materializing, or sorting the full adjacency!
    let (_source, scope) = fresh_scope();
    let empty_host = fanout_host(0);
    let base_budget = minimal_work_budget(&empty_host, &scope);
    let (_, base_read_work) = funded_ready(&empty_host, &scope, 64 * 1024 * 1024, 1 << 22, 1)
        .expect("0-edge baseline succeeds");

    let host_400 = fanout_host(400);
    let budget_400 = minimal_work_budget(&host_400, &scope);
    let (rows_400, read_work_400) = funded_ready(&host_400, &scope, 64 * 1024 * 1024, 1 << 22, 1)
        .expect("400-edge fanout succeeds under generous budget");
    assert_eq!(rows_400, [1]);
    assert!(
        read_work_400 > base_read_work + 400 * 2,
        "400-edge read_work must scale with fanout: base={base_read_work}, full={read_work_400}"
    );
    assert!(
        budget_400 > base_budget + 400 * 3,
        "400-edge total execution work budget must reflect demand + scan + materialize + sort: base={base_budget}, full={budget_400}"
    );

    // At `tight_budget = base_budget + 150` (far below scanning 400 or 800 edges),
    // native frontier traversal must stop early and refuse with subject `BlockedBy`
    // before touching the remaining fanout!
    let tight_budget = base_budget + 150;
    assert!(tight_budget < budget_400);
    for (label, host) in [("400-edge", &host_400), ("800-edge", &fanout_host(800))] {
        match funded_ready(host, &scope, 64 * 1024 * 1024, tight_budget, 1) {
            Err(WorthQueryApplicationRequestQueryDenial::Execution(denial)) => {
                assert!(
                    matches!(
                        denial.kind(),
                        WorthQueryApplicationOneShotDenialKind::ExecutionRequest(_)
                            | WorthQueryApplicationOneShotDenialKind::WorkLimitExceeded
                    ),
                    "{label}: unexpected denial kind: {denial:?}"
                );
                assert_eq!(
                    denial.subject(),
                    "BlockedBy",
                    "{label}: early refusal must occur during BlockedBy frontier traversal: {denial:?}"
                );
            }
            other => panic!(
                "{label}: tight budget ({tight_budget} vs base {base_budget}) must refuse early: {other:?}"
            ),
        }
    }
}
