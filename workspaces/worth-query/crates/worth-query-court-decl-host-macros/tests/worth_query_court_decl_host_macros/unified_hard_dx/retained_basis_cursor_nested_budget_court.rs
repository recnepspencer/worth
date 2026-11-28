//! End-to-end Store M4.4 courts verifying:
//! 1. Distinct admitted string parameter variants (`""` vs `InternedString::Symbol(0)`)
//!    produce distinct `request_binding_digest`s and Store refuses cross-substitution
//!    of their receipts on BOTH page 1 and resume.
//! 2. Batched streaming relation materialization under `streaming_window_bytes` refuses
//!    high-fan-out roots BOTH without a relation predicate AND with a relation predicate
//!    without materializing full adjacency.
//! 3. Derived `#[derive(WorthQueryRow)]` parent rows containing `Vec<ChildRow>` accurately
//!    sum nested child heap bytes via derived `KnownProjectedHeapBytes` and enforce
//!    `streaming_window_bytes` end-to-end.

use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    historical_cursor::{
        CancellationSource, ExecutionRequest, HistoricalBasisRetentionKind, HistoricalReadDenial,
        KnownProjectedHeapBytes, SerialMemoryBudget, SerialRequest,
    },
};

use super::courts::temp_home_path;
use super::program::owner;
use super::retained_basis_cursor_nested_budget_schema::{
    InternedTagIntent, PredicatedOutlineIntent, ProjectTasksOutline,
};
use super::retained_basis_cursor_security_court::{
    budget_with_window, open_custom_daymark, security_schema_version,
};
use super::retained_basis_cursor_window_budget_court::assert_cross_substitution_refused;

