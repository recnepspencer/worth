//! Store M3 C6: Unified Seeded Durability DST + In-Memory TLA+ Reference Oracle
//! + NDJSON Trace Emitter (`specs/tla/store_m3/StoreDurabilityTrace.tla`).
//!
//! Runs deterministic SplitMix64 interleaved sequences composing C1-C5 Store
//! durability transitions against REAL `Location::Directory` Store Homes,
//! verified step-by-step against the in-memory `StoreDurability.tla` oracle
//! and exact durable Relational/World branch & payload parity.
mod driver;
mod faults;
mod observe;
mod ops;
mod recovery;
mod tla_oracle;
mod tla_trace;

use self::driver::{run_durability_dst_seed, DstObservedStats};
use self::tla_oracle::{BuildTag, TlaAction, TlaOracle};
use super::super::media_fault_sweep::panic_message;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

const DEFAULT_DST_SEED_COUNT: u64 = 64;

fn parse_dst_seeds() -> Vec<u64> {
    let Ok(raw) = std::env::var("WORTH_DST_SEEDS") else {
        return (0..DEFAULT_DST_SEED_COUNT).collect();
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return (0..DEFAULT_DST_SEED_COUNT).collect();
    }
    if let Some((a, b)) = trimmed.split_once("..=") {
        let start: u64 = a.trim().parse().expect("valid start seed");
        let end: u64 = b.trim().parse().expect("valid inclusive end seed");
        assert!(start <= end, "empty seed range {trimmed}");
        return (start..=end).collect();
    }
    if let Some((a, b)) = trimmed.split_once("..") {
        let start: u64 = a.trim().parse().expect("valid start seed");
        let end: u64 = b.trim().parse().expect("valid end seed");
        assert!(start < end, "empty seed range {trimmed}");
        return (start..end).collect();
    }
    if trimmed.contains(',') {
        let seeds: Vec<u64> = trimmed
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().parse().expect("valid comma-separated seed"))
            .collect();
        assert!(!seeds.is_empty(), "WORTH_DST_SEEDS must not be empty");
        return seeds;
    }
    let count: u64 = trimmed
        .parse()
        .expect("WORTH_DST_SEEDS must be count, start..end, start..=end, or CSV");
    assert!(count > 0, "WORTH_DST_SEEDS count must be > 0");
    (0..count).collect()
}

fn trace_dir() -> Option<PathBuf> {
    std::env::var_os("WORTH_DST_TRACE_DIR")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

fn run_dst_parallel(seeds: &[u64], trace_out: Option<&PathBuf>) {
    if let Some(dir) = trace_out {
        std::fs::create_dir_all(dir)
            .unwrap_or_else(|e| panic!("create WORTH_DST_TRACE_DIR {dir:?}: {e}"));
    }
    let workers = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
        .clamp(1, 16)
        .min(seeds.len().max(1));
    let next = AtomicUsize::new(0);
    let aborted = Arc::new(AtomicBool::new(false));
    let errors = Mutex::new(Vec::<String>::new());
    let stats = Mutex::new(DstObservedStats::default());
    let started = Instant::now();

    std::thread::scope(|scope| {
        for _ in 0..workers {
            let (next, errors, stats) = (&next, &errors, &stats);
            let aborted = Arc::clone(&aborted);
            scope.spawn(move || loop {
                if aborted.load(Ordering::Acquire) {
                    break;
                }
                let idx = next.fetch_add(1, Ordering::Relaxed);
                if idx >= seeds.len() {
                    break;
                }
                let seed = seeds[idx];
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    run_durability_dst_seed(seed, trace_out.map(PathBuf::as_path))
                }));
                match res {
                    Ok(seed_stats) => {
                        stats.lock().unwrap().merge(seed_stats);
                    }
                    Err(payload) => {
                        aborted.store(true, Ordering::Release);
                        errors
                            .lock()
                            .unwrap()
                            .push(format!("seed={seed}: {}", panic_message(&*payload)));
                    }
                }
            });
        }
    });

    let elapsed = started.elapsed();
    let errs = errors.into_inner().unwrap();
    assert!(
        errs.is_empty(),
        "C6 Store Durability DST failures (re-run with WORTH_DST_SEEDS=<seed>):\n{}",
        errs.join("\n")
    );
    let totals = stats.into_inner().unwrap();
    if seeds.len() >= 16 {
        assert!(
            totals.fault_write_fired > 0
                && totals.fault_indeterminate_fired > 0
                && totals.fault_checkpoint_fired > 0
                && totals.fault_fork_fired > 0
                && totals.fault_recovery_staging_fired > 0
                && totals.commits_retired > 0
                && totals.d14_blocks_verified > 0,
            "C6 DST sweep missed expected fault/retirement/D14 coverage: {totals:?}"
        );
    }
    let secs = elapsed.as_secs_f64().max(1e-6);
    eprintln!(
        "[C6-DST] passed {} seeds ({} TLA+ steps, stats={:?}) in {:.2}s ({:.1} seeds/s, workers={})",
        seeds.len(),
        totals.steps,
        totals,
        secs,
        (seeds.len() as f64) / secs,
        workers
    );
}

