//! Store M4.4 `S5` two-phase pin + catalog crash-cut recovery (both reopen AND
//! in-process failed-C1/C2 recovery without reopen) and duplicate-retain
//! idempotence/conflict certification courts.

use worth_execution::ExecutionRequest;
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::{HistoricalBasisRetentionKind, HistoricalReadDenial};
use worth_store_application::{
    DurableHistoricalCrashCut, HistoricalChunkGeometry, MemoryHistoricalBasisStore,
};

use super::{
    close_durable_home, drain_cursor, make_rows, open_durable_home, test_budget, test_origin,
    test_query, test_schema,
};

fn retain_pin(
    store: &worth_store_application::DurableHistoricalBasisStore,
    cid: CommitId,
    origin: &worth_runtime_world::facade::BranchOriginIdentity,
    schema: &worth_runtime_world::facade::SchemaVersionIdentity,
    rows: Vec<worth_runtime_world::facade::HistoricalBasisRow>,
    geom: HistoricalChunkGeometry,
) -> Result<([u8; 32], worth_store::pin_table::RootPinId), HistoricalReadDenial> {
    store.retain_basis(
        cid,
        origin.clone(),
        schema.clone(),
        HistoricalBasisRetentionKind::PinLease,
        rows,
        geom,
    )
}

