//! End-to-end Decl+Host courts for bounded work/memory refusal and unicode/duplicate keys.
use super::queries::*;
use super::schema::*;
use super::{
    delete_task, home_lease, install, keys, limits, open_directory_home,
    open_directory_home_with_limits, try_create_task, try_reschedule_task,
};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::primary_graph::*;
use worth_query_host::facade::runtime::MapKernelStop;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

fn prefix_q(prefix: &str) -> FindTitlePrefix {
    FindTitlePrefix {
        workspace: "main".into(),
        prefix: prefix.to_owned(),
    }
}

fn focus_q(start: i64, end: i64) -> FindFocusOverlap {
    FindFocusOverlap {
        workspace: "main".into(),
        window: FocusWindow::new(start, end),
    }
}

#[test]
fn bounded_work_ceiling_leaves_ordered_indexes_consistent() {
    // 1. Tight work ceiling: normal queries pass, heavy 50 KiB native watch probes
    //    deterministically hit WorkCeiling, and both indexes remain consistent after refusal.
    let seed = [
        (1, "Alpha", 10, 30),
        (2, "Alpine", 25, 60),
        (3, "Beta", 50, 90),
    ];
    let bounded = install(&seed, None, Some(500_000));
    let scope = request_scope();
    let principal = authenticate_operator(bounded.installed_schema(), &scope);
    let request = bounded.runtime().request(&principal, &scope);

    let title_idx = request
        .query(prefix_q("al"))
        .open_index::<TitlePrefixIndex>()
        .unwrap();
    let focus_idx = request
        .query(focus_q(20, 40))
        .open_index::<FocusIntervalIndex>()
        .unwrap();
    request
        .query(prefix_q("al"))
        .rebuild_index(&title_idx)
        .unwrap();
    request
        .query(focus_q(20, 40))
        .rebuild_index(&focus_idx)
        .unwrap();

    let mut watches = Vec::new();
    let mut hit_work_ceiling = false;
    for ch in ['m', 'n', 'o', 'p', 'q', 'r', 's', 't'] {
        let heavy = prefix_q(&std::iter::repeat_n(ch, 50_000).collect::<String>());
        match request.query(heavy).watch_index::<TitlePrefixIndex>() {
            Ok((_, w)) => watches.push(w),
            Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause))
                if matches!(
                    cause.cause(),
                    WorthQueryManagedIndexDenialCause::Work(
                        WorthQueryManagedIndexWorkDenial::Controlled(
                            WorthQueryManagedIndexStop::ValueDecode(
                                WorthQueryValueDecodeDenial::Checkpoint(MapKernelStop::WorkCeiling)
                            ) | WorthQueryManagedIndexStop::Request(_)
                        )
                    )
                ) =>
            {
                hit_work_ceiling = true;
                break;
            }
            Err(other) => panic!("unexpected denial on heavy watch probe: {other:?}"),
        }
    }
    assert!(
        hit_work_ceiling,
        "50 KiB native probes must hit WorkCeiling"
    );
    assert_eq!(
        keys(request.query(prefix_q("al")).execute().unwrap().rows()),
        [1, 2]
    );
    assert_eq!(
        keys(request.query(focus_q(20, 40)).execute().unwrap().rows()),
        [1, 2]
    );
    for w in watches {
        w.close().unwrap();
    }
    title_idx.close();
    focus_idx.close();
}

