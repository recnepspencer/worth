//! End-to-end Store M4.4 branch retirement custody & streaming-window byte-budget
//! courts (Findings 4 & 7) on persistent Directory `WorthQueryApplicationHome`.

use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    historical_cursor::{
        CancellationSource, ExecutionRequest, HistoricalBasisRetentionKind, HistoricalReadDenial,
        SerialMemoryBudget, SerialRequest,
    },
};

use super::courts::temp_home_path;
use super::program::owner;
use super::queries::BacklogTasks;
use super::retained_basis_cursor_security_court::{
    budget_with_window, open_custom_daymark, security_schema_version,
};

fn active_branch_name(
    session: &worth_query_host::facade::WorthQueryHostSession<
        '_,
        super::schema::DaymarkSchema,
        super::program::DaymarkProgram,
    >,
) -> String {
    session
        .snapshot()
        .unwrap()
        .branch_identity()
        .name()
        .as_str()
        .to_owned()
}

#[test]
fn retained_basis_cursor_recovers_planned_branch_intent_and_retains_refused_closure_retry() {
    let path_a = temp_home_path("retained-basis-branch-recovery-a");
    let path_b = temp_home_path("retained-basis-branch-recovery-b");
    let path_c = temp_home_path("retained-basis-branch-recovery-c");
    let clock = WorthQueryAdmissionClock::system();
    let budget = budget_with_window(16 * 1024);

    // Part A: crash-recoverable planned branch intent retires orphaned branch across reopen.
    let orphaned_branch_name = {
        let home = open_custom_daymark(&path_a, &clock, vec![(100, "Alpha".into())]);
        let me = owner(&home, &clock);
        let session = home.session(&me);
        let store = home
            .durable_historical_basis_store(budget)
            .unwrap()
            .unwrap();
        let preview = home.fork_preview("simulated-crash-preview").unwrap();
        let name = active_branch_name(&session.clone().on_branch(preview.branch()));
        store.record_inflight_retain_intent(&name).unwrap();
        assert!(store.inflight_retain_branches().contains(&name));
        assert!(!store.pending_retire_branches().contains(&name));
        home.reconcile_historical_branch_retirements(&session, budget);
        assert!(store.inflight_retain_branches().contains(&name));
        drop(preview);
        drop((store, session, me));
        home.close_clean().unwrap();
        name
    };
    // Cold reopen: Home startup MUST automatically retire the crash-leftover
    // orphan branch even on a read-only reopened Home (ZERO retain/reconcile calls!).
    {
        let home = open_custom_daymark(&path_a, &clock, vec![(100, "Alpha".into())]);
        let me = owner(&home, &clock);
        let session = home.session(&me);
        assert_eq!(session.query(BacklogTasks { project: 2 }).unwrap().len(), 1);
        let store = home
            .durable_historical_basis_store(budget)
            .unwrap()
            .unwrap();
        assert!(!store
            .inflight_retain_branches()
            .contains(&orphaned_branch_name));
        assert!(
            !store.pending_retire_branches().contains(&orphaned_branch_name),
            "cold Home reopen must automatically close and clear orphaned branch without retain/reconcile"
        );
        let scope = session.fresh_scope();
        let still_exists = home
            .primary_graph()
            .with_funded_application_completion(
                &scope,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                |phase| {
                    home.primary_graph()
                        .branches()
                        .inventory(&phase)
                        .map(|inv| inv.find_by_name(&orphaned_branch_name).is_some())
                },
            )
            .unwrap()
            .unwrap();
        assert!(!still_exists, "cold Home reopen must close orphaned branch");
        drop((store, session, me));
        home.close_clean().unwrap();
    }

    // Part B: refused close_branch (ActiveBranchProtected) during BOTH cold reopen
    // AND live reconcile MUST preserve durable retry state until unblocked!
    let protected_active_name = {
        let home = open_custom_daymark(&path_b, &clock, vec![(100, "Alpha".into())]);
        let me = owner(&home, &clock);
        let session = home.session(&me);
        let store = home
            .durable_historical_basis_store(budget)
            .unwrap()
            .unwrap();
        let active_name = active_branch_name(&session);
        store.record_pending_branch_intent(&active_name).unwrap();
        home.reconcile_historical_branch_retirements(&session, budget);
        assert!(store.pending_retire_branches().contains(&active_name));
        drop((store, session, me));
        home.close_clean().unwrap();
        active_name
    };
    {
        let home = open_custom_daymark(&path_b, &clock, vec![(100, "Alpha".into())]);
        let store = home
            .durable_historical_basis_store(budget)
            .unwrap()
            .unwrap();
        assert!(
            store
                .pending_retire_branches()
                .contains(&protected_active_name),
            "refused cold-open close_branch must preserve durable pending_retire_branches"
        );
        drop(store);
        home.close_clean().unwrap();
    }
    {
        let home = open_custom_daymark(&path_c, &clock, vec![(100, "Alpha".into())]);
        let me = owner(&home, &clock);
        let session = home.session(&me);
        let store = home
            .durable_historical_basis_store(budget)
            .unwrap()
            .unwrap();
        let active_name = active_branch_name(&session);
        store.record_pending_branch_intent(&active_name).unwrap();
        home.reconcile_historical_branch_retirements(&session, budget);
        assert!(store.pending_retire_branches().contains(&active_name));
        let replacement = home.fork_preview("adopt-to-unblock").unwrap();
        let old_active = home.adopt_preview(replacement).unwrap();
        home.reconcile_historical_branch_retirements(&session, budget);
        assert!(!store.pending_retire_branches().contains(&active_name));
        assert!(home.close_branch(old_active).is_err());
        drop((store, session, me));
        home.close_clean().unwrap();
    }
    let _ = std::fs::remove_dir_all(path_a);
    let _ = std::fs::remove_dir_all(path_b);
    let _ = std::fs::remove_dir_all(path_c);
}