#[test]
fn s5_directory_recovers_cleanly_from_all_six_retain_consolidate_and_prune_crash_cuts() {
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let budget = test_budget(128 * 1024, 64 * 1024);
    let schema = test_schema(1, 0xD1);
    let query = test_query(&schema);
    let origin = test_origin("main", 0xD2);
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 4,
        inline_history_limit: 2,
        continuation_page_versions: 2,
        layered_initial: true,
    };
    let c0 = CommitId(700);
    let c0_rows = make_rows(12, "baseline-c0", 4, 32);

    // 1 & 2. Cuts R1 (AfterRetainStageBeforePin) and R2 (AfterRetainPinBeforeBind):
    // reopen rolls back uncommitted staged entry and expires any orphaned R2 pin!
    for (cut, c1, pins_before_reopen) in [
        (
            DurableHistoricalCrashCut::AfterRetainStageBeforePin,
            CommitId(701),
            1usize,
        ),
        (
            DurableHistoricalCrashCut::AfterRetainPinBeforeBind,
            CommitId(702),
            2usize,
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let c0_comm = {
            let home = open_durable_home(dir.path(), &request);
            let store = home.historical_basis_store(budget).unwrap();
            let (c0_comm, _) =
                retain_pin(&store, c0, &origin, &schema, c0_rows.clone(), geom).unwrap();
            store.arm_crash_cut(Some(cut));
            assert_eq!(
                retain_pin(
                    &store,
                    c1,
                    &origin,
                    &schema,
                    make_rows(12, "cut-r", 4, 32),
                    geom
                )
                .unwrap_err(),
                HistoricalReadDenial::RequestStopped
            );
            assert_eq!(store.live_pin_count(), pins_before_reopen);
            drop(store);
            close_durable_home(home, &request);
            c0_comm
        };
        let reopened = open_durable_home(dir.path(), &request);
        let store = reopened.historical_basis_store(budget).unwrap();
        assert_eq!(store.retained_basis_count(), 1);
        assert_eq!(
            store.live_pin_count(),
            1,
            "reconcile_on_open must leave 1 live pin after {cut:?}"
        );
        assert_eq!(store.pending_retire_pin_count(), 0);
        assert!(store.basis_locator_summary(c1).is_none());
        let mut cur = store
            .open_historical_cursor(&execution, c0, c0_comm, query.clone(), &schema)
            .unwrap();
        assert_eq!(drain_cursor(&mut cur, &execution, 5), c0_rows);
        drop(cur);
        drop(store);
        close_durable_home(reopened, &request);
    }

    // 3. Cuts C1, C2, C3 (Consolidate cuts across reopen): pre-consolidation token MUST resume with 1 live pin!
    for cut in [
        DurableHistoricalCrashCut::AfterConsolidateStageBeforeNewPin,
        DurableHistoricalCrashCut::AfterConsolidateNewPinBeforeBind,
        DurableHistoricalCrashCut::AfterConsolidateBindBeforeOldExpire,
    ] {
        let dir = tempfile::tempdir().unwrap();
        let (p1_rows, wire) = {
            let home = open_durable_home(dir.path(), &request);
            let store = home.historical_basis_store(budget).unwrap();
            let (c0_comm, _) =
                retain_pin(&store, c0, &origin, &schema, c0_rows.clone(), geom).unwrap();
            let mut cur = store
                .open_historical_cursor(&execution, c0, c0_comm, query.clone(), &schema)
                .unwrap();
            let p1 = cur.next_page(&execution, 4).unwrap();
            let wire = p1.checkpoint_token().encode_wire();
            drop(cur);
            store.arm_crash_cut(Some(cut));
            assert_eq!(
                store.consolidate_basis(c0).unwrap_err(),
                HistoricalReadDenial::RequestStopped
            );
            drop(store);
            close_durable_home(home, &request);
            (p1.into_rows(), wire)
        };
        let reopened = open_durable_home(dir.path(), &request);
        let store = reopened.historical_basis_store(budget).unwrap();
        assert_eq!(store.retained_basis_count(), 1, "cut {cut:?}");
        assert_eq!(
            store.live_pin_count(),
            1,
            "reconcile_on_open must leave 1 live pin after {cut:?}"
        );
        assert_eq!(store.pending_retire_pin_count(), 0, "cut {cut:?}");
        let mut resumed = store
            .resume_cursor_wire(&execution, &wire, query.clone(), &schema)
            .unwrap();
        let mut combined = p1_rows;
        combined.extend(drain_cursor(&mut resumed, &execution, 5));
        assert_eq!(combined, c0_rows, "exact rows after {cut:?}");
        drop(resumed);
        drop(store);
        close_durable_home(reopened, &request);
    }

    // 4. In-process failed C1 & C2 recovery WITHOUT reopen: subsequent mutations reconcile automatically!
    for cut in [
        DurableHistoricalCrashCut::AfterConsolidateStageBeforeNewPin,
        DurableHistoricalCrashCut::AfterConsolidateNewPinBeforeBind,
    ] {
        let dir = tempfile::tempdir().unwrap();
        let home = open_durable_home(dir.path(), &request);
        let store = home.historical_basis_store(budget).unwrap();
        let (c0_comm, _) = retain_pin(&store, c0, &origin, &schema, c0_rows.clone(), geom).unwrap();
        let mut cur = store
            .open_historical_cursor(&execution, c0, c0_comm, query.clone(), &schema)
            .unwrap();
        let p1 = cur.next_page(&execution, 4).unwrap();
        let wire = p1.checkpoint_token().encode_wire();
        drop(cur);
        store.arm_crash_cut(Some(cut));
        assert_eq!(
            store.consolidate_basis(c0).unwrap_err(),
            HistoricalReadDenial::RequestStopped
        );

        let c_next = CommitId(799);
        let c_next_rows = make_rows(8, "post-failed-cut", 3, 24);
        let (next_comm, _) =
            retain_pin(&store, c_next, &origin, &schema, c_next_rows.clone(), geom)
                .expect("in-process retain must reconcile failed consolidation cut cleanly");
        assert_eq!(store.retained_basis_count(), 2);
        assert_eq!(
            store.live_pin_count(),
            2,
            "zero orphan pins after in-process reconcile of {cut:?}"
        );
        let (post_comm, _) = store
            .consolidate_basis(c0)
            .expect("retry consolidate in-process");
        assert_eq!(post_comm, c0_comm);
        assert_eq!(store.live_pin_count(), 2);
        let mut resumed = store
            .resume_cursor_wire(&execution, &wire, query.clone(), &schema)
            .unwrap();
        let mut combined = p1.into_rows();
        combined.extend(drain_cursor(&mut resumed, &execution, 4));
        assert_eq!(
            combined, c0_rows,
            "exact C0 rows after in-process {cut:?} recovery"
        );
        drop(resumed);
        let mut cur_next = store
            .open_historical_cursor(&execution, c_next, next_comm, query.clone(), &schema)
            .unwrap();
        assert_eq!(drain_cursor(&mut cur_next, &execution, 4), c_next_rows);
        drop(cur_next);
        drop(store);
        close_durable_home(home, &request);
    }

    // 5. Cut P1 (AfterPruneCatalogBeforePinExpire): reopen expires pending_retire_pins & refuses token!
    {
        let dir = tempfile::tempdir().unwrap();
        let wire = {
            let home = open_durable_home(dir.path(), &request);
            let store = home.historical_basis_store(budget).unwrap();
            let (c0_comm, _) =
                retain_pin(&store, c0, &origin, &schema, c0_rows.clone(), geom).unwrap();
            let mut cur = store
                .open_historical_cursor(&execution, c0, c0_comm, query.clone(), &schema)
                .unwrap();
            let wire = cur
                .next_page(&execution, 3)
                .unwrap()
                .checkpoint_token()
                .encode_wire();
            drop(cur);
            store.arm_crash_cut(Some(
                DurableHistoricalCrashCut::AfterPruneCatalogBeforePinExpire,
            ));
            assert_eq!(
                store.prune_basis(c0).unwrap_err(),
                HistoricalReadDenial::RequestStopped
            );
            assert_eq!(store.live_pin_count(), 1);
            assert_eq!(store.pending_retire_pin_count(), 1);
            drop(store);
            close_durable_home(home, &request);
            wire
        };
        let reopened = open_durable_home(dir.path(), &request);
        let store = reopened.historical_basis_store(budget).unwrap();
        assert_eq!(store.retained_basis_count(), 0);
        assert_eq!(
            store.live_pin_count(),
            0,
            "reconcile_on_open must expire P1 pending-retire pin"
        );
        assert_eq!(store.pending_retire_pin_count(), 0);
        assert_eq!(
            store
                .resume_cursor_wire(&execution, &wire, query, &schema)
                .err(),
            Some(HistoricalReadDenial::HistoryUnavailable { commit_id: c0 })
        );
        drop(store);
        close_durable_home(reopened, &request);
    }
}

