//! Layer 3 `#[global_allocator]` + visited-node scaling courts for:
//! 1. Equal-cardinality member-swap interleaving across `1x` vs `10x` forks.
//! 2. Warm historical forks created after `main` already advanced past their cutoffs.

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::lineage::{
    PersistentNameIdentity, PersistentNameKey, PersistentNameSelection,
};
use worth_testkit_relational::cold_recovery::{capture_external_checkpoint, immutable_source};
use worth_testkit_relational::durable_medium::TestDurableMedium;

use super::super::bounded_naming_fixture::{
    durable_runtime_with, naming_pages, naming_schema, observe_branch, publish_group_partition,
    reopen_with, GROUP_SCHEME,
};
use super::super::naming_branch_root_and_dissolve::fork_retained_at;
use super::fixture::create_named_batch_in_partition;
use super::fork_and_layer_scaling::fork_branch;
use super::fork_delta_checkpoint_scaling::{assert_exact_entity, assert_unknown_entity};
use super::{ensure_tracker_installed, measure};

struct ScalingMetrics {
    pages: usize,
    member_bytes: usize,
    capture_alloc_bytes: usize,
    diff_visits: usize,
}

fn run_equal_cardinality_swap_interleaving(
    request: &ExecutionRequest<'_, '_>,
    fork_every: usize,
) -> ScalingMetrics {
    let medium = TestDurableMedium::new();
    let producer = durable_runtime_with(&medium, u64::MAX / 4, naming_schema(true));
    let main = producer.main_branch_identity().branch_id().clone();

    let mut base_entities = Vec::new();
    let mut base_names = Vec::new();
    for b in 0..12usize {
        let keys: Vec<String> = (0..20).map(|i| format!("swap-b{b:02}-{i:02}")).collect();
        let (e, n) =
            create_named_batch_in_partition(request, &producer, &main, PartitionId(1), &keys);
        base_entities.extend(e);
        base_names.extend(n);
    }

    // 20 kept groups of 12 members each; every round swaps 1 member between
    // two adjacent groups (rotating pairs), keeping exact total cardinality and
    // table lengths identical at every commit while keeping per-group timelines compact.
    let mut groups: Vec<Vec<(PersistentNameIdentity, AggregateMemberRole)>> = base_names
        .chunks(12)
        .map(|chunk| {
            chunk
                .iter()
                .enumerate()
                .map(|(i, &n)| {
                    (
                        n,
                        if i == 0 {
                            AggregateMemberRole::Anchor
                        } else {
                            AggregateMemberRole::Interior
                        },
                    )
                })
                .collect()
        })
        .collect();
    publish_group_partition(
        request,
        &producer,
        &main,
        base_entities[0],
        "swap-init",
        groups.clone(),
    );

    let mut forks: Vec<(BranchId, PersistentNameIdentity, PersistentNameIdentity)> = Vec::new();
    for round in 0..40usize {
        let ga = (round * 2) % groups.len();
        let gb = ga + 1;
        let slot = 1 + (round % 11);
        let a = groups[ga][slot];
        let b = groups[gb][slot];
        groups[ga][slot] = b;
        groups[gb][slot] = a;
        publish_group_partition(
            request,
            &producer,
            &main,
            base_entities[0],
            &format!("swap-r{round:02}"),
            groups.clone(),
        );
        if (round + 1) % fork_every == 0 {
            let br = fork_branch(request, &producer, &main, format!("swap-fork-{round:02}"));
            forks.push((br, groups[ga][0].0, groups[ga][slot].0));
        }
    }

    let ((_, _, diff_visits), cap_win) =
        measure(|| producer.seal_naming_snapshot_with_visits().unwrap());
    let (ck, _) = capture_external_checkpoint(request, &producer);
    let src = immutable_source(&ck, &medium);
    let locs = naming_pages(&src);
    let member_bytes: usize = locs.iter().map(|l| src.members[l].bytes().len()).sum();

    let fresh = durable_runtime_with(&medium, u64::MAX / 4, naming_schema(true));
    let reopened = reopen_with(request, ck, src, fresh);
    for (br, anchor_n, swapped_n) in [&forks[0], forks.last().unwrap()] {
        let obs = observe_branch(&reopened, br);
        reopened
            .with_persistent_names_at_observation(request, &obs.observation(), |view| {
                let g0 = view
                    .group_of_member(GROUP_SCHEME, anchor_n)
                    .expect("group 0 survives");
                assert_eq!(view.group_of_member(GROUP_SCHEME, swapped_n), Some(g0));
            })
            .unwrap();
    }

    ScalingMetrics {
        pages: locs.len(),
        member_bytes,
        capture_alloc_bytes: cap_win.alloc_bytes,
        diff_visits,
    }
}

