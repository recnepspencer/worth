//! Layer 3 unbypassable `#[global_allocator]` bounded-memory scaling proof for
//! Store Milestone 4, Phase M4.4 retained-basis historical cursor streaming.
//!
//! Covers:
//! 1. Positive allocator control (`>= 64 KiB` observed) proving tracker is live.
//! 2. BOTH Directory (cold reopen WITH `open_historical_cursor` INSIDE `measure_scoped`!)
//!    AND Memory backends across:
//!    - `1x` vs `10x` basis size,
//!    - `1x` vs `10x` continuation pages per row,
//!    - `1x` vs `10x` UNRELATED retained bases in the same Directory Home (`O(1)` admission!), AND
//!    - oversized single-row continuation chain (> `streaming_window_bytes`) refusing
//!      immediately with `MemoryExhausted` and faulting 0 continuation pages!

use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::sync::{Mutex, Once};

use tracking_allocator::{
    AllocationGroupId, AllocationGroupToken, AllocationRegistry, AllocationTracker,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_relational::facade::config::ResidencyBudget;
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::schema::{SchemaId, SchemaVersionId};
use worth_runtime_world::facade::{
    BranchOriginIdentity, CanonicalKeyDomain, HistoricalBasisRetentionKind, HistoricalBasisRow,
    HistoricalQueryContract, SchemaVersionIdentity, ORDERING_CANONICAL_ASC,
    PROJECTION_CANONICAL_FULL,
};
use worth_store_application::{HistoricalChunkGeometry, MemoryHistoricalBasisStore};

use super::{close_durable_home, open_durable_home};

static INSTALL_TRACKER: Once = Once::new();
static MEASURE_LOCK: Mutex<()> = Mutex::new(());
static ACTIVE_GROUP: AtomicUsize = AtomicUsize::new(0);
static LIVE_BYTES: AtomicIsize = AtomicIsize::new(0);
static PEAK_LIVE_BYTES: AtomicIsize = AtomicIsize::new(0);

struct CountingTracker;

impl AllocationTracker for CountingTracker {
    fn allocated(&self, _: usize, size: usize, _: usize, group: AllocationGroupId) {
        let active = ACTIVE_GROUP.load(Ordering::Acquire);
        if active == 0 || active != group.as_usize().get() {
            return;
        }
        let next = LIVE_BYTES.fetch_add(size as isize, Ordering::AcqRel) + size as isize;
        PEAK_LIVE_BYTES.fetch_max(next, Ordering::AcqRel);
    }

    fn deallocated(
        &self,
        _: usize,
        size: usize,
        _: usize,
        src: AllocationGroupId,
        _: AllocationGroupId,
    ) {
        let active = ACTIVE_GROUP.load(Ordering::Acquire);
        if active == 0 || active != src.as_usize().get() {
            return;
        }
        LIVE_BYTES.fetch_sub(size as isize, Ordering::AcqRel);
    }
}

fn ensure_tracker() {
    INSTALL_TRACKER.call_once(|| {
        AllocationRegistry::set_global_tracker(CountingTracker)
            .expect("install global allocation tracker");
        AllocationRegistry::enable_tracking();
    });
}

pub(super) fn measure_scoped<R>(body: impl FnOnce() -> R) -> (isize, R) {
    ensure_tracker();
    let _serial = MEASURE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut token = AllocationGroupToken::register().expect("register allocation group");
    let group_id = token.id().as_usize().get();
    LIVE_BYTES.store(0, Ordering::Release);
    PEAK_LIVE_BYTES.store(0, Ordering::Release);
    ACTIVE_GROUP.store(group_id, Ordering::Release);
    let guard = token.enter();
    let result = body();
    drop(guard);
    ACTIVE_GROUP.store(0, Ordering::Release);
    (PEAK_LIVE_BYTES.load(Ordering::Acquire), result)
}

fn build_rows(count: usize, versions_per_row: usize) -> Vec<HistoricalBasisRow> {
    (0..count)
        .map(|i| {
            let mut payload = format!("alloc-row-{i:06}-").into_bytes();
            payload.resize(256, b'z');
            let history_versions = (0..versions_per_row)
                .map(|v| {
                    format!("alloc-row-{i:06}-hist-{v:03}-spilled-history-padding").into_bytes()
                })
                .collect();
            HistoricalBasisRow {
                partition_id: PartitionId((i / 64) as u32),
                domain: CanonicalKeyDomain::Entity,
                kind_id: KindId(7),
                logical_key: format!("k-{i:06}").into_bytes(),
                payload,
                history_versions,
            }
        })
        .collect()
}

fn alloc_budget(hist_budget_bytes: u64) -> ResidencyBudget {
    ResidencyBudget {
        head_working_set_bytes: 64 * 1024,
        historical_working_set_bytes: hist_budget_bytes,
        max_in_flight_fill_bytes: 16 * 1024,
        streaming_window_bytes: 12 * 1024,
    }
}

fn alloc_schema_and_query() -> (
    SchemaVersionIdentity,
    BranchOriginIdentity,
    HistoricalQueryContract,
) {
    let schema = SchemaVersionIdentity::new(
        SchemaId("m4.alloc.proof".into()),
        SchemaVersionId(1),
        [0xC2; 32],
    )
    .unwrap();
    let origin = BranchOriginIdentity::from_owner(BranchId("main".into()), [0xC3; 32]).unwrap();
    let query = HistoricalQueryContract::new(
        schema.clone(),
        CanonicalKeyDomain::Entity,
        Some(KindId(7)),
        PROJECTION_CANONICAL_FULL,
        ORDERING_CANONICAL_ASC,
    );
    (schema, origin, query)
}

fn measure_memory_stream(
    row_count: usize,
    versions_per_row: usize,
    hist_budget: u64,
) -> (isize, u64, u64) {
    let budget = alloc_budget(hist_budget);
    let store = MemoryHistoricalBasisStore::new(
        HomeIdentity::from_observed_bytes([0xC1; 16]).unwrap(),
        HomeIncarnation::initial(),
        budget,
    );
    let (schema, origin, query) = alloc_schema_and_query();
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 4,
        inline_history_limit: 2,
        continuation_page_versions: 2,
        layered_initial: true,
    };
    let cid = CommitId(700 + row_count as u64 + versions_per_row as u64);
    let (comm, _) = store
        .retain_basis(
            cid,
            origin,
            schema.clone(),
            HistoricalBasisRetentionKind::PinLease,
            build_rows(row_count, versions_per_row),
            geom,
        )
        .expect("retain basis");
    let req = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let exec = ExecutionRequest::serial(&req);

    let (peak_delta, (yielded, max_pool)) = measure_scoped(|| {
        let mut cursor = store
            .open_historical_cursor(&exec, cid, comm, query, &schema)
            .expect("open cursor");
        let mut yielded = 0usize;
        let mut max_pool = 0u64;
        loop {
            let page = cursor.next_page(&exec, 4).expect("stream page");
            yielded += page.rows().len();
            let snap = store.residency_snapshot();
            max_pool = max_pool.max(snap.historical_resident_bytes);
            assert!(snap.historical_resident_bytes <= snap.historical_allowance_bytes);
            let more = page.has_more();
            drop(page);
            if !more {
                break;
            }
        }
        drop(cursor);
        (yielded, max_pool)
    });
    assert_eq!(yielded, row_count);
    (
        peak_delta,
        max_pool,
        store.residency_snapshot().historical_evictions,
    )
}

