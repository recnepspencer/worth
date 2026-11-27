//! Layer 3 unbypassable `#[global_allocator]` complexity proofs for Store M4.3f
//! (`P0-A` paged naming & group state + retention-floor compaction).
//!
//! Uses `tracking_allocator`'s scoped `AllocationGroupToken` over the court
//! binary's `#[global_allocator]` (`Allocator<System>`) to track exact
//! `(alloc_bytes, dealloc_bytes, live_bytes, peak_live_bytes)` on the calling
//! thread with zero manual production work counters and zero cross-thread noise.
mod cold_compaction;
mod fixture;
mod fork_and_layer_scaling;

use std::cell::Cell;
use std::sync::Once;
use tracking_allocator::{
    AllocationGroupId, AllocationGroupToken, AllocationRegistry, AllocationTracker,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    aggregate_identity::AggregateMemberRole,
    lineage::{PersistentNameKey, PersistentNameSelection},
};
use worth_testkit_relational::durable_medium::TestDurableMedium;

static INSTALL_TRACKER: Once = Once::new();
thread_local! {
    static ACTIVE_GROUP: Cell<usize> = const { Cell::new(0) };
    static ALLOC_BYTES: Cell<usize> = const { Cell::new(0) };
    static DEALLOC_BYTES: Cell<usize> = const { Cell::new(0) };
    static LIVE_BYTES: Cell<isize> = const { Cell::new(0) };
    static PEAK_LIVE_BYTES: Cell<isize> = const { Cell::new(0) };
}

struct CountingTracker;
impl AllocationTracker for CountingTracker {
    fn allocated(&self, _: usize, size: usize, _: usize, group: AllocationGroupId) {
        if ACTIVE_GROUP.with(Cell::get) != group.as_usize().get() {
            return;
        }
        ALLOC_BYTES.with(|c| c.set(c.get() + size));
        let next = LIVE_BYTES.with(|l| {
            let v = l.get() + size as isize;
            l.set(v);
            v
        });
        PEAK_LIVE_BYTES.with(|p| p.set(p.get().max(next)));
    }
    fn deallocated(
        &self,
        _: usize,
        size: usize,
        _: usize,
        src: AllocationGroupId,
        _: AllocationGroupId,
    ) {
        if ACTIVE_GROUP.with(Cell::get) != src.as_usize().get() {
            return;
        }
        DEALLOC_BYTES.with(|c| c.set(c.get() + size));
        LIVE_BYTES.with(|l| l.set(l.get() - size as isize));
    }
}

fn ensure_tracker_installed() {
    INSTALL_TRACKER.call_once(|| {
        AllocationRegistry::set_global_tracker(CountingTracker)
            .expect("global allocation tracker installs once");
        AllocationRegistry::enable_tracking();
    });
}

#[derive(Debug, Clone, Copy)]
struct AllocWindow {
    alloc_bytes: usize,
    net_live_delta: isize,
    peak_live_delta: isize,
}

struct GroupReset;
impl Drop for GroupReset {
    fn drop(&mut self) {
        ACTIVE_GROUP.with(|g| g.set(0));
    }
}

fn reset_counters(id: usize) -> GroupReset {
    ACTIVE_GROUP.with(|g| assert_eq!(g.replace(id), 0));
    ALLOC_BYTES.with(|c| c.set(0));
    DEALLOC_BYTES.with(|c| c.set(0));
    LIVE_BYTES.with(|c| c.set(0));
    PEAK_LIVE_BYTES.with(|c| c.set(0));
    GroupReset
}

fn measure<T>(operation: impl FnOnce() -> T) -> (T, AllocWindow) {
    ensure_tracker_installed();
    let mut token = AllocationGroupToken::register().expect("allocation observation group");
    let id = token.id().as_usize().get();
    let guard = token.enter();
    let _reset = reset_counters(id);
    let value = operation();
    let win = AllocWindow {
        alloc_bytes: ALLOC_BYTES.with(Cell::get),
        net_live_delta: LIVE_BYTES.with(Cell::get),
        peak_live_delta: PEAK_LIVE_BYTES.with(Cell::get),
    };
    drop(_reset);
    drop(guard);
    (value, win)
}