#[test]
fn bounded_memory_ceiling_refuses_and_recovers_on_reopen() {
    let seed = [
        (1, "Alpha", 10, 30),
        (2, "Alpine", 25, 60),
        (3, "Beta", 50, 90),
    ];
    let location = std::env::temp_dir().join(format!(
        "worth-ordered-index-mem-ceiling-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let lease = home_lease();
    let execution = ExecutionRequest::leased(&lease);
    let initial = open_directory_home(&location, &execution, true, &seed);
    initial.close(&execution).unwrap().reclaim();

    let tight_mem = open_directory_home_with_limits(
        &location,
        &execution,
        false,
        &[],
        limits(Some(16 * 1024), None),
    );
    let scope = request_scope();
    let principal = authenticate_operator(tight_mem.runtime().installed_schema(), &scope);
    let request = tight_mem.runtime().runtime().request(&principal, &scope);
    let mem_title = request
        .query(prefix_q("al"))
        .open_index::<TitlePrefixIndex>();
    let mem_focus = request
        .query(focus_q(20, 40))
        .open_index::<FocusIntervalIndex>();
    for (label, outcome) in [
        ("title", mem_title.map(|idx| idx.close())),
        ("focus", mem_focus.map(|idx| idx.close())),
    ] {
        match outcome {
            Err(err) => {
                let detail = format!("{err:?}");
                assert!(
                    detail.contains("MemoryLimit") || detail.contains("PolicyMemoryExhausted"),
                    "expected memory-limit denial for {label} under 16 KiB ceiling, got {detail}"
                );
            }
            Ok(()) => panic!("{label} open_index must refuse under 16 KiB memory ceiling"),
        }
    }
    tight_mem.close(&execution).unwrap().reclaim();

    let recovered = open_directory_home(&location, &execution, false, &[]);
    let scope = request_scope();
    let principal = authenticate_operator(recovered.runtime().installed_schema(), &scope);
    let request = recovered.runtime().runtime().request(&principal, &scope);
    let ok_title = request
        .query(prefix_q("al"))
        .open_index::<TitlePrefixIndex>()
        .unwrap();
    let ok_focus = request
        .query(focus_q(20, 40))
        .open_index::<FocusIntervalIndex>()
        .unwrap();
    request
        .query(prefix_q("al"))
        .rebuild_index(&ok_title)
        .unwrap();
    request
        .query(focus_q(20, 40))
        .rebuild_index(&ok_focus)
        .unwrap();
    assert_eq!(
        keys(request.query(prefix_q("al")).execute().unwrap().rows()),
        [1, 2]
    );
    assert_eq!(
        keys(request.query(focus_q(20, 40)).execute().unwrap().rows()),
        [1, 2]
    );
    ok_title.close();
    ok_focus.close();
    recovered.close(&execution).unwrap().reclaim();
    std::fs::remove_dir_all(location).unwrap();
}

#[test]
fn unicode_mixed_case_and_duplicate_keys_preserve_exact_prefix_and_interval_results() {
    let seed = [
        (1, "élan", 10, 20),
        (2, "Éclair", 10, 20),
        (3, "élan", 10, 20),
        (4, "😀plan", 15, 30),
        (5, "😀pulse", 25, 40),
        (6, "Zebra", 5, 100),
    ];
    let host = install(&seed, None, None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);

    let title_idx = request
        .query(prefix_q(""))
        .open_index::<TitlePrefixIndex>()
        .unwrap();
    let focus_idx = request
        .query(focus_q(0, 10))
        .open_index::<FocusIntervalIndex>()
        .unwrap();
    request
        .query(prefix_q(""))
        .rebuild_index(&title_idx)
        .unwrap();
    request
        .query(focus_q(0, 10))
        .rebuild_index(&focus_idx)
        .unwrap();

    assert_eq!(
        keys(request.query(prefix_q("é")).execute().unwrap().rows()),
        [1, 3]
    );
    assert_eq!(
        keys(request.query(prefix_q("É")).execute().unwrap().rows()),
        [2]
    );
    assert_eq!(
        keys(request.query(prefix_q("😀p")).execute().unwrap().rows()),
        [4, 5]
    );
    assert_eq!(
        keys(request.query(prefix_q("z")).execute().unwrap().rows()),
        [6]
    );
    assert_eq!(
        keys(request.query(focus_q(10, 20)).execute().unwrap().rows()),
        [1, 2, 3, 4, 6]
    );
    assert_eq!(
        keys(request.query(focus_q(50, 60)).execute().unwrap().rows()),
        [6]
    );

    delete_task(&host, 1, 200);
    assert_eq!(
        keys(request.query(prefix_q("é")).execute().unwrap().rows()),
        [3]
    );
    assert_eq!(
        keys(request.query(focus_q(10, 20)).execute().unwrap().rows()),
        [2, 3, 4, 6]
    );

    title_idx.close();
    focus_idx.close();
}

#[test]
fn empty_and_reversed_intervals_are_refused_on_write_and_never_match_focus_index() {
    // 1. CreateTask and RescheduleTask refuse empty [15, 15) and reversed [20, 10) spans
    //    with InputMismatch, leaving rows and both checked indexes untouched.
    let host = install(&[(1, "Alpha", 10, 30)], None, None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let title_idx = request
        .query(prefix_q("al"))
        .open_index::<TitlePrefixIndex>()
        .unwrap();
    let focus_idx = request
        .query(focus_q(10, 20))
        .open_index::<FocusIntervalIndex>()
        .unwrap();
    request
        .query(prefix_q("al"))
        .rebuild_index(&title_idx)
        .unwrap();
    request
        .query(focus_q(10, 20))
        .rebuild_index(&focus_idx)
        .unwrap();

    for (idem, start, end) in [(1, 15, 15), (2, 20, 10)] {
        let create = try_create_task(&host, 2, "BadSpan", start, end, idem).unwrap();
        assert!(
            matches!(
                create,
                WorthQueryApplicationMutationOutcome::DomainDenied(
                    WorthQueryOperationBodyDenial::InputMismatch { .. }
                )
            ),
            "CreateTask([{start}, {end})) must refuse with InputMismatch, got {create:?}"
        );
        let resched = try_reschedule_task(&host, 1, start, end, idem + 10).unwrap();
        assert!(
            matches!(
                resched,
                WorthQueryApplicationMutationOutcome::DomainDenied(
                    WorthQueryOperationBodyDenial::InputMismatch { .. }
                )
            ),
            "RescheduleTask([{start}, {end})) must refuse with InputMismatch, got {resched:?}"
        );
    }
    assert_eq!(
        keys(request.query(prefix_q("")).execute().unwrap().rows()),
        [1]
    );
    assert_eq!(
        keys(request.query(focus_q(10, 20)).execute().unwrap().rows()),
        [1]
    );
    title_idx.close();
    focus_idx.close();

    // 2. Non-match invariant: even if malformed empty/reversed intervals exist in Source,
    //    FocusIntervalDomain never admits [15, 15) or [20, 10) as hits for [10, 20).
    let malformed = install(
        &[
            (1, "Empty", 15, 15),
            (2, "Reversed", 20, 10),
            (3, "Valid", 12, 18),
        ],
        None,
        None,
    );
    let scope = request_scope();
    let principal = authenticate_operator(malformed.installed_schema(), &scope);
    let request = malformed.runtime().request(&principal, &scope);
    let idx = request
        .query(focus_q(10, 20))
        .open_index::<FocusIntervalIndex>()
        .unwrap();
    request.query(focus_q(10, 20)).rebuild_index(&idx).unwrap();
    assert_eq!(
        keys(request.query(focus_q(10, 20)).execute().unwrap().rows()),
        [3]
    );
    assert!(request
        .query(focus_q(14, 16))
        .execute()
        .unwrap()
        .rows()
        .iter()
        .all(|r| r.key == 3));
    idx.close();
}