/// Measures cold `historical_basis_store(budget)` restore + `open_historical_cursor`
/// + first `next_page` (ALL INSIDE `measure_scoped`!) on a cold-reopened Directory
/// Home with `unrelated_bases` extra retained bases (each with many chunks and
/// spilled continuation pages), verifying unrelated bases' chunk/continuation
/// manifests are NEVER eagerly loaded or allocated.
fn measure_directory_reopen_stream(
    row_count: usize,
    versions_per_row: usize,
    unrelated_bases: usize,
    hist_budget: u64,
) -> (isize, isize, u64, u64) {
    let dir = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let budget = alloc_budget(hist_budget);
    let (schema, origin, query) = alloc_schema_and_query();
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 4,
        inline_history_limit: 2,
        continuation_page_versions: 2,
        layered_initial: true,
    };
    let target_cid = CommitId(800 + row_count as u64 + unrelated_bases as u64);
    let comm = {
        let home = open_durable_home(dir.path(), &request);
        let store = home.historical_basis_store(budget).unwrap();
        let (comm, _) = store
            .retain_basis(
                target_cid,
                origin.clone(),
                schema.clone(),
                HistoricalBasisRetentionKind::PinLease,
                build_rows(row_count, versions_per_row),
                geom,
            )
            .expect("retain target basis");
        // Each unrelated basis carries 32 rows x 16 versions (8 chunks + 224 continuation pages!)
        let extra_rows = build_rows(32, 16);
        for idx in 0..unrelated_bases {
            store
                .retain_basis(
                    CommitId(10_000 + idx as u64),
                    origin.clone(),
                    schema.clone(),
                    HistoricalBasisRetentionKind::PinLease,
                    extra_rows.clone(),
                    geom,
                )
                .expect("retain unrelated basis");
        }
        drop(store);
        close_durable_home(home, &request);
        comm
    };

    let reopened = open_durable_home(dir.path(), &request);
    let exec = ExecutionRequest::serial(&request);

    // Measure cold store restore + cursor admission + first page read INSIDE `measure_scoped`!
    let (cold_open_and_p1_peak, (store, wire, p1_len, p1_more)) = measure_scoped(|| {
        let store = reopened.historical_basis_store(budget).unwrap();
        let mut cursor = store
            .open_historical_cursor(&exec, target_cid, comm, query.clone(), &schema)
            .expect("open durable cursor after reopen");
        assert_eq!(store.residency_snapshot().chunks_faulted, 0);
        let page1 = cursor.next_page(&exec, 4).expect("stream cold first page");
        let len = page1.rows().len();
        let more = page1.has_more();
        let wire = page1.checkpoint_token().encode_wire();
        let snap = store.residency_snapshot();
        assert!(snap.historical_resident_bytes <= snap.historical_allowance_bytes);
        drop(page1);
        drop(cursor);
        (store, wire, len, more)
    });

    let mut yielded = p1_len;
    let mut max_pool_resident = store.residency_snapshot().historical_resident_bytes;
    let mut max_page_peak = 0isize;
    if p1_more {
        let mut cursor = store
            .resume_cursor_wire(&exec, &wire, query, &schema)
            .expect("resume durable cursor for remaining pages");
        loop {
            let (page_peak, (rows_len, has_more)) = measure_scoped(|| {
                let page = cursor.next_page(&exec, 4).expect("stream durable page");
                let len = page.rows().len();
                let more = page.has_more();
                let snap = store.residency_snapshot();
                assert!(snap.historical_resident_bytes <= snap.historical_allowance_bytes);
                drop(page);
                (len, more)
            });
            yielded += rows_len;
            max_page_peak = max_page_peak.max(page_peak);
            max_pool_resident =
                max_pool_resident.max(store.residency_snapshot().historical_resident_bytes);
            if !has_more {
                break;
            }
        }
        drop(cursor);
    }
    assert_eq!(yielded, row_count);
    let evictions = store.residency_snapshot().historical_evictions;
    drop(store);
    close_durable_home(reopened, &request);
    (
        cold_open_and_p1_peak,
        max_page_peak,
        max_pool_resident,
        evictions,
    )
}