#[test]
fn cold_reopen_and_point_ops_scale_sublinearly_1x_vs_10x() {
    ensure_tracker_installed();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);

    // 1x = scale 1 (~256 bulk named entities + 8x32 group rounds across 2 branches),
    // 10x = scale 10 (~2,560 bulk named entities + 80x32 group rounds across 2 branches),
    // scaling BOTH live named-entity/group population AND multi-branch history 10x.
    let warmup = fixture::build_multi_branch_checkpoint(request, 1);
    let small = fixture::build_multi_branch_checkpoint(request, 1);
    let large = fixture::build_multi_branch_checkpoint(request, 10);
    assert!(
        small.naming_page_count >= 12,
        "1x baseline must already be multi-page per table: {} naming pages",
        small.naming_page_count
    );
    // Each branch keeps a one-page floor per table run at 1x, and kept
    // groups log their member deltas, so ~10x state is at least 6x pages.
    assert!(
        large.naming_page_count >= small.naming_page_count * 6,
        "10x checkpoint must scale naming pages ~10x: 1x={}, 10x={}",
        small.naming_page_count,
        large.naming_page_count
    );

    // Warm up on an independent throwaway checkpoint (NEVER `&small`).
    let _warm = fixture::reopen_cold(request, &warmup, u64::MAX / 4);
    drop(_warm);
    drop(warmup);

    let (reopened_1x, reopen_win_1x) =
        measure(|| fixture::reopen_cold(request, &small, u64::MAX / 4));
    let (reopened_10x, reopen_win_10x) =
        measure(|| fixture::reopen_cold(request, &large, u64::MAX / 4));

    assert_eq!(small.source.naming_page_reads.lock().unwrap().len(), 0);
    assert_eq!(large.source.naming_page_reads.lock().unwrap().len(), 0);

    let reopen_alloc_ratio = reopen_win_10x.alloc_bytes as f64 / reopen_win_1x.alloc_bytes as f64;
    let reopen_peak_ratio =
        reopen_win_10x.peak_live_delta.max(1) as f64 / reopen_win_1x.peak_live_delta.max(1) as f64;
    eprintln!(
        "[M4.3f-ALLOC] reopen 1x={} B (peak={} B, pages={}) vs 10x={} B (peak={} B, pages={}) -> alloc_ratio={:.4}x peak_ratio={:.4}x",
        reopen_win_1x.alloc_bytes,
        reopen_win_1x.peak_live_delta,
        small.naming_page_count,
        reopen_win_10x.alloc_bytes,
        reopen_win_10x.peak_live_delta,
        large.naming_page_count,
        reopen_alloc_ratio,
        reopen_peak_ratio,
    );

    // Probe 2: Point resolution (entity name + group name) + single named edit after reopen.
    let run_point_and_edit = |runtime: &worth_relational::facade::runtime::RelationalRuntime,
                              built: &fixture::BuiltCheckpoint| {
        let head = fixture::observe_branch(runtime, &built.main);
        runtime
            .with_persistent_names_at_observation(request, &head.observation(), |view| {
                let entity_key = PersistentNameKey::Entity(built.sample_name);
                assert!(view
                    .resolve(entity_key, &PersistentNameSelection::Exact)
                    .result
                    .is_ok());
                let group_key = view
                    .group_of_member(fixture::GROUP_SCHEME, &built.sample_name)
                    .expect("group survives reopen");
                assert!(view
                    .resolve(group_key, &PersistentNameSelection::Exact)
                    .result
                    .is_ok());
                assert!(!view.group_members(group_key).unwrap().is_empty());
            })
            .unwrap();
        let (created_e, created_n) =
            fixture::create_named_batch(request, runtime, &built.main, &["post-edit-e".into()]);
        fixture::publish_group(
            request,
            runtime,
            &built.main,
            created_e[0],
            vec![vec![
                (built.sample_name, AggregateMemberRole::Anchor),
                (created_n[0], AggregateMemberRole::Ordinary),
            ]],
        );
    };

    let ((), op_win_1x) = measure(|| run_point_and_edit(&reopened_1x, &small));
    let ((), op_win_10x) = measure(|| run_point_and_edit(&reopened_10x, &large));
    let op_alloc_ratio = op_win_10x.alloc_bytes as f64 / op_win_1x.alloc_bytes as f64;
    let reads_1x = small.source.naming_page_reads.lock().unwrap().len();
    let reads_10x = large.source.naming_page_reads.lock().unwrap().len();
    eprintln!(
        "[M4.3f-ALLOC] point+edit 1x={} B (naming_reads={}/{}) vs 10x={} B (naming_reads={}/{}) -> alloc_ratio={:.4}x",
        op_win_1x.alloc_bytes,
        reads_1x,
        small.naming_page_count,
        op_win_10x.alloc_bytes,
        reads_10x,
        large.naming_page_count,
        op_alloc_ratio
    );
    // For a true 10x live-entity + live-group + multi-branch scaling (~14 -> ~119 naming pages),
    // cold reopen reads 0 naming pages and scales only with O(pages) manifest metadata (<= 1.50x),
    // while point resolution + single edit touches an O(1) constant table count (<= 16 pages).
    assert!(
        reopen_alloc_ratio <= 1.50,
        "10x/1x cold reopen alloc_bytes ratio must stay <= 1.50x, got {reopen_alloc_ratio:.4}x"
    );
    assert!(
        reopen_peak_ratio <= 1.50,
        "10x/1x cold reopen peak_live_bytes ratio must stay <= 1.50x, got {reopen_peak_ratio:.4}x"
    );
    assert!(
        op_alloc_ratio <= 1.35 && reads_10x <= 16 && reads_10x * 5 < large.naming_page_count,
        "10x/1x point+edit must stay O(1): ratio={op_alloc_ratio:.4}x, reads={reads_10x}/{}",
        large.naming_page_count
    );

    drop(reopened_1x);
    drop(reopened_10x);
}

