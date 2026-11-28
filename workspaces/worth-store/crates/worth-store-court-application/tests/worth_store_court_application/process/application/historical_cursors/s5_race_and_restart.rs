//! Store M4.4 `S5` multi-handle atomic resume-vs-prune & consolidation races
//! (using independently obtained Home handles) and restart/no-HEAD-rebind courts
//! on BOTH Directory and Memory backends.

use std::sync::{Arc, Barrier};
use std::thread;

use worth_execution::report::{Disposition, Reportable};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::{HistoricalBasisRetentionKind, HistoricalReadDenial};
use worth_store::pin_table::RootPinState;
use worth_store_application::{HistoricalChunkGeometry, MemoryHistoricalBasisStore};

use super::{
    close_durable_home, drain_cursor, make_rows, open_durable_home, test_budget, test_origin,
    test_query, test_schema,
};

#[test]
fn s5_multi_handle_atomic_resume_vs_prune_and_consolidation_races() {
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let budget = test_budget(128 * 1024, 64 * 1024);
    let schema = test_schema(1, 0x61);
    let query = test_query(&schema);
    let origin = test_origin("main", 0x71);
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 4,
        inline_history_limit: 2,
        continuation_page_versions: 3,
        layered_initial: true,
    };

    // 1. Directory backend barrier race using INDEPENDENTLY obtained handles from the SAME Home!
    let dir = tempfile::tempdir().unwrap();
    let home = open_durable_home(dir.path(), &request);
    let handle_retain = home.historical_basis_store(budget).expect("handle A");
    let handle_resume = home.historical_basis_store(budget).expect("handle B");
    let handle_prune = home.historical_basis_store(budget).expect("handle C");
    let (mut d_loan_won, mut d_prune_won) = (0usize, 0usize);
    for iter in 0..16u64 {
        let cid = CommitId(1000 + iter);
        let rows = make_rows(16, &format!("dir-race-{iter}"), 5, 32);
        let (comm, pin_id) = handle_retain
            .retain_basis(
                cid,
                origin.clone(),
                schema.clone(),
                HistoricalBasisRetentionKind::PinLease,
                rows.clone(),
                geom,
            )
            .expect("retain via handle A");
        // Handle B sees the basis retained by handle A immediately (ONE shared Home catalog).
        let mut init = handle_resume
            .open_historical_cursor(&execution, cid, comm, query.clone(), &schema)
            .expect("open via independent handle B");
        let token = init
            .next_page(&execution, 4)
            .expect("p1")
            .checkpoint_token()
            .clone();
        drop(init);

        let barrier = Arc::new(Barrier::new(2));
        let (s_res, s_pru) = (handle_resume.clone(), handle_prune.clone());
        let (b_res, b_pru) = (Arc::clone(&barrier), Arc::clone(&barrier));
        let (q_res, sc_res, tok_res) = (query.clone(), schema.clone(), token.clone());
        let expected_tail = rows[4..].to_vec();

        let h1 = thread::spawn(move || {
            let req = crate::workload::serial_request();
            let exec = ExecutionRequest::serial(&req);
            b_res.wait();
            match s_res.resume_cursor(&exec, &tok_res, q_res, &sc_res) {
                Ok(mut cur) => {
                    // Yield so prune_basis completes and overwrites anchor while loan is live!
                    // Cursor MUST keep reading via its pin-loan-derived reader (`pin_loan.records()`).
                    thread::yield_now();
                    let tail = drain_cursor(&mut cur, &exec, 3);
                    assert_eq!(
                        tail, expected_tail,
                        "loan-won read must complete from pinned root even after concurrent prune"
                    );
                    true
                }
                Err(HistoricalReadDenial::HistoryUnavailable { commit_id }) => {
                    assert_eq!(commit_id, cid);
                    false
                }
                Err(other) => panic!("unlawful race outcome on Directory: {other:?}"),
            }
        });
        let h2 = thread::spawn(move || {
            b_pru.wait();
            let state = s_pru
                .prune_basis(cid)
                .expect("prune via independent handle C");
            assert!(matches!(
                state,
                Some(RootPinState::Expiring | RootPinState::Expired)
            ));
        });

        if h1.join().expect("resume thread") {
            d_loan_won += 1;
        } else {
            d_prune_won += 1;
        }
        h2.join().expect("prune thread");
        let _ = handle_retain.settle_drained_pins().expect("settle");
        assert_eq!(
            handle_retain.raw_pin_state(pin_id),
            Some(RootPinState::Expired)
        );
        assert_eq!(
            handle_resume
                .resume_cursor(&execution, &token, query.clone(), &schema)
                .err(),
            Some(HistoricalReadDenial::HistoryUnavailable { commit_id: cid })
        );
    }
    assert_eq!(d_loan_won + d_prune_won, 16);

    // 2. Multi-handle live cursor across concurrent consolidation on independent handles:
    //    Cursor opened BEFORE consolidate_basis continues reading from its pinned pre-consolidation
    //    root while handle C consolidates and expires the old pin; once dropped, old pin drains!
    let cid_cons = CommitId(1500);
    let rows_cons = make_rows(24, "dir-cons-race", 6, 40);
    let (comm_cons, old_pin) = handle_retain
        .retain_basis(
            cid_cons,
            origin.clone(),
            schema.clone(),
            HistoricalBasisRetentionKind::PinLease,
            rows_cons.clone(),
            geom,
        )
        .expect("retain for consolidation race");
    let mut pre_cur = handle_resume
        .open_historical_cursor(&execution, cid_cons, comm_cons, query.clone(), &schema)
        .expect("open pre-consolidation cursor");
    let p1 = pre_cur
        .next_page(&execution, 6)
        .expect("p1 before consolidate");
    let pre_wire = p1.checkpoint_token().encode_wire();

    // Consolidate on independent handle C while `pre_cur` holds a live ReaderLoan on `old_pin`!
    let (post_comm, new_pin) = handle_prune
        .consolidate_basis(cid_cons)
        .expect("consolidate while cursor live");
    assert_eq!(post_comm, comm_cons);
    assert_ne!(new_pin, old_pin);
    assert_eq!(
        handle_retain.raw_pin_state(old_pin),
        Some(RootPinState::Expiring),
        "old pin must stay Expiring while pre-consolidation ReaderLoan is live"
    );
    // Live pre-consolidation cursor finishes draining cleanly from its pinned root!
    let mut combined = p1.into_rows();
    combined.extend(drain_cursor(&mut pre_cur, &execution, 5));
    assert_eq!(combined, rows_cons);
    drop(pre_cur);
    let _ = handle_retain.settle_drained_pins().expect("settle old pin");
    assert_eq!(
        handle_retain.raw_pin_state(old_pin),
        Some(RootPinState::Expired)
    );
    // Pre-consolidation wire token resumes cleanly on post-consolidation root!
    let mut post_cur = handle_resume
        .resume_cursor_wire(&execution, &pre_wire, query.clone(), &schema)
        .expect("resume wire token on post-consolidation root");
    assert_eq!(drain_cursor(&mut post_cur, &execution, 7), rows_cons[6..]);
    drop(post_cur);
    drop(handle_retain);
    drop(handle_resume);
    drop(handle_prune);
    close_durable_home(home, &request);

    // 3. Memory backend barrier race (32 iterations) across cloned handles
    let home_id = HomeIdentity::from_observed_bytes([0x44; 16]).unwrap();
    let mem = MemoryHistoricalBasisStore::new(home_id, HomeIncarnation::initial(), budget);
    let (mut m_loan_won, mut m_prune_won) = (0usize, 0usize);
    for iter in 0..32u64 {
        let cid = CommitId(2000 + iter);
        let rows = make_rows(16, &format!("mem-race-{iter}"), 5, 32);
        let (comm, pin_id) = mem
            .retain_basis(
                cid,
                origin.clone(),
                schema.clone(),
                HistoricalBasisRetentionKind::RetentionHorizon,
                rows.clone(),
                geom,
            )
            .expect("retain");
        let mut init = mem
            .open_historical_cursor(&execution, cid, comm, query.clone(), &schema)
            .expect("open");
        let token = init
            .next_page(&execution, 4)
            .expect("p1")
            .checkpoint_token()
            .clone();
        drop(init);

        let barrier = Arc::new(Barrier::new(2));
        let (s_res, s_pru) = (mem.clone(), mem.clone());
        let (b_res, b_pru) = (Arc::clone(&barrier), Arc::clone(&barrier));
        let (q_res, sc_res, tok_res) = (query.clone(), schema.clone(), token.clone());
        let expected_tail = rows[4..].to_vec();

        let h1 = thread::spawn(move || {
            let req = crate::workload::serial_request();
            let exec = ExecutionRequest::serial(&req);
            b_res.wait();
            match s_res.resume_cursor(&exec, &tok_res, q_res, &sc_res) {
                Ok(mut cur) => {
                    thread::yield_now();
                    let tail = drain_cursor(&mut cur, &exec, 3);
                    assert_eq!(tail, expected_tail);
                    true
                }
                Err(HistoricalReadDenial::HistoryUnavailable { commit_id }) => {
                    assert_eq!(commit_id, cid);
                    false
                }
                Err(other) => panic!("unlawful race outcome on Memory: {other:?}"),
            }
        });
        let h2 = thread::spawn(move || {
            b_pru.wait();
            let _ = s_pru.prune_basis(cid).expect("prune");
        });

        if h1.join().unwrap() {
            m_loan_won += 1;
        } else {
            m_prune_won += 1;
        }
        h2.join().unwrap();
        assert_eq!(mem.raw_pin_state(pin_id), Some(RootPinState::Expired));
    }
    assert_eq!(m_loan_won + m_prune_won, 32);
}

