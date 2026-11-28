//! Store M4.4 `S5` concurrent `Historical` vs `Head` residency pool isolation
//! and `max_in_flight_fill_bytes` enforcement across independently obtained Home handles.

use std::sync::{Arc, Barrier};
use std::thread;

use worth_execution::ExecutionRequest;
use worth_relational::facade::config::ResidencyBudget;
use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::{HistoricalBasisRetentionKind, HistoricalReadDenial};
use worth_store_application::HistoricalChunkGeometry;

use super::{
    close_durable_home, make_rows, open_durable_home, test_origin, test_query, test_schema,
};

#[test]
fn s5_concurrent_residency_pool_enforces_in_flight_and_working_set_ceilings_across_handles() {
    let request = crate::workload::serial_request();
    let head_ceiling = 32 * 1024;
    let hist_ceiling = 14 * 1024;
    let in_flight_ceiling = 8 * 1024;
    let budget = ResidencyBudget {
        head_working_set_bytes: head_ceiling,
        historical_working_set_bytes: hist_ceiling,
        max_in_flight_fill_bytes: in_flight_ceiling,
        streaming_window_bytes: 8 * 1024,
    };
    let schema = test_schema(1, 0xB1);
    let query = test_query(&schema);
    let origin = test_origin("main", 0xB2);
    let cid = CommitId(601);
    let rows = make_rows(80, "residency", 6, 256);
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 4,
        inline_history_limit: 2,
        continuation_page_versions: 2,
        layered_initial: true,
    };

    let dir = tempfile::tempdir().unwrap();
    let home = open_durable_home(dir.path(), &request);
    let h_main = home.historical_basis_store(budget).unwrap();
    let head_fill = 28 * 1024;
    let _head_guard = h_main
        .pin_head_bytes(head_fill)
        .expect("pin Head working set");
    let (comm, _) = h_main
        .retain_basis(
            cid,
            origin,
            schema.clone(),
            HistoricalBasisRetentionKind::RetentionHorizon,
            rows.clone(),
            geom,
        )
        .unwrap();

    // Four independently obtained handles scan concurrently sharing ONE Home residency pool!
    let barrier = Arc::new(Barrier::new(4));
    let mut joins = Vec::new();
    for _ in 0..4 {
        let handle = home.historical_basis_store(budget).unwrap();
        let b = Arc::clone(&barrier);
        let q = query.clone();
        let sc = schema.clone();
        let expected = rows.clone();
        joins.push(thread::spawn(move || {
            let req = crate::workload::serial_request();
            let exec = ExecutionRequest::serial(&req);
            b.wait();
            let mut cur = handle
                .open_historical_cursor(&exec, cid, comm, q, &sc)
                .unwrap();
            let mut seen = Vec::new();
            loop {
                match cur.next_page(&exec, 5) {
                    Ok(page) => {
                        seen.extend(page.rows().iter().cloned());
                        let snap = handle.residency_snapshot();
                        assert_eq!(snap.head_resident_bytes, head_fill);
                        assert!(
                            snap.historical_resident_bytes + snap.in_flight_fill_bytes
                                <= snap.historical_allowance_bytes,
                            "resident ({}) + in_flight ({}) exceeded allowance ({})",
                            snap.historical_resident_bytes,
                            snap.in_flight_fill_bytes,
                            snap.historical_allowance_bytes
                        );
                        assert!(
                            snap.in_flight_fill_bytes <= in_flight_ceiling,
                            "in_flight ({}) exceeded max_in_flight_fill_bytes ({})",
                            snap.in_flight_fill_bytes,
                            in_flight_ceiling
                        );
                        if !page.has_more() {
                            break;
                        }
                    }
                    Err(HistoricalReadDenial::MemoryExhausted { .. }) => {
                        // Retryable backpressure when concurrent fills saturate in-flight ceiling.
                        thread::yield_now();
                    }
                    Err(other) => panic!("unexpected denial under concurrent scan: {other:?}"),
                }
            }
            assert_eq!(seen, expected);
        }));
    }
    for j in joins {
        j.join().expect("concurrent scan thread");
    }
    let final_snap = h_main.residency_snapshot();
    assert_eq!(final_snap.head_resident_bytes, head_fill);
    assert_eq!(final_snap.in_flight_fill_bytes, 0);
    assert!(final_snap.historical_evictions > 0);
    assert!(final_snap.chunks_faulted >= 20);
    assert!(final_snap.continuation_pages_faulted > 0);
    drop(_head_guard);
    drop(h_main);
    close_durable_home(home, &request);
}