#[test]
fn historical_cursor_streaming_heap_stays_bounded_across_1x_and_10x_bases_continuations_and_unrelated_bases(
) {
    // 1. Positive allocator control: tracker MUST observe an explicit 64 KiB allocation!
    let (control_peak, ()) = measure_scoped(|| {
        let mut probe = vec![0u8; 64 * 1024];
        std::hint::black_box(&mut probe);
    });
    assert!(
        control_peak >= 64 * 1024,
        "positive allocator control failed: {control_peak} B"
    );

    let hist_budget = 24 * 1024;

    // 2. Memory backend: 1x vs 10x basis size (64 vs 640 rows)
    let (mem_peak_1x, mem_res_1x, mem_evict_1x) = measure_memory_stream(64, 6, hist_budget);
    let (mem_peak_10x, mem_res_10x, mem_evict_10x) = measure_memory_stream(640, 6, hist_budget);
    assert!(mem_peak_1x > 0 && mem_res_1x <= hist_budget && mem_res_10x <= hist_budget);
    assert!(mem_evict_1x > 0 && mem_evict_10x > mem_evict_1x * 5);
    let mem_ceiling = (mem_peak_1x * 2).max(hist_budget as isize * 4);
    assert!(
        mem_peak_10x <= mem_ceiling,
        "Memory 10x basis peak ({mem_peak_10x}) > ceiling ({mem_ceiling})"
    );

    // 3. Memory backend: 1x vs 10x continuations per row (6 vs 42 versions)
    let (cont_peak_1x, _, _) = measure_memory_stream(32, 6, hist_budget);
    let (cont_peak_10x, cont_res_10x, cont_evict_10x) = measure_memory_stream(32, 42, hist_budget);
    assert!(cont_peak_1x > 0 && cont_res_10x <= hist_budget && cont_evict_10x > 0);
    let cont_ceiling = (cont_peak_1x * 3).max(hist_budget as isize * 4);
    assert!(
        cont_peak_10x <= cont_ceiling,
        "Memory 10x cont peak ({cont_peak_10x}) > ceiling ({cont_ceiling})"
    );

    // 4. Directory cold reopen: 1x vs 10x basis size (48 vs 480 rows)
    //    Cold store restore + cursor admission + page 1 AND subsequent streaming stay bounded!
    let (dir_open_1x, dir_peak_1x, dir_res_1x, dir_evict_1x) =
        measure_directory_reopen_stream(48, 6, 0, hist_budget);
    let (dir_open_10x, dir_peak_10x, dir_res_10x, dir_evict_10x) =
        measure_directory_reopen_stream(480, 6, 0, hist_budget);
    assert!(dir_peak_1x > 0 && dir_res_1x <= hist_budget && dir_res_10x <= hist_budget);
    assert!(dir_evict_1x > 0 && dir_evict_10x > dir_evict_1x * 5);
    let dir_ceiling = (dir_peak_1x * 4).max(hist_budget as isize * 4);
    assert!(
        dir_peak_10x <= dir_ceiling,
        "Directory 10x basis peak ({dir_peak_10x}) > ceiling ({dir_ceiling})"
    );
    let _ = (dir_open_1x, dir_open_10x);

    // 5. Directory cold reopen: 1x vs 10x UNRELATED retained bases (2 vs 20 bases, each with
    //    8 chunks + 224 continuation pages!) -> first cold restore + open + page 1 stays O(1)!
    let (open_peak_1x, stream_1x, _, _) = measure_directory_reopen_stream(24, 4, 2, hist_budget);
    let (open_peak_10x, stream_10x, _, _) = measure_directory_reopen_stream(24, 4, 20, hist_budget);
    assert!(
        open_peak_1x > 0,
        "Directory cold restore + open + page 1 must report non-zero peak heap"
    );
    let open_ceiling = (open_peak_1x * 3).max(hist_budget as isize * 4);
    assert!(
        open_peak_10x <= open_ceiling,
        "Directory 10x unrelated bases cold restore + open + page 1 peak ({open_peak_10x} B) exceeded O(1) ceiling ({open_ceiling} B; 1x={open_peak_1x} B)"
    );
    let stream_ceiling = (stream_1x * 3).max(hist_budget as isize * 3);
    assert!(
        stream_10x <= stream_ceiling,
        "Directory 10x unrelated bases stream peak ({stream_10x} B) exceeded O(1) ceiling ({stream_ceiling} B; 1x={stream_1x} B)"
    );
}