#[test]
fn working_set_eviction_bounds_live_bytes_under_tight_historical_budget() {
    ensure_tracker_installed();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let built = fixture::build_multi_branch_checkpoint(request, 3);
    assert!(
        built.naming_page_count >= 15,
        "expected >=15 naming pages, got {}",
        built.naming_page_count
    );

    let tight_budget = 64 * 1024;
    let runtime = fixture::reopen_cold(request, &built, tight_budget);
    let head = fixture::observe_branch(&runtime, &built.main);

    // Naming-ONLY traversal (zero commit-index lookups) proving ColdNamingStore
    // itself evicts under the historical pool budget.
    let ((), trav_win) = measure(|| {
        runtime
            .with_persistent_names_at_observation(request, &head.observation(), |view| {
                for name in &built.all_names {
                    let key = PersistentNameKey::Entity(*name);
                    assert!(view
                        .resolve(key, &PersistentNameSelection::Exact)
                        .result
                        .is_ok());
                    let _ = view.group_of_member(fixture::GROUP_SCHEME, name);
                }
                for &group_key in &built.all_group_keys {
                    let _ = view.resolve(group_key, &PersistentNameSelection::Exact);
                }
            })
            .unwrap();
    });

    let pressure = runtime.residency_pressure().unwrap();
    let naming_obs = runtime
        .cold_naming_observation()
        .expect("cold-reopened runtime exposes ColdNamingStore observation");
    eprintln!(
        "[M4.3f-ALLOC] naming-only eviction: naming_evictions={}, naming_resident_pages={}, historical_resident={} B (budget={} B), net_live_delta={} B, peak_live_delta={} B",
        naming_obs.evictions,
        naming_obs.resident_pages,
        pressure.historical_resident_bytes,
        tight_budget,
        trav_win.net_live_delta,
        trav_win.peak_live_delta,
    );
    assert!(
        naming_obs.evictions > 0,
        "ColdNamingStore itself must evict pages under tight historical budget: {naming_obs:?}"
    );
    assert!(
        naming_obs.resident_pages < built.naming_page_count,
        "resident naming pages {} must stay below total naming pages {}",
        naming_obs.resident_pages,
        built.naming_page_count
    );
    assert!(pressure.historical_resident_bytes <= tight_budget);
    let total_raw_naming_bytes: usize = built
        .source
        .members
        .iter()
        .filter(|(_, b)| {
            b.bytes().starts_with(b"WORTH-REL-CHUNK1\n") && b.bytes().get(17) == Some(&5)
        })
        .map(|(_, b)| b.bytes().len())
        .sum();
    assert!(
        trav_win.net_live_delta < total_raw_naming_bytes as isize,
        "retained live delta {} B must stay below total raw naming pages {} B",
        trav_win.net_live_delta,
        total_raw_naming_bytes,
    );
}

