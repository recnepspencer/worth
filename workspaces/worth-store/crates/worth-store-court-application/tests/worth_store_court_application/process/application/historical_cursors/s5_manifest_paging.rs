//! Store M4.4 `S5` paged manifest & out-of-line continuation reference courts:
//! 1. A chunk with many continuation references scanned by a `KeysOnly` cursor
//!    under a tight `HistoricalWorkingSetPool` budget succeeds after cold reopen
//!    without loading a single continuation reference or continuation page, while
//!    a `Full` cursor resolves all continuation pages accurately.
//! 2. Valid 1,800-byte keys sharing a long common prefix across many chunks
//!    converge to a finite bounded-height `WHBM3` index tree and support exact
//!    keyset seek/scan and token resume after cold reopen.

use worth_execution::ExecutionRequest;
use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_runtime_world::facade::{
    CanonicalKeyDomain, HistoricalBasisRetentionKind, HistoricalBasisRow,
    PROJECTION_CANONICAL_FULL, PROJECTION_CANONICAL_HEAD_ONLY, PROJECTION_CANONICAL_KEYS_ONLY,
};
use worth_store_application::HistoricalChunkGeometry;

use super::{
    close_durable_home, drain_cursor, open_durable_home, test_budget, test_origin,
    test_query_with_projection, test_schema,
};

#[test]
fn s5_keys_only_and_head_only_cursors_never_load_out_of_line_continuation_manifests() {
    let dir = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let schema = test_schema(1, 0xC1);
    let origin = test_origin("main", 0xC2);
    let commit_id = CommitId(401);

    // 2 rows in 1 chunk; row 0 has 90 history versions -> 89 spilled continuation
    // pages (well over 3 KiB of raw continuation refs if stored inline!).
    let mut rows = Vec::new();
    for i in 0..2usize {
        let versions = if i == 0 {
            (0..90usize)
                .map(|v| format!("cont-version-{v:04}").into_bytes())
                .collect()
        } else {
            vec![b"v0".to_vec()]
        };
        rows.push(HistoricalBasisRow {
            partition_id: PartitionId(0),
            domain: CanonicalKeyDomain::Entity,
            kind_id: KindId(7),
            logical_key: format!("many-cont-key-{i:02}").into_bytes(),
            payload: format!("payload-{i:02}").into_bytes(),
            history_versions: versions,
        });
    }
    let expected_full = rows.clone();
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 8,
        inline_history_limit: 1,
        continuation_page_versions: 1,
        layered_initial: true,
    };

    let root_commitment = {
        let home = open_durable_home(dir.path(), &request);
        let store = home
            .historical_basis_store(test_budget(128 * 1024, 64 * 1024))
            .unwrap();
        let (root, _) = store
            .retain_basis(
                commit_id,
                origin,
                schema.clone(),
                HistoricalBasisRetentionKind::PinLease,
                rows,
                geom,
            )
            .expect("retain chunk with 89 continuation pages");
        drop(store);
        close_durable_home(home, &request);
        root
    };

    // Cold reopen under a tight 8 KiB historical pool (far smaller than all 89
    // continuation pages + continuation manifests combined): KeysOnly and HeadOnly
    // MUST scan cleanly with ZERO continuation page loads!
    {
        let home = open_durable_home(dir.path(), &request);
        let tight = test_budget(64 * 1024, 8 * 1024);
        let store = home.historical_basis_store(tight).unwrap();

        let keys_q = test_query_with_projection(&schema, PROJECTION_CANONICAL_KEYS_ONLY);
        let mut keys_cur = store
            .open_historical_cursor(&execution, commit_id, root_commitment, keys_q, &schema)
            .expect("open KeysOnly cursor under tight pool");
        let keys_rows = drain_cursor(&mut keys_cur, &execution, 4);
        drop(keys_cur);
        assert_eq!(keys_rows.len(), 2);
        assert!(keys_rows
            .iter()
            .all(|r| r.payload.is_empty() && r.history_versions.is_empty()));
        assert_eq!(
            store.residency_snapshot().continuation_pages_faulted,
            0,
            "KeysOnly cursor must fault zero continuation pages"
        );

        let head_q = test_query_with_projection(&schema, PROJECTION_CANONICAL_HEAD_ONLY);
        let mut head_cur = store
            .open_historical_cursor(&execution, commit_id, root_commitment, head_q, &schema)
            .expect("open HeadOnly cursor under tight pool");
        let head_rows = drain_cursor(&mut head_cur, &execution, 4);
        drop(head_cur);
        assert_eq!(head_rows.len(), 2);
        assert_eq!(head_rows[0].payload, b"payload-00");
        assert!(head_rows.iter().all(|r| r.history_versions.is_empty()));
        assert_eq!(
            store.residency_snapshot().continuation_pages_faulted,
            0,
            "HeadOnly cursor must fault zero continuation pages"
        );

        // Full cursor resolves all 89 out-of-line continuation pages cleanly via WHCM1!
        let full_q = test_query_with_projection(&schema, PROJECTION_CANONICAL_FULL);
        let mut full_cur = store
            .open_historical_cursor(&execution, commit_id, root_commitment, full_q, &schema)
            .expect("open Full cursor");
        let full_rows = drain_cursor(&mut full_cur, &execution, 4);
        drop(full_cur);
        assert_eq!(full_rows, expected_full);
        assert_eq!(store.residency_snapshot().continuation_pages_faulted, 89);

        drop(store);
        close_durable_home(home, &request);
    }
}