#[test]
fn retained_basis_cursor_rejects_empty_text_vs_zero_symbol_parameter_substitution() {
    let path = temp_home_path("retained-basis-text-vs-symbol-param");
    let clock = WorthQueryAdmissionClock::system();
    let schema = security_schema_version();
    let budget = budget_with_window(16 * 1024);
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let exec = ExecutionRequest::serial(&serial);
    let tasks = vec![(100, "A".into()), (101, "B".into()), (102, "C".into())];
    let home = open_custom_daymark(&path, &clock, tasks);
    let me = owner(&home, &clock);
    let session = home.session(&me);
    let receipt = home
        .retain_historical_basis(
            &session,
            "interned-param",
            HistoricalBasisRetentionKind::PinLease,
            schema.clone(),
            budget,
        )
        .unwrap();
    let store = home
        .durable_historical_basis_store(budget)
        .unwrap()
        .unwrap();
    assert_cross_substitution_refused(
        &home,
        &store,
        &session,
        &exec,
        &receipt,
        &schema,
        budget,
        InternedTagIntent {
            project: 2,
            use_zero_symbol: false,
        },
        InternedTagIntent {
            project: 2,
            use_zero_symbol: true,
        },
        1,
    );
    drop((store, session, me));
    home.close_clean().unwrap();
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn retained_basis_cursor_enforces_streaming_window_on_high_fanout_relations_and_nested_rows() {
    let clock = WorthQueryAdmissionClock::system();
    let schema = security_schema_version();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let exec = ExecutionRequest::serial(&serial);

    let path_fanout = temp_home_path("retained-basis-high-fanout-relation");
    let many_tasks: Vec<(u64, String)> = (100..148).map(|k| (k, format!("t-{k}"))).collect();
    let home = open_custom_daymark(&path_fanout, &clock, many_tasks);
    let me = owner(&home, &clock);
    let session = home.session(&me);
    let tight = budget_with_window(256);
    let receipt = home
        .retain_historical_basis(
            &session,
            "high-fanout",
            HistoricalBasisRetentionKind::PinLease,
            schema.clone(),
            tight,
        )
        .unwrap();
    let mut cur_plain = home
        .open_historical_query_cursor(
            &session,
            &exec,
            receipt.commit_id,
            receipt.root_semantic_commitment,
            ProjectTasksOutline { project: 2 },
            &schema,
            tight,
        )
        .unwrap();
    let err_plain = cur_plain.next_page(&exec, 1).unwrap_err();
    assert!(
        matches!(
            err_plain,
            HistoricalReadDenial::MemoryExhausted {
                allowance_bytes: 256,
                ..
            }
        ),
        "{err_plain:?}"
    );
    drop(cur_plain);

    let mut cur_pred = home
        .open_historical_query_cursor(
            &session,
            &exec,
            receipt.commit_id,
            receipt.root_semantic_commitment,
            PredicatedOutlineIntent { project: 2 },
            &schema,
            tight,
        )
        .unwrap();
    let err_pred = cur_pred.next_page(&exec, 1).unwrap_err();
    assert!(
        matches!(
            err_pred,
            HistoricalReadDenial::MemoryExhausted {
                allowance_bytes: 256,
                ..
            }
        ),
        "{err_pred:?}"
    );
    drop((cur_pred, session, me));
    home.close_clean().unwrap();
    let _ = std::fs::remove_dir_all(path_fanout);

    let path_nested = temp_home_path("retained-basis-nested-vec-heap");
    let big_children = vec![
        (100, format!("child-a-{}", "X".repeat(1500))),
        (101, format!("child-b-{}", "Y".repeat(1500))),
    ];
    let home2 = open_custom_daymark(&path_nested, &clock, big_children);
    let me2 = owner(&home2, &clock);
    let session2 = home2.session(&me2);
    let win_1024 = budget_with_window(1024);
    let receipt2 = home2
        .retain_historical_basis(
            &session2,
            "nested-vec",
            HistoricalBasisRetentionKind::PinLease,
            schema.clone(),
            win_1024,
        )
        .unwrap();
    let mut cur_nested = home2
        .open_historical_query_cursor(
            &session2,
            &exec,
            receipt2.commit_id,
            receipt2.root_semantic_commitment,
            ProjectTasksOutline { project: 2 },
            &schema,
            win_1024,
        )
        .unwrap();
    let err_nested = cur_nested.next_page(&exec, 1).unwrap_err();
    assert!(
        matches!(
            err_nested,
            HistoricalReadDenial::MemoryExhausted {
                requested_bytes,
                allowance_bytes: 1024
            } if requested_bytes >= 1500
        ),
        "nested child tree must refuse MemoryExhausted on first oversized child, got {err_nested:?}"
    );
    drop(cur_nested);
    let direct_rows = session2.query(ProjectTasksOutline { project: 2 }).unwrap();
    assert_eq!(direct_rows.len(), 1);
    assert!(
        direct_rows[0].known_heap_bytes() >= 3000,
        "derived KnownProjectedHeapBytes must sum all nested Vec<TaskRow> children"
    );
    drop((session2, me2));
    home2.close_clean().unwrap();
    let _ = std::fs::remove_dir_all(path_nested);
}

#[test]
fn retained_basis_cursor_streamed_adjacency_aborts_on_tight_work_ceiling() {
    use std::num::NonZeroUsize;
    use worth_foundational::facade::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    use worth_query_host::facade::{
        application_entry::WorthQueryApplicationRequestQueryDenial, primary_graph::*,
        WorthQuerySessionQuery,
    };

    let path = temp_home_path("retained-basis-streamed-work-ceiling");
    let clock = WorthQueryAdmissionClock::system();
    let many_tasks: Vec<(u64, String)> = (100..148).map(|k| (k, format!("task-{k}"))).collect();

    let home = open_custom_daymark(&path, &clock, many_tasks);
    let me = owner(&home, &clock);
    let session = home.session(&me);
    let ok = ProjectTasksOutline { project: 2 }
        .execute_window_in_session(
            &session,
            None,
            WorthQueryApplicationRootWindow::first(NonZeroUsize::new(1).unwrap())
                .with_max_projected_bytes(1024 * 1024),
        )
        .expect("generous work ceiling succeeds");
    assert_eq!(ok.into_rows()[0].tasks.len(), 48);

    // Find the exact minimal work ceiling where small project-1 clears
    // principal resolution + root selection + 2-task streamed adjacency.
    let run_with_ceiling = |project: u64, ceiling: u64| {
        let policy = ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, None, Some(ceiling)),
        );
        let bounded = home.session(&me).with_policy(policy);
        ProjectTasksOutline { project }.execute_window_in_session(
            &bounded,
            None,
            WorthQueryApplicationRootWindow::first(NonZeroUsize::new(1).unwrap())
                .with_max_projected_bytes(1024 * 1024),
        )
    };
    let (mut lo, mut hi) = (1u64, 50_000u64);
    assert!(
        run_with_ceiling(1, hi).is_ok(),
        "small project must succeed under 50_000 work units"
    );
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if run_with_ceiling(1, mid).is_ok() {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    let small_min = lo;
    let small = run_with_ceiling(1, small_min)
        .expect("small project clears principal resolution and adjacency at minimal ceiling");
    assert_eq!(small.into_rows().len(), 1);

    // Giving +50 extra work units clears principal resolution + root selection
    // with margin, but MUST abort during streamed adjacency on 48-task project-2
    // (where Prepare + relation batches + O(n log n) sort alone debit > 385 units).
    let err = run_with_ceiling(2, small_min + 50)
        .err()
        .expect("tight work ceiling MUST abort streamed adjacency on prepare/sort charges");
    assert!(
        matches!(
            err,
            WorthQueryApplicationRequestQueryDenial::Execution(_)
                | WorthQueryApplicationRequestQueryDenial::ExecutionRequest(_)
        ),
        "expected work-ceiling refusal during streamed adjacency (not principal resolution), got {err:?}"
    );

    drop((session, me));
    home.close_clean().unwrap();
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn retained_basis_cursor_next_page_enforces_caller_execution_request_work_ceiling() {
    use std::num::NonZeroUsize;
    use worth_foundational::facade::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };

    let path = temp_home_path("retained-basis-caller-work-ceiling");
    let clock = WorthQueryAdmissionClock::system();
    let schema = security_schema_version();
    let budget = budget_with_window(1024 * 1024);
    let many_tasks: Vec<(u64, String)> = (100..148).map(|k| (k, format!("task-{k}"))).collect();

    let home = open_custom_daymark(&path, &clock, many_tasks);
    let me = owner(&home, &clock);
    // Session keeps the default generous work budget (1 << 32).
    let session = home.session(&me);
    let receipt = home
        .retain_historical_basis(
            &session,
            "caller-ceiling",
            HistoricalBasisRetentionKind::PinLease,
            schema.clone(),
            budget,
        )
        .unwrap();
    let open_serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let open_exec = ExecutionRequest::serial(&open_serial);

    let run_next_page_with_ceiling = |project: u64, ceiling: u64| {
        let mut cursor = home
            .open_historical_query_cursor(
                &session,
                &open_exec,
                receipt.commit_id,
                receipt.root_semantic_commitment,
                ProjectTasksOutline { project },
                &schema,
                budget,
            )
            .unwrap();
        let policy = ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, None, Some(ceiling)),
        );
        let tight_serial =
            SerialRequest::from_policy(&policy, CancellationSource::new().token(), None);
        let tight_exec = ExecutionRequest::serial(&tight_serial);
        cursor.next_page(&tight_exec, 1)
    };

    let mut warmup = home
        .open_historical_query_cursor(
            &session,
            &open_exec,
            receipt.commit_id,
            receipt.root_semantic_commitment,
            ProjectTasksOutline { project: 2 },
            &schema,
            budget,
        )
        .unwrap();
    let page = warmup
        .next_page(&open_exec, 1)
        .expect("unbounded next_page succeeds on 48-task project");
    assert_eq!(page.rows()[0].tasks.len(), 48);
    drop(warmup);

    let (mut lo, mut hi) = (1u64, 500_000u64);
    run_next_page_with_ceiling(1, hi)
        .expect("small project cursor next_page must succeed under 500_000 work units");
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if run_next_page_with_ceiling(1, mid).is_ok() {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    let small_min = lo;
    assert_eq!(
        run_next_page_with_ceiling(1, small_min)
            .expect("small project succeeds at minimal caller ceiling")
            .rows()
            .len(),
        1
    );

    // Caller's tight ceiling on `next_page` MUST abort 48-task project-2 even
    // though `session` holds a generous default budget.
    let err = run_next_page_with_ceiling(2, small_min + 50).unwrap_err();
    assert!(
        matches!(err, HistoricalReadDenial::RequestStopped),
        "tight caller ExecutionRequest on next_page must abort cleanly, got {err:?}"
    );

    drop((session, me));
    home.close_clean().unwrap();
    let _ = std::fs::remove_dir_all(path);
}