#[test]
fn equal_cardinality_member_swap_interleaving_scales_capture_visits_and_allocs_sublinearly() {
    ensure_tracker_installed();
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);

    let _ = run_equal_cardinality_swap_interleaving(request, 20);
    let small = run_equal_cardinality_swap_interleaving(request, 10); // 1x = 4 forks
    let large = run_equal_cardinality_swap_interleaving(request, 1); // 10x = 40 forks

    let visits_ratio = large.diff_visits as f64 / small.diff_visits.max(1) as f64;
    let alloc_ratio = large.capture_alloc_bytes as f64 / small.capture_alloc_bytes.max(1) as f64;
    let bytes_ratio = large.member_bytes as f64 / small.member_bytes.max(1) as f64;
    eprintln!(
        "[CHK24-EQ-SWAP] 1x(4 forks) vs 10x(40 forks) over 240 names / 20 kept groups:\n  \
         diff_visits: {} -> {} ({visits_ratio:.3}x)\n  \
         capture_alloc_bytes: {} -> {} ({alloc_ratio:.3}x)\n  \
         pages: {} -> {}, member_bytes: {} -> {} ({bytes_ratio:.3}x)",
        small.diff_visits,
        large.diff_visits,
        small.capture_alloc_bytes,
        large.capture_alloc_bytes,
        small.pages,
        large.pages,
        small.member_bytes,
        large.member_bytes,
    );
    assert!(
        visits_ratio < 4.5,
        "10x equal-cardinality interleaved forks must not scale quadratically in visited nodes: {visits_ratio:.3}x ({} -> {})",
        small.diff_visits,
        large.diff_visits
    );
    assert!(
        alloc_ratio < 3.5,
        "10x equal-cardinality interleaved forks capture allocs must scale sublinearly: {alloc_ratio:.3}x"
    );
    assert!(
        bytes_ratio < 2.5,
        "10x equal-cardinality interleaved forks member bytes must scale sublinearly: {bytes_ratio:.3}x"
    );
}