#[test]
fn retention_floor_compaction_drops_global_allocator_live_bytes() {
    ensure_tracker_installed();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);

    // Enter one observation group across both workload build and compaction so
    // deallocations of pruned retired names/tenures attribute back to this test.
    let mut token = AllocationGroupToken::register().expect("compaction observation group");
    let id = token.id().as_usize().get();
    let guard = token.enter();
    let _reset = reset_counters(id);

    let medium = TestDurableMedium::new();
    let runtime = fixture::durable_runtime(&medium, u64::MAX / 4);
    let main = runtime.main_branch_identity().branch_id().clone();

    let (anchor_e, anchor_n) =
        fixture::create_named_batch(request, &runtime, &main, &["anchor".into()]);
    let (keep_name, prior_commit) = cold_compaction::churn_rounds(
        request,
        &runtime,
        &main,
        anchor_e[0],
        anchor_n[0],
        "churn",
        120,
    );

    let head = fixture::observe_branch(&runtime, &main);
    let oracle_group = runtime
        .with_persistent_names_at_observation(request, &head.observation(), |view| {
            view.group_of_member(fixture::GROUP_SCHEME, &keep_name)
        })
        .unwrap();
    assert!(oracle_group.is_some());

    let live_before = LIVE_BYTES.with(Cell::get);
    let alloc_before = ALLOC_BYTES.with(Cell::get);
    let dealloc_before = DEALLOC_BYTES.with(Cell::get);
    runtime
        .advance_naming_retention_floor(&main, prior_commit)
        .unwrap();
    let live_after = LIVE_BYTES.with(Cell::get);
    let compact_alloc = ALLOC_BYTES.with(Cell::get).saturating_sub(alloc_before);
    let compact_dealloc = DEALLOC_BYTES.with(Cell::get).saturating_sub(dealloc_before);
    let bytes_freed = live_before - live_after;
    eprintln!(
        "[M4.3f-ALLOC] retention compaction: live_before={} B, live_after={} B, freed={} B (dealloc={} B, alloc={} B)",
        live_before, live_after, bytes_freed, compact_dealloc, compact_alloc
    );
    assert!(
        bytes_freed > 16 * 1024,
        "retention-floor compaction must meaningfully drop live bytes, freed only {bytes_freed} B"
    );

    let after_group = runtime
        .with_persistent_names_at_observation(request, &head.observation(), |view| {
            let g = view.group_of_member(fixture::GROUP_SCHEME, &keep_name);
            assert!(view
                .resolve(
                    PersistentNameKey::Entity(anchor_n[0]),
                    &PersistentNameSelection::Exact
                )
                .result
                .is_ok());
            g
        })
        .unwrap();
    assert_eq!(after_group, oracle_group);
    runtime
        .advance_naming_retention_floor(&main, prior_commit)
        .unwrap();
    drop(_reset);
    drop(guard);
}