#[test]
fn s5_large_1800_byte_keys_across_many_chunks_converge_and_resume_after_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let schema = test_schema(1, 0xD1);
    let origin = test_origin("main", 0xD2);
    let commit_id = CommitId(402);

    // 48 chunks (1 row per chunk) where EVERY logical_key is 1,800 bytes and
    // shares a 400-byte common prefix (exercising truncated 256-byte WHBM3 index
    // separators + multi-level bounded index convergence!).
    let common_prefix = "P".repeat(400);
    let suffix_pad = "Z".repeat(1390);
    let rows: Vec<HistoricalBasisRow> = (0..48usize)
        .map(|i| {
            let logical_key = format!("{common_prefix}-{i:04}-{suffix_pad}").into_bytes();
            assert_eq!(logical_key.len(), 1796);
            HistoricalBasisRow {
                partition_id: PartitionId(0),
                domain: CanonicalKeyDomain::Entity,
                kind_id: KindId(7),
                logical_key,
                payload: format!("large-key-row-{i:04}").into_bytes(),
                history_versions: vec![format!("h-{i:04}").into_bytes()],
            }
        })
        .collect();
    let expected = rows.clone();
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 1,
        inline_history_limit: 2,
        continuation_page_versions: 2,
        layered_initial: false,
    };

    let (root_commitment, first_half, wire_token) = {
        let home = open_durable_home(dir.path(), &request);
        let store = home
            .historical_basis_store(test_budget(128 * 1024, 64 * 1024))
            .unwrap();
        let (root, _) = store
            .retain_basis(
                commit_id,
                origin,
                schema.clone(),
                HistoricalBasisRetentionKind::PinLease,
                rows,
                geom,
            )
            .expect("retain 48 chunks of ~1800-byte keys with bounded WHBM3 index");
        let query = test_query_with_projection(&schema, PROJECTION_CANONICAL_FULL);
        let mut cursor = store
            .open_historical_cursor(&execution, commit_id, root, query, &schema)
            .unwrap();
        let mut first_half = Vec::new();
        let mut tok = None;
        for _ in 0..4 {
            let page = cursor.next_page(&execution, 5).expect("page");
            tok = Some(page.checkpoint_token().encode_wire());
            first_half.extend(page.into_rows());
        }
        assert_eq!(first_half.len(), 20);
        drop(cursor);
        drop(store);
        close_durable_home(home, &request);
        (root, first_half, tok.unwrap())
    };

    // Cold reopen: resume from row 20 through row 48 using bounded WHBM3 seek!
    {
        let home = open_durable_home(dir.path(), &request);
        let store = home
            .historical_basis_store(test_budget(128 * 1024, 32 * 1024))
            .unwrap();
        let query = test_query_with_projection(&schema, PROJECTION_CANONICAL_FULL);
        let mut resumed = store
            .resume_cursor_wire(&execution, &wire_token, query.clone(), &schema)
            .expect("resume across 1800-byte key manifest tree after cold reopen");
        let mut combined = first_half;
        combined.extend(drain_cursor(&mut resumed, &execution, 5));
        drop(resumed);
        assert_eq!(combined, expected);

        // Full fresh scan from Start after cold reopen must also match identically.
        let mut fresh = store
            .open_historical_cursor(
                &execution,
                commit_id,
                root_commitment,
                query.clone(),
                &schema,
            )
            .unwrap();
        assert_eq!(drain_cursor(&mut fresh, &execution, 7), expected);
        drop(fresh);

        // Capture a late checkpoint token at chunk 45 (index 44) to test cold
        // O(log chunks) late resume over 1,796-byte shared-prefix keys!
        let mut advance = store
            .open_historical_cursor(&execution, commit_id, root_commitment, query, &schema)
            .unwrap();
        let mut late_wire = None;
        for _ in 0..45 {
            let p = advance.next_page(&execution, 1).unwrap();
            late_wire = Some(p.checkpoint_token().encode_wire());
        }
        drop(advance);
        drop(store);
        close_durable_home(home, &request);

        // Fresh cold reopen: resume at chunk 46 (after row 45) and step 1-row
        // pages! Manifest page faults MUST be strictly O(log chunks), far less
        // than linear 48 pages!
        let home_late = open_durable_home(dir.path(), &request);
        let store_late = home_late
            .historical_basis_store(test_budget(128 * 1024, 32 * 1024))
            .unwrap();
        let query_late = test_query_with_projection(&schema, PROJECTION_CANONICAL_FULL);
        let mut late_cursor = store_late
            .resume_cursor_wire(&execution, &late_wire.unwrap(), query_late, &schema)
            .expect("resume at chunk 46 over shared-prefix keys");
        let faults_before = store_late.manifest_pages_faulted();
        let p46 = late_cursor.next_page(&execution, 1).unwrap();
        let faults_after_seek = store_late.manifest_pages_faulted() - faults_before;
        assert_eq!(p46.rows().len(), 1);
        assert_eq!(p46.rows()[0], expected[45]);
        assert!(
            faults_after_seek <= 16,
            "cold late seek to chunk 46 over 48 shared-prefix chunks must read O(log N) manifest pages, faulted {faults_after_seek}"
        );
        let p47 = late_cursor.next_page(&execution, 1).unwrap();
        let p48 = late_cursor.next_page(&execution, 1).unwrap();
        assert_eq!(p47.rows(), &[expected[46].clone()]);
        assert_eq!(p48.rows(), &[expected[47].clone()]);
        assert!(!p48.has_more());
        let total_late_faults = store_late.manifest_pages_faulted() - faults_before;
        assert!(
            total_late_faults <= 20,
            "stepping 1-row pages across chunks 46..48 must stay O(log N), faulted {total_late_faults}"
        );
        drop(late_cursor);
        drop(store_late);
        close_durable_home(home_late, &request);
    }
}