#[test]
fn oversized_single_row_continuation_chain_refuses_before_faulting_continuations() {
    use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
    use worth_relational::facade::identity::{KindId, PartitionId};
    use worth_runtime_world::facade::{CanonicalKeyDomain, HistoricalBasisRow};
    use worth_store_application::MemoryHistoricalBasisStore;

    let request = crate::workload::serial_request();
    let exec = ExecutionRequest::serial(&request);
    let budget = worth_relational::facade::config::ResidencyBudget {
        head_working_set_bytes: 64 * 1024,
        historical_working_set_bytes: 24 * 1024,
        max_in_flight_fill_bytes: 16 * 1024,
        streaming_window_bytes: 12 * 1024,
    };
    let schema = test_schema(1, 0xC2);
    let origin = test_origin("main", 0xC3);
    let query = test_query(&schema);
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 2,
        inline_history_limit: 1,
        continuation_page_versions: 2,
        layered_initial: true,
    };
    let oversized_row = HistoricalBasisRow {
        partition_id: PartitionId(0),
        domain: CanonicalKeyDomain::Entity,
        kind_id: KindId(7),
        logical_key: b"oversized-000001".to_vec(),
        payload: vec![b'p'; 256],
        history_versions: (0..24)
            .map(|i| vec![(i as u8).wrapping_add(1); 1024])
            .collect(),
    };
    let cid = CommitId(9991);

    // 1. Memory backend: refuses MemoryExhausted on first row, faults 0 continuations!
    let mem = MemoryHistoricalBasisStore::new(
        HomeIdentity::from_observed_bytes([0xC8; 16]).unwrap(),
        HomeIncarnation::initial(),
        budget,
    );
    let (m_comm, _) = mem
        .retain_basis(
            cid,
            origin.clone(),
            schema.clone(),
            HistoricalBasisRetentionKind::PinLease,
            vec![oversized_row.clone()],
            geom,
        )
        .unwrap();
    let mut m_cur = mem
        .open_historical_cursor(&exec, cid, m_comm, query.clone(), &schema)
        .unwrap();
    let (m_peak, m_err) =
        super::allocation_court::measure_scoped(|| m_cur.next_page(&exec, 1).unwrap_err());
    assert!(matches!(
        m_err,
        HistoricalReadDenial::MemoryExhausted {
            allowance_bytes: 12_288,
            ..
        }
    ));
    assert_eq!(mem.residency_snapshot().continuation_pages_faulted, 0);
    assert!(
        m_peak < 16 * 1024,
        "preflight refusal peak ({m_peak} B) must stay bounded"
    );
    drop(m_cur);

    // 2. Directory backend (cold reopen): refuses MemoryExhausted on first row, faults 0 continuations!
    let dir = tempfile::tempdir().unwrap();
    let d_comm = {
        let home = open_durable_home(dir.path(), &request);
        let store = home.historical_basis_store(budget).unwrap();
        let (comm, _) = store
            .retain_basis(
                cid,
                origin,
                schema.clone(),
                HistoricalBasisRetentionKind::PinLease,
                vec![oversized_row],
                geom,
            )
            .unwrap();
        drop(store);
        close_durable_home(home, &request);
        comm
    };
    let reopened = open_durable_home(dir.path(), &request);
    let store = reopened.historical_basis_store(budget).unwrap();
    let mut d_cur = store
        .open_historical_cursor(&exec, cid, d_comm, query, &schema)
        .unwrap();
    let d_err = d_cur.next_page(&exec, 1).unwrap_err();
    assert!(matches!(
        d_err,
        HistoricalReadDenial::MemoryExhausted {
            allowance_bytes: 12_288,
            ..
        }
    ));
    assert_eq!(store.residency_snapshot().continuation_pages_faulted, 0);
    drop(d_cur);
    drop(store);
    close_durable_home(reopened, &request);
}