#[test]
fn s5_duplicate_retain_is_idempotent_on_exact_match_and_refuses_conflicts() {
    let request = crate::workload::serial_request();
    let budget = test_budget(128 * 1024, 64 * 1024);
    let schema = test_schema(1, 0xD3);
    let origin = test_origin("main", 0xD4);
    let other_origin = test_origin("feature", 0xD9);
    let cid = CommitId(750);
    let rows_a = make_rows(8, "dup-a", 2, 32);
    let rows_b = make_rows(8, "dup-conflict-b", 2, 32);
    let geom = HistoricalChunkGeometry::default();

    // Directory
    let dir = tempfile::tempdir().unwrap();
    let home = open_durable_home(dir.path(), &request);
    let durable = home.historical_basis_store(budget).unwrap();
    let (comm1, pin1) = retain_pin(&durable, cid, &origin, &schema, rows_a.clone(), geom).unwrap();
    let (comm2, pin2) = retain_pin(&durable, cid, &origin, &schema, rows_a.clone(), geom)
        .expect("exact duplicate retain must be idempotent");
    assert_eq!((comm1, pin1), (comm2, pin2));
    assert_eq!(
        durable.live_pin_count(),
        1,
        "idempotent retain leaks 0 pins"
    );
    assert!(matches!(
        retain_pin(&durable, cid, &origin, &schema, rows_b.clone(), geom).unwrap_err(),
        HistoricalReadDenial::CorruptStorage { .. }
    ));
    assert!(matches!(
        durable
            .retain_basis(
                cid,
                origin.clone(),
                schema.clone(),
                HistoricalBasisRetentionKind::ArchivedBranch,
                rows_a.clone(),
                geom
            )
            .unwrap_err(),
        HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }
    ));
    assert!(matches!(
        retain_pin(&durable, cid, &other_origin, &schema, rows_a.clone(), geom).unwrap_err(),
        HistoricalReadDenial::CorruptStorage { .. }
    ));
    assert_eq!(durable.live_pin_count(), 1);
    drop(durable);
    close_durable_home(home, &request);

    // Memory parity
    let mem = MemoryHistoricalBasisStore::new(
        HomeIdentity::from_observed_bytes([0xD5; 16]).unwrap(),
        HomeIncarnation::initial(),
        budget,
    );
    let pin_k = HistoricalBasisRetentionKind::PinLease;
    let arch_k = HistoricalBasisRetentionKind::ArchivedBranch;
    let (m1, mp1) = mem
        .retain_basis(
            cid,
            origin.clone(),
            schema.clone(),
            pin_k,
            rows_a.clone(),
            geom,
        )
        .unwrap();
    let (m2, mp2) = mem
        .retain_basis(
            cid,
            origin.clone(),
            schema.clone(),
            pin_k,
            rows_a.clone(),
            geom,
        )
        .unwrap();
    assert_eq!((m1, mp1), (m2, mp2));
    assert!(matches!(
        mem.retain_basis(cid, origin.clone(), schema.clone(), pin_k, rows_b, geom)
            .unwrap_err(),
        HistoricalReadDenial::CorruptStorage { .. }
    ));
    assert!(matches!(
        mem.retain_basis(cid, origin, schema.clone(), arch_k, rows_a.clone(), geom)
            .unwrap_err(),
        HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }
    ));
    assert!(matches!(
        mem.retain_basis(cid, other_origin, schema, pin_k, rows_a, geom)
            .unwrap_err(),
        HistoricalReadDenial::CorruptStorage { .. }
    ));
}
