//! Layer 3 `#[global_allocator]` scaling proofs for `[P1-9]`:
//! 1. Parent commit cost stays flat across `1x` vs `10x` unrelated older direct
//!    forks (`fork_children` indexed by cutoff commit via `BTreeMap::range`).
//! 2. Point-miss cost stays bounded (`O(log N)`) across `1x` vs `10x` repeated
//!    fork/write cycles (structural `CowMap` AVL tree with zero layer stacks).

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::lineage::{
    PersistentNameIdentity, PersistentNameKey, PersistentNameSelection,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;

use super::{ensure_tracker_installed, fixture, measure};

pub(super) fn fork_branch(
    request: &ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    source_branch: &BranchId,
    target_name: String,
) -> BranchId {
    let target = BranchId(target_name);
    let basis = fixture::observe_branch(runtime, source_branch);
    let services = runtime.owner_component_services();
    let src = services
        .fork_port()
        .admit_retained_fork_source(&basis)
        .unwrap();
    services
        .fork_port()
        .fork_retained_branch(request, target.clone(), src)
        .unwrap();
    target
}

#[test]
fn parent_commit_cost_is_flat_across_1x_vs_10x_unrelated_older_forks() {
    ensure_tracker_installed();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);

    let run_case = |unrelated_forks: usize| {
        let medium = TestDurableMedium::new();
        let runtime = fixture::durable_runtime(&medium, u64::MAX / 4);
        let main = runtime.main_branch_identity().branch_id().clone();
        let (seed_e, seed_n) = fixture::create_named_batch(
            request,
            &runtime,
            &main,
            &["f-seed-0".into(), "f-seed-1".into()],
        );
        fixture::publish_group(
            request,
            &runtime,
            &main,
            seed_e[0],
            vec![vec![
                (seed_n[0], AggregateMemberRole::Anchor),
                (seed_n[1], AggregateMemberRole::Interior),
            ]],
        );
        // Create `unrelated_forks` older direct forks at distinct earlier commits.
        for i in 0..unrelated_forks {
            let _ = fork_branch(request, &runtime, &main, format!("old-fork-{i:04}"));
        }
        // Measure a new named entity + group commit on `main` strictly AFTER all older forks.
        let ((), win) = measure(|| {
            let (e, n) =
                fixture::create_named_batch(request, &runtime, &main, &["after-forks".into()]);
            fixture::publish_group(
                request,
                &runtime,
                &main,
                e[0],
                vec![vec![
                    (seed_n[0], AggregateMemberRole::Anchor),
                    (n[0], AggregateMemberRole::Interior),
                ]],
            );
        });
        win
    };

    // Independent warmup before measuring 1x vs 10x.
    let _ = run_case(4);
    let win_1x = run_case(16);
    let win_10x = run_case(160);
    let ratio = win_10x.alloc_bytes as f64 / win_1x.alloc_bytes as f64;
    eprintln!(
        "[M4.3f-ALLOC] unrelated-forks parent commit: 1x(16 forks)={} B vs 10x(160 forks)={} B -> alloc_ratio={:.4}x",
        win_1x.alloc_bytes, win_10x.alloc_bytes, ratio
    );
    assert!(
        ratio <= 1.20,
        "parent commit cost must stay flat across 1x vs 10x unrelated older forks, got {ratio:.4}x"
    );
}