#[test]
fn store_m3_c6_durability_dst_seeded() {
    let seeds = parse_dst_seeds();
    let trace_out = trace_dir();
    run_dst_parallel(&seeds, trace_out.as_ref());
}

/// Pinned regression for `seed = 3` (caught a genuine zero-crash production
/// reopen corruption bug where a live `RelationalBranchCreationPlan::ReuseExact`
/// World branch sharing Native `"main"` bricked reopen with
/// `CorruptCheckpoint("World cannot displace a non-Released original")` after
/// `"Original World Root"` advanced `"main"` and closed cleanly).
#[test]
fn store_m3_c6_durability_dst_pinned_seed_3_reuse_exact_reopen() {
    let stats = run_durability_dst_seed(3, None);
    assert!(
        stats.steps > 0,
        "pinned seed 3 must execute TLA+ verified steps"
    );
}

/// Minimal zero-crash regression for co-existing `ReuseExact` live World head
/// on a shared Native branch (`"main"`): Root advances C1 -> C2 while a live
/// `ReuseExact` branch remains at C1, closes cleanly, and reopens with exact
/// independent entity payloads on BOTH branches.
#[test]
fn store_m3_c6_reuse_exact_shared_native_branch_clean_reopen() {
    let lease = worth_query_testkit_host::facade::host::checkpoint_capture::lease(128 << 20);
    let exec = worth_execution::ExecutionRequest::leased(&lease);
    let root_dir = tempfile::tempdir().unwrap();
    let home_dir = root_dir.path().join("home");
    let cancel = worth_runtime_world::facade::RuntimeWorldCancellationSource::new();

    let (fx, _) = faults::open_world_home_with_faults(&home_dir, BuildTag::B1, 1, &exec);
    let boot = fx.bootstrap_root(&exec);
    let root0 = boot.product_branch().clone();
    drop(boot);
    let p1 = fx.prepare_publication(&exec, root0, crate::batches::create(1), &cancel.token());
    let root1 = fx
        .publish(&exec, p1, &cancel.token())
        .consume()
        .take_successor_observation()
        .unwrap();
    let c1_ord = root1.selected_commit().ordinal();
    let c1_payloads = observe::observe_branch_entity_payloads(&fx, &root1, &exec, "c1");
    assert_eq!(c1_payloads.len(), 1);
    let (reuse, _) = faults::try_create_branch(
        &fx,
        &exec,
        "reuse-live",
        "reuse-live",
        false,
        false,
        "reuse",
    )
    .expect("ReuseExact branch creation must succeed");
    assert_eq!(reuse.selected_commit().ordinal(), c1_ord);
    drop(reuse);
    let p2 = fx.prepare_publication(&exec, root1, crate::batches::create(2), &cancel.token());
    let root2 = fx
        .publish(&exec, p2, &cancel.token())
        .consume()
        .take_successor_observation()
        .unwrap();
    let c2_ord = root2.selected_commit().ordinal();
    let c2_payloads = observe::observe_branch_entity_payloads(&fx, &root2, &exec, "c2");
    assert_eq!(c2_payloads.len(), 2);
    assert!(c2_ord > c1_ord);
    drop(root2);
    let _ = fx
        .home
        .close(&exec)
        .unwrap_or_else(|e| panic!("clean close must succeed: {:?}", e.reason()));

    let (reopened, _) = faults::open_world_home_with_faults(&home_dir, BuildTag::B2, 1, &exec);
    let installed = reopened
        .home
        .world()
        .observation_port()
        .observe_installed_product_branches(&exec)
        .unwrap();
    assert_eq!(installed.branches().len(), 2);
    let root_after = installed
        .branches()
        .iter()
        .find(|b| b.branch_identity().name().as_str() == "Original World Root")
        .unwrap();
    let reuse_after = installed
        .branches()
        .iter()
        .find(|b| b.branch_identity().name().as_str() == "reuse-live")
        .unwrap();
    assert_eq!(root_after.selected_commit().ordinal(), c2_ord);
    assert_eq!(reuse_after.selected_commit().ordinal(), c1_ord);
    assert_eq!(
        observe::observe_branch_entity_payloads(&reopened, root_after, &exec, "root-reopen"),
        c2_payloads
    );
    assert_eq!(
        observe::observe_branch_entity_payloads(&reopened, reuse_after, &exec, "reuse-reopen"),
        c1_payloads
    );
    drop(installed);
    let _ = reopened.home.close(&exec);
}