fn serial_req() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    )
}

#[test]
fn retained_basis_cursor_enforces_streaming_window_bytes_on_projected_heap_rows() {
    let path = temp_home_path("retained-basis-query-byte-budget");
    let clock = WorthQueryAdmissionClock::system();
    let schema = security_schema_version();
    let serial = serial_req();
    let exec = ExecutionRequest::serial(&serial);
    let tasks: Vec<(u64, String)> = (100..104)
        .map(|k| (k, format!("Row-{k}-{}", "P".repeat(1500))))
        .collect();

    let receipt = {
        let home = open_custom_daymark(&path, &clock, tasks.clone());
        let me = owner(&home, &clock);
        let session = home.session(&me);
        let receipt = home
            .retain_historical_basis(
                &session,
                "large-rows",
                HistoricalBasisRetentionKind::PinLease,
                schema.clone(),
                budget_with_window(64 * 1024),
            )
            .unwrap();
        drop((session, me));
        home.close_clean().unwrap();
        receipt
    };

    // Reopen with streaming_window_bytes = 512 (> size_of::<TaskRow>(), < 1500B projected row).
    {
        let tight = budget_with_window(512);
        let home = open_custom_daymark(&path, &clock, tasks.clone());
        let me = owner(&home, &clock);
        let session = home.session(&me);
        let mut cursor = home
            .open_historical_query_cursor(
                &session,
                &exec,
                receipt.commit_id,
                receipt.root_semantic_commitment,
                BacklogTasks { project: 2 },
                &schema,
                tight,
            )
            .unwrap();
        let err = cursor.next_page(&exec, 4).unwrap_err();
        assert!(
            matches!(
                err,
                HistoricalReadDenial::MemoryExhausted {
                    requested_bytes,
                    allowance_bytes: 512
                } if requested_bytes > 512
            ),
            "expected MemoryExhausted on oversized first projected row, got {err:?}"
        );
        drop((cursor, session, me));
        home.close_clean().unwrap();
    }

    // Reopen with streaming_window_bytes = 3000 (fits 1 projected row ~2408B, cannot fit 2).
    {
        let one_row = budget_with_window(3000);
        let home = open_custom_daymark(&path, &clock, tasks);
        let me = owner(&home, &clock);
        let session = home.session(&me);
        let mut cursor = home
            .open_historical_query_cursor(
                &session,
                &exec,
                receipt.commit_id,
                receipt.root_semantic_commitment,
                BacklogTasks { project: 2 },
                &schema,
                one_row,
            )
            .unwrap();
        let p1 = cursor.next_page(&exec, 4).unwrap();
        assert_eq!(
            p1.rows().len(),
            1,
            "page must stop at 1 row under 3000B ceiling"
        );
        assert!(p1.has_more());
        let mut keys: Vec<u64> = p1.into_rows().into_iter().map(|r| r.task_key).collect();
        loop {
            let page = cursor.next_page(&exec, 4).unwrap();
            assert_eq!(page.rows().len(), 1);
            let more = page.has_more();
            keys.extend(page.into_rows().into_iter().map(|r| r.task_key));
            if !more {
                break;
            }
        }
        assert_eq!(keys, vec![100, 101, 102, 103]);
        drop((cursor, session, me));
        home.close_clean().unwrap();
    }
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn retained_basis_cursor_fences_active_retain_against_concurrent_reconciliation() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let path = temp_home_path("retained-basis-concurrent-retain-reconcile");
    let clock = WorthQueryAdmissionClock::system();
    let schema = security_schema_version();
    let budget = budget_with_window(16 * 1024);
    let serial = serial_req();
    let exec = ExecutionRequest::serial(&serial);
    let tasks: Vec<(u64, String)> = (100..104).map(|k| (k, format!("Task-{k}"))).collect();

    let home = Arc::new(open_custom_daymark(&path, &clock, tasks));
    let me = Arc::new(owner(&home, &clock));
    let store = home
        .durable_historical_basis_store(budget)
        .unwrap()
        .unwrap();
    store
        .record_inflight_retain_intent("planned-before-fork")
        .unwrap();
    let session = home.session(&*me);
    home.reconcile_historical_branch_retirements(&session, budget);
    assert!(store
        .inflight_retain_branches()
        .iter()
        .any(|b| b == "planned-before-fork"));
    store
        .abandon_inflight_retain_intent("planned-before-fork")
        .unwrap();
    store.mark_branch_retired("planned-before-fork").unwrap();

    let preview = home.fork_preview("phase1-fence").unwrap();
    let branch_name = session
        .clone()
        .on_branch(preview.branch())
        .snapshot()
        .unwrap()
        .branch_identity()
        .name()
        .as_str()
        .to_owned();
    store.record_inflight_retain_intent(&branch_name).unwrap();
    home.reconcile_historical_branch_retirements(&session, budget);
    assert!(session
        .clone()
        .on_branch(preview.branch())
        .snapshot()
        .is_ok());
    assert!(store.inflight_retain_branches().contains(&branch_name));
    store.abandon_inflight_retain_intent(&branch_name).unwrap();
    preview.discard(&home).unwrap();
    store.mark_branch_retired(&branch_name).unwrap();

    // Phase 2: Concurrent background thread hammering `reconcile_historical_branch_retirements`
    // while foreground commits + calls `retain_historical_basis` repeatedly.
    let stop = Arc::new(AtomicBool::new(false));
    let bg_home = Arc::clone(&home);
    let bg_me = Arc::clone(&me);
    let bg_stop = Arc::clone(&stop);
    let reconciler = std::thread::spawn(move || {
        let bg_session = bg_home.session(bg_me);
        let mut passes = 0usize;
        while !bg_stop.load(Ordering::Acquire) || passes < 8 {
            bg_home.reconcile_historical_branch_retirements(&bg_session, budget);
            passes += 1;
            std::thread::yield_now();
        }
        passes
    });

    let mut receipts = Vec::new();
    for step in 0..4u64 {
        session
            .run(super::actions::SetTaskState {
                task: 100 + step,
                state: "DONE".to_owned(),
            })
            .unwrap();
        let receipt = home
            .retain_historical_basis(
                &session,
                format!("concurrent-retain-{step}"),
                HistoricalBasisRetentionKind::PinLease,
                schema.clone(),
                budget,
            )
            .expect("retain_historical_basis succeeds under concurrent reconciliation");
        receipts.push(receipt);
    }
    stop.store(true, Ordering::Release);
    let passes = reconciler.join().expect("reconciler thread completes");
    assert!(passes >= 8);

    // Every retained basis MUST remain open and readable (none reaped by reconciler!).
    for receipt in &receipts {
        let mut cur = home
            .open_historical_query_cursor(
                &session,
                &exec,
                receipt.commit_id,
                receipt.root_semantic_commitment,
                BacklogTasks { project: 2 },
                &schema,
                budget,
            )
            .expect("retained basis remains live after concurrent reconciliation");
        let page = cur.next_page(&exec, 4).expect("retained cursor reads page");
        assert_eq!(page.rows().len(), 4);
        drop(cur);
    }

    drop((store, session, me));
    Arc::try_unwrap(home)
        .ok()
        .expect("sole home owner")
        .close_clean()
        .unwrap();
    let _ = std::fs::remove_dir_all(path);
}
