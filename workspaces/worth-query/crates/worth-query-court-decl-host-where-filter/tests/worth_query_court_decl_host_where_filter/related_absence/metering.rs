//! Work metering, memory exhaustion, and bounded early-refusal fanout courts
//! for related-record absence/existence predicates.
use super::task;
use super::world::{funded_ready, funded_unlinked, install};
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
    match funded_ready(&host, &scope, 64 * 1024 * 1024, lo - 1, 1).map_err(|e| *e) {
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
    match funded_ready(&wide_host, &scope, mem_lo - 1, 1 << 24, 1).map_err(|e| *e) {
        Err(WorthQueryApplicationRequestQueryDenial::Execution(_)) => {}
        other => panic!("one byte short of minimal memory must refuse inside execution: {other:?}"),
    }
}

fn minimal_unlinked_work_budget(host: &super::world::Host, scope: &WorthQueryRequestScope) -> u64 {
    let (mut lo, mut hi) = (1_u64, 1 << 22);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        match funded_unlinked(host, scope, 64 * 1024 * 1024, mid, 1) {
            Ok(_) => hi = mid,
            Err(_) => lo = mid + 1,
        }
    }
    lo
}

fn minimal_unlinked_memory_budget(
    host: &super::world::Host,
    scope: &WorthQueryRequestScope,
) -> u64 {
    let (mut lo, mut hi) = (4096_u64, 64 * 1024 * 1024);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        match funded_unlinked(host, scope, mid, 1 << 22, 1) {
            Ok(_) => hi = mid,
            Err(_) => lo = mid + 1,
        }
    }
    lo
}

fn assert_blocked_by_refusal(
    label: &str,
    result: Result<(Vec<u64>, usize), Box<WorthQueryApplicationRequestQueryDenial>>,
) {
    match result.map_err(|e| *e) {
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
        other => panic!("{label}: must refuse early inside BlockedBy: {other:?}"),
    }
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
    // out-of-scope tasks in project 2, queried via unfiltered `UnlinkedTasksQuery`
    // (ZERO target field reads, so 100% of fanout work and memory belongs to
    // `BlockedBy` discovery + native collector + sort + endpoint conversion).
    // We prove BOTH:
    //   1. Discovery/early-scan work refusal (`base_budget + 150`),
    //   2. Collector-specific memory refusal: discovery allocates O(1) iterator
    //      bytes and ZERO record clones, so with generous work (`1 << 22`) and
    //      `mid_collector_mem = base_mem + (mem_400 - base_mem) / 3`, discovery
    //      succeeds completely and the collector itself stops early before
    //      materializing/sorting the full fanout!
    //   3. Collector-specific mid-work refusal (`base_budget + delta_400 / 2`,
    //      past discovery's work cost and far below full collector + sort).
    let (_source, scope) = fresh_scope();
    let empty_host = fanout_host(0);
    let base_budget = minimal_unlinked_work_budget(&empty_host, &scope);
    let base_mem = minimal_unlinked_memory_budget(&empty_host, &scope);
    let (empty_rows, base_read_work) =
        funded_unlinked(&empty_host, &scope, 64 * 1024 * 1024, 1 << 22, 1)
            .expect("0-edge baseline succeeds");
    assert_eq!(empty_rows, [1]);

    let host_400 = fanout_host(400);
    let host_800 = fanout_host(800);
    let budget_400 = minimal_unlinked_work_budget(&host_400, &scope);
    let mem_400 = minimal_unlinked_memory_budget(&host_400, &scope);
    let (rows_400, read_work_400) =
        funded_unlinked(&host_400, &scope, 64 * 1024 * 1024, 1 << 22, 1)
            .expect("400-edge fanout succeeds under generous budget");
    assert!(rows_400.is_empty());
    assert!(
        read_work_400 > base_read_work + 400 * 2,
        "400-edge read_work must scale with fanout: base={base_read_work}, full={read_work_400}"
    );
    assert!(
        budget_400 > base_budget + 400 * 3,
        "400-edge total work budget must scale with discovery + collector + sort: base={base_budget}, full={budget_400}"
    );
    assert!(
        mem_400 > base_mem + 32 * 1024,
        "400-edge collector + conversion memory must scale with fanout: base={base_mem}, full={mem_400}"
    );

    let tight_work = base_budget + 150;
    // `budget_400 - base_budget` spans discovery (~800), native collector + sort
    // (~4800), and post-collector edge conversion/sort (~4000). A 1/4 slice
    // (~2500 above base) clears discovery completely and stops inside the native
    // collector pass itself with subject `BlockedBy`.
    let mid_collector_work = base_budget + (budget_400 - base_budget) / 4;
    let mid_collector_mem = base_mem + (mem_400 - base_mem) / 3;
    assert!(tight_work < mid_collector_work && mid_collector_work < budget_400);
    assert!(base_mem < mid_collector_mem && mid_collector_mem < mem_400);

    for (label, host) in [("400-edge", &host_400), ("800-edge", &host_800)] {
        assert_blocked_by_refusal(
            &format!("{label} early-work"),
            funded_unlinked(host, &scope, 64 * 1024 * 1024, tight_work, 1),
        );
        assert_blocked_by_refusal(
            &format!("{label} collector-mid-work"),
            funded_unlinked(host, &scope, 64 * 1024 * 1024, mid_collector_work, 1),
        );
        assert_blocked_by_refusal(
            &format!("{label} collector-memory"),
            funded_unlinked(host, &scope, mid_collector_mem, 1 << 22, 1),
        );
    }
}