#[test]
fn store_m3_c6_durability_dst_oracle_guards() {
    // 1. Retiring past sealed checkpointEpoch (using unsealed stagedCheckpoint) must panic.
    let mut oracle = TlaOracle::new(false);
    oracle.apply_and_verify(TlaAction::BeginCommit, "g1/begin");
    oracle.apply_and_verify(TlaAction::FlushJournal, "g1/flush");
    oracle.apply_and_verify(TlaAction::SealManifest, "g1/seal");
    oracle.apply_and_verify(TlaAction::BeginCheckpoint { epoch: 1 }, "g1/stage-cp");
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            oracle.apply_and_verify(
                TlaAction::RetireHistoryPrefix { new_floor: 1 },
                "g1/bad-retire",
            );
        }))
        .is_err(),
        "TLA+ oracle must reject RetiredHistoryGap past unsealed stagedCheckpoint"
    );

    // 2. Regressing durableHead after manifest_switch + chained crash must panic.
    let mut oracle2 = TlaOracle::new(false);
    oracle2.apply_and_verify(TlaAction::BeginCommit, "g2/c1-b");
    oracle2.apply_and_verify(TlaAction::FlushJournal, "g2/c1-f");
    oracle2.apply_and_verify(TlaAction::SealManifest, "g2/c1-s");
    oracle2.apply_and_verify(TlaAction::BeginCommit, "g2/c2-b");
    oracle2.apply_and_verify(TlaAction::FlushJournal, "g2/c2-f");
    oracle2.apply_and_verify(TlaAction::CrashKill, "g2/crash");
    oracle2.apply_and_verify(
        TlaAction::BeginRecover {
            build_id: BuildTag::B2,
        },
        "g2/rec1",
    );
    oracle2.apply_and_verify(
        TlaAction::AdvanceRecoverStage { resolved_head: 2 },
        "g2/adv1",
    );
    oracle2.apply_and_verify(TlaAction::CrashKill, "g2/chained");
    oracle2.apply_and_verify(
        TlaAction::BeginRecover {
            build_id: BuildTag::B3,
        },
        "g2/rec2",
    );
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            oracle2.apply_and_verify(
                TlaAction::AdvanceRecoverStage { resolved_head: 1 },
                "g2/bad-adv",
            );
        }))
        .is_err(),
        "TLA+ oracle must reject durableHead regression after manifest_switch"
    );

    // 3. In-session BeginCheckpoint after DiscardDerivedState (D14) must panic.
    let mut oracle3 = TlaOracle::new(false);
    oracle3.apply_and_verify(TlaAction::BeginCommit, "g3/c1-b");
    oracle3.apply_and_verify(TlaAction::FlushJournal, "g3/c1-f");
    oracle3.apply_and_verify(TlaAction::SealManifest, "g3/c1-s");
    oracle3.apply_and_verify(TlaAction::DiscardDerivedState, "g3/d14");
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            oracle3.apply_and_verify(TlaAction::BeginCheckpoint { epoch: 1 }, "g3/bad-cp");
        }))
        .is_err(),
        "TLA+ oracle must reject in-session checkpoint after D14 DiscardDerivedState"
    );
}