#[test]
fn point_miss_cost_stays_bounded_across_1x_vs_10x_fork_write_cycles() {
    ensure_tracker_installed();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);

    let absent_keys: Vec<PersistentNameKey> = (0..64u8)
        .map(|i| {
            PersistentNameKey::Entity(PersistentNameIdentity::of_pending_birth(
                "absent.scheme",
                "absent.occ",
                "absent.role",
                &[],
                &[i, i ^ 0x5A],
                false,
            ))
        })
        .collect();

    let run_case = |cycles: usize| {
        let medium = TestDurableMedium::new();
        let runtime = fixture::durable_runtime(&medium, u64::MAX / 4);
        let mut current = runtime.main_branch_identity().branch_id().clone();
        let (_, seed_n) =
            fixture::create_named_batch(request, &runtime, &current, &["cyc-seed".into()]);
        for c in 0..cycles {
            let _ =
                fixture::create_named_batch(request, &runtime, &current, &[format!("cyc-{c:04}")]);
            current = fork_branch(request, &runtime, &current, format!("cyc-b-{c:04}"));
        }
        let head = fixture::observe_branch(&runtime, &current);
        let ((), win) = measure(|| {
            runtime
                .with_persistent_names_at_observation(request, &head.observation(), |view| {
                    assert!(view
                        .resolve(
                            PersistentNameKey::Entity(seed_n[0]),
                            &PersistentNameSelection::Exact,
                        )
                        .result
                        .is_ok());
                    for &key in &absent_keys {
                        assert!(view
                            .resolve(key, &PersistentNameSelection::Exact)
                            .result
                            .is_err());
                    }
                })
                .unwrap();
        });
        win
    };

    // Independent warmup before measuring 1x vs 10x.
    let _ = run_case(4);
    let win_1x = run_case(20);
    let win_10x = run_case(200);
    let ratio = win_10x.alloc_bytes as f64 / win_1x.alloc_bytes as f64;
    eprintln!(
        "[M4.3f-ALLOC] fork/write-cycles point-miss: 1x(20 cycles)={} B vs 10x(200 cycles)={} B -> alloc_ratio={:.4}x",
        win_1x.alloc_bytes, win_10x.alloc_bytes, ratio
    );
    assert!(
        ratio <= 1.20,
        "point-miss cost must stay bounded across 1x vs 10x fork/write cycles, got {ratio:.4}x"
    );
}

#[test]
fn cold_uncheckpointed_fork_depth_point_miss_is_o1_in_reads_and_allocs() {
    ensure_tracker_installed();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);

    let absent_keys: Vec<PersistentNameKey> = (0..64u8)
        .map(|i| {
            PersistentNameKey::Entity(PersistentNameIdentity::of_pending_birth(
                "cold.absent.scheme",
                "cold.absent.occ",
                "cold.absent.role",
                &[],
                &[i ^ 0xA5, i],
                false,
            ))
        })
        .collect();

    let built = fixture::build_multi_branch_checkpoint(request, 1);

    let run_case = |depth: usize| {
        let runtime = fixture::reopen_cold(request, &built, u64::MAX / 4);
        let mut current = built.main.clone();
        for d in 0..depth {
            current = fork_branch(request, &runtime, &current, format!("cold-depth-{d:04}"));
        }
        let head = fixture::observe_branch(&runtime, &current);
        let reads_before = built.source.naming_page_reads.lock().unwrap().len();
        let ((), win) = measure(|| {
            runtime
                .with_persistent_names_at_observation(request, &head.observation(), |view| {
                    assert!(view
                        .resolve(
                            PersistentNameKey::Entity(built.sample_name),
                            &PersistentNameSelection::Exact,
                        )
                        .result
                        .is_ok());
                    for &key in &absent_keys {
                        assert!(view
                            .resolve(key, &PersistentNameSelection::Exact)
                            .result
                            .is_err());
                    }
                })
                .unwrap();
        });
        let reads = built.source.naming_page_reads.lock().unwrap().len() - reads_before;
        (win, reads)
    };

    // Independent warmup before measuring 1x (depth 10) vs 10x (depth 100).
    let _ = run_case(4);
    let (win_1x, reads_1x) = run_case(10);
    let (win_10x, reads_10x) = run_case(100);
    let ratio = win_10x.alloc_bytes as f64 / win_1x.alloc_bytes as f64;
    eprintln!(
        "[M4.3f-ALLOC] cold uncheckpointed fork-depth point-miss: 1x(depth=10)={} B (reads={}) vs 10x(depth=100)={} B (reads={}) -> alloc_ratio={:.4}x",
        win_1x.alloc_bytes, reads_1x, win_10x.alloc_bytes, reads_10x, ratio
    );
    assert_eq!(
        reads_10x, reads_1x,
        "cold naming page reads must be O(1) in uncheckpointed fork depth (10 vs 100)"
    );
    assert!(
        ratio <= 1.20,
        "cold uncheckpointed fork-depth point-miss allocs must stay O(1), got {ratio:.4}x"
    );
}