#[test]
fn s5_batched_continuation_publication_scales_with_tree_height_not_chunk_count() {
    let dir = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let schema = test_schema(1, 0xE1);
    let origin = test_origin("main", 0xE2);
    let commit_id = CommitId(403);

    // 32 chunks (1 row per chunk), EVERY chunk has 2 spilled continuation pages.
    let rows: Vec<HistoricalBasisRow> = (0..32usize)
        .map(|i| HistoricalBasisRow {
            partition_id: PartitionId(0),
            domain: CanonicalKeyDomain::Entity,
            kind_id: KindId(7),
            logical_key: format!("batched-cont-key-{i:04}").into_bytes(),
            payload: format!("payload-{i:04}").into_bytes(),
            history_versions: vec![
                format!("v0-{i:04}").into_bytes(),
                format!("v1-{i:04}").into_bytes(),
                format!("v2-{i:04}").into_bytes(),
            ],
        })
        .collect();
    let expected = rows.clone();
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 1,
        inline_history_limit: 1,
        continuation_page_versions: 1,
        layered_initial: true,
    };

    let root = {
        let home = open_durable_home(dir.path(), &request);
        let store = home
            .historical_basis_store(test_budget(128 * 1024, 64 * 1024))
            .unwrap();
        let pubs_before = store.basis_stage_publications();
        let (root, _) = store
            .retain_basis(
                commit_id,
                origin,
                schema.clone(),
                HistoricalBasisRetentionKind::PinLease,
                rows,
                geom,
            )
            .expect("retain 32 multi-continuation chunks with batched publication");
        let pubs_delta = store.basis_stage_publications() - pubs_before;
        // 1 data stage + 1 batched WHCM1 level + <=3 WHBM4 levels = <= 5 total (NOT 32+!).
        assert!(
            pubs_delta <= 6,
            "batched continuation publication across 32 chunks must scale with tree height (<= 6), did {pubs_delta}"
        );
        drop(store);
        close_durable_home(home, &request);
        root
    };

    // Cold reopen and verify every chunk's full history resolves identically.
    let home = open_durable_home(dir.path(), &request);
    let store = home
        .historical_basis_store(test_budget(128 * 1024, 32 * 1024))
        .unwrap();
    let query = test_query_with_projection(&schema, PROJECTION_CANONICAL_FULL);
    let mut cursor = store
        .open_historical_cursor(&execution, commit_id, root, query, &schema)
        .unwrap();
    assert_eq!(drain_cursor(&mut cursor, &execution, 5), expected);
    assert_eq!(store.residency_snapshot().continuation_pages_faulted, 64);
    drop(cursor);
    drop(store);
    close_durable_home(home, &request);
}