#[test]
fn s5_restart_after_prune_refuses_history_unavailable_and_never_rebinds_to_head() {
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let budget = test_budget(128 * 1024, 64 * 1024);
    let schema = test_schema(1, 0x81);
    let query = test_query(&schema);
    let origin = test_origin("main", 0x82);
    let (c1, c2) = (CommitId(301), CommitId(302));
    let c1_rows = make_rows(12, "pruned-c1", 2, 32);
    let c2_rows = make_rows(12, "live-head-c2", 2, 32);
    let geom = HistoricalChunkGeometry::default();

    let dir = tempfile::tempdir().unwrap();
    let wire_dir = {
        let home = open_durable_home(dir.path(), &request);
        let store = home.historical_basis_store(budget).unwrap();
        let (c1_comm, _) = store
            .retain_basis(
                c1,
                origin.clone(),
                schema.clone(),
                HistoricalBasisRetentionKind::PinLease,
                c1_rows.clone(),
                geom,
            )
            .unwrap();
        store
            .retain_basis(
                c2,
                origin.clone(),
                schema.clone(),
                HistoricalBasisRetentionKind::LiveBranch,
                c2_rows.clone(),
                geom,
            )
            .unwrap();
        let mut cur = store
            .open_historical_cursor(&execution, c1, c1_comm, query.clone(), &schema)
            .unwrap();
        let wire = cur
            .next_page(&execution, 3)
            .unwrap()
            .checkpoint_token()
            .encode_wire();
        drop(cur);
        store.prune_basis(c1).unwrap();
        drop(store);
        close_durable_home(home, &request);
        wire
    };
    let reopened = open_durable_home(dir.path(), &request);
    let store2 = reopened.historical_basis_store(budget).unwrap();
    let err = store2
        .resume_cursor_wire(&execution, &wire_dir, query.clone(), &schema)
        .err()
        .expect("pruned C1 must refuse after reopen");
    assert_eq!(
        err,
        HistoricalReadDenial::HistoryUnavailable { commit_id: c1 }
    );
    assert_eq!(err.disposition(), Disposition::Final);
    drop(store2);
    close_durable_home(reopened, &request);

    let home_id = HomeIdentity::from_observed_bytes([0x83; 16]).unwrap();
    let mem = MemoryHistoricalBasisStore::new(home_id, HomeIncarnation::initial(), budget);
    let (c1_comm, _) = mem
        .retain_basis(
            c1,
            origin.clone(),
            schema.clone(),
            HistoricalBasisRetentionKind::PinLease,
            c1_rows,
            geom,
        )
        .unwrap();
    mem.retain_basis(
        c2,
        origin,
        schema.clone(),
        HistoricalBasisRetentionKind::LiveBranch,
        c2_rows,
        geom,
    )
    .unwrap();
    let mut cur = mem
        .open_historical_cursor(&execution, c1, c1_comm, query.clone(), &schema)
        .unwrap();
    let wire_mem = cur
        .next_page(&execution, 3)
        .unwrap()
        .checkpoint_token()
        .encode_wire();
    drop(cur);
    mem.prune_basis(c1).unwrap();
    let mem2 = mem.simulate_restart(budget);
    let err = mem2
        .resume_cursor_wire(&execution, &wire_mem, query, &schema)
        .err()
        .expect("pruned C1 must refuse after restart");
    assert_eq!(
        err,
        HistoricalReadDenial::HistoryUnavailable { commit_id: c1 }
    );
    assert_eq!(err.disposition(), Disposition::Final);
}