fn run_warm_historical_forks(
    request: &ExecutionRequest<'_, '_>,
    fork_every: usize,
) -> ScalingMetrics {
    let medium = TestDurableMedium::new();
    let producer = durable_runtime_with(&medium, u64::MAX / 4, naming_schema(true));
    let main = producer.main_branch_identity().branch_id().clone();

    let mut base_entities = Vec::new();
    let mut base_names = Vec::new();
    for b in 0..16usize {
        let keys: Vec<String> = (0..25).map(|i| format!("warm-b{b:02}-{i:02}")).collect();
        let (e, n) =
            create_named_batch_in_partition(request, &producer, &main, PartitionId(1), &keys);
        base_entities.extend(e);
        base_names.extend(n);
    }
    let groups: Vec<Vec<(PersistentNameIdentity, AggregateMemberRole)>> = base_names
        .chunks(50)
        .map(|chunk| {
            chunk
                .iter()
                .enumerate()
                .map(|(i, &n)| {
                    (
                        n,
                        if i == 0 {
                            AggregateMemberRole::Anchor
                        } else {
                            AggregateMemberRole::Interior
                        },
                    )
                })
                .collect()
        })
        .collect();
    publish_group_partition(
        request,
        &producer,
        &main,
        base_entities[0],
        "warm-groups",
        groups,
    );

    // Hold `main`'s 20 historical batches fixed, recording cutoffs BEFORE advancing
    // `main`, then create ALL retained historical forks AFTER `main` moved past them.
    let mut cutoffs = Vec::with_capacity(20);
    for step in 0..20usize {
        let basis = observe_branch(&producer, &main);
        let keys = vec![
            format!("warm-step-{step:02}-a"),
            format!("warm-step-{step:02}-b"),
        ];
        let (e, n) =
            create_named_batch_in_partition(request, &producer, &main, PartitionId(1), &keys);
        cutoffs.push((basis, e[0], n[0]));
    }
    let mut forks: Vec<(BranchId, usize, EntityId, PersistentNameIdentity)> = Vec::new();
    for (idx, (basis, post_e, post_n)) in cutoffs.iter().enumerate() {
        if (idx + 1) % fork_every == 0 {
            let br = BranchId(format!("warm-hist-{idx:02}"));
            fork_retained_at(request, &producer, basis, &br).unwrap();
            forks.push((br, idx, *post_e, *post_n));
        }
    }

    let ((_, _, diff_visits), cap_win) =
        measure(|| producer.seal_naming_snapshot_with_visits().unwrap());
    let (ck, _) = capture_external_checkpoint(request, &producer);
    let src = immutable_source(&ck, &medium);
    let locs = naming_pages(&src);
    let member_bytes: usize = locs.iter().map(|l| src.members[l].bytes().len()).sum();

    let fresh = durable_runtime_with(&medium, u64::MAX / 4, naming_schema(true));
    let reopened = reopen_with(request, ck, src, fresh);
    for (br, cutoff_idx, _, post_n) in &forks {
        let obs = observe_branch(&reopened, br);
        reopened
            .with_persistent_names_at_observation(request, &obs.observation(), |view| {
                assert_exact_entity(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(base_names[0]),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    base_entities[0],
                );
                assert!(view.group_of_member(GROUP_SCHEME, &base_names[0]).is_some());
                assert_unknown_entity(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(*post_n),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    *post_n,
                );
                if *cutoff_idx > 0 {
                    let (_, prev_e, prev_n) = cutoffs[*cutoff_idx - 1];
                    assert_exact_entity(
                        &view
                            .resolve(
                                PersistentNameKey::Entity(prev_n),
                                &PersistentNameSelection::Exact,
                            )
                            .result,
                        prev_e,
                    );
                }
            })
            .unwrap();
    }

    ScalingMetrics {
        pages: locs.len(),
        member_bytes,
        capture_alloc_bytes: cap_win.alloc_bytes,
        diff_visits,
    }
}

#[test]
fn warm_historical_forks_preserve_structural_sharing_and_scale_sublinearly_1x_vs_10x() {
    ensure_tracker_installed();
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);

    let _ = run_warm_historical_forks(request, 10);
    let small = run_warm_historical_forks(request, 10); // 1x = 2 warm historical forks
    let large = run_warm_historical_forks(request, 1); // 10x = 20 warm historical forks

    let page_ratio = large.pages as f64 / small.pages.max(1) as f64;
    let bytes_ratio = large.member_bytes as f64 / small.member_bytes.max(1) as f64;
    let alloc_ratio = large.capture_alloc_bytes as f64 / small.capture_alloc_bytes.max(1) as f64;
    let visits_per_fork = large.diff_visits as f64 / 20.0;
    let visits_ratio = large.diff_visits as f64 / small.diff_visits.max(1) as f64;
    eprintln!(
        "[CHK24-WARM-HIST] 1x(2 forks) vs 10x(20 forks) over 400 names + groups:\n  \
         pages: {} -> {} ({page_ratio:.3}x)\n  \
         member_bytes: {} -> {} ({bytes_ratio:.3}x)\n  \
         capture_alloc_bytes: {} -> {} ({alloc_ratio:.3}x)\n  \
         diff_visits: {} -> {} ({visits_ratio:.3}x, {visits_per_fork:.1}/fork)",
        small.pages,
        large.pages,
        small.member_bytes,
        large.member_bytes,
        small.capture_alloc_bytes,
        large.capture_alloc_bytes,
        small.diff_visits,
        large.diff_visits,
    );
    assert!(
        page_ratio < 3.0,
        "10x warm historical forks pages must stay sublinear: {page_ratio:.3}x"
    );
    assert!(
        bytes_ratio < 1.5,
        "10x warm historical forks member bytes must scale O(names + fork deltas): {bytes_ratio:.3}x"
    );
    assert!(
        alloc_ratio < 2.0,
        "10x warm historical forks capture allocs must scale sublinearly: {alloc_ratio:.3}x"
    );
    assert!(
        visits_per_fork < 400.0 && visits_ratio < 7.5,
        "10x warm historical forks must preserve O(log N) CowMap sharing: visits={} ({visits_per_fork:.1}/fork, ratio={visits_ratio:.3}x)",
        large.diff_visits
    );
}
