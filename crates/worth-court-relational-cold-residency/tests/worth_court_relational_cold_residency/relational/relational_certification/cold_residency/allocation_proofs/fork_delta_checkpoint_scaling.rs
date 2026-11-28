//! Layer 3 `#[global_allocator]` + manifest/member byte scaling court for
//! fork-delta `CHK24` naming sealing (`#480`).
//!
//! 1. Holds `N = 600` structural-identity names (and Anchor groups) fixed on
//!    `main` while scaling branch count `1x` (`4` forks) vs `10x` (`40` forks),
//!    asserting naming pages, member bytes, `manifest.naming` bytes, capture
//!    allocations, and cold reopen allocations scale `O(names + fork deltas)`.
//! 2. Interleaved historical sibling forks (`1x` = 4 forks vs `10x` = 40 forks
//!    across 40 batches of 10 names): measures capture `#[global_allocator]`
//!    allocations, naming pages, and member bytes, proving capture work itself
//!    scales `O(names + fork deltas)` instead of `Θ(branches * names)`.

use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::durability::RelationalNativeCheckpoint;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::lineage::{
    PersistentNameIdentity, PersistentNameKey, PersistentNameResolution,
    PersistentNameResolutionDenial, PersistentNameSelection, PersistentNameTarget,
};
use worth_testkit_relational::cold_recovery::{capture_external_checkpoint, immutable_source};
use worth_testkit_relational::durable_medium::immutable_members::ImmutableCheckpointMemberSource;
use worth_testkit_relational::durable_medium::TestDurableMedium;

use super::super::bounded_naming_fixture::{
    create_named_entities, durable_runtime_with, naming_pages, naming_schema, observe_branch,
    publish_group_partition, reopen_with, GROUP_SCHEME,
};
use super::super::checkpoint_wire_fixture::{field, CHECKPOINT_MAGIC};
use super::fixture::create_named_batch_in_partition;
use super::fork_and_layer_scaling::fork_branch;
use super::{ensure_tracker_installed, measure, AllocWindow};

const BASE_BATCHES: usize = 12;
const NAMES_PER_BATCH: usize = 50; // 600 structural-identity names on `main`
const FORK_DELTA_NAMES: usize = 2;

struct ForkScalingRun {
    naming_pages: usize,
    naming_member_bytes: usize,
    naming_manifest_bytes: usize,
    manifest_bytes: usize,
    capture_alloc: AllocWindow,
    reopen_alloc: AllocWindow,
}

pub(super) fn assert_exact_entity(
    result: &Result<PersistentNameResolution, PersistentNameResolutionDenial>,
    expected_entity: EntityId,
) {
    assert!(
        matches!(
            result,
            Ok(PersistentNameResolution::Exact(
                PersistentNameTarget::Entity { entity, .. }
            )) if *entity == expected_entity
        ),
        "expected Exact entity {expected_entity:?}, got {result:?}"
    );
}

pub(super) fn assert_unknown_entity(
    result: &Result<PersistentNameResolution, PersistentNameResolutionDenial>,
    expected_name: PersistentNameIdentity,
) {
    assert_eq!(
        result,
        &Err(PersistentNameResolutionDenial::UnknownName {
            name: PersistentNameKey::Entity(expected_name),
        }),
        "expected exact UnknownName denial (never storage damage)"
    );
}

fn run_fork_scaling(request: &ExecutionRequest<'_, '_>, fork_count: usize) -> ForkScalingRun {
    let medium = TestDurableMedium::new();
    let producer = durable_runtime_with(&medium, u64::MAX / 4, naming_schema(true));
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, seed, _) = create_named_entities(request, &producer, &main, &["scale-seed".into()]);
    let mut main_entities = Vec::with_capacity(BASE_BATCHES * NAMES_PER_BATCH);
    let mut main_names = Vec::with_capacity(BASE_BATCHES * NAMES_PER_BATCH);
    for batch in 0..BASE_BATCHES {
        let keys: Vec<String> = (0..NAMES_PER_BATCH)
            .map(|i| format!("base-{batch:02}-{i:02}"))
            .collect();
        let (entities, names) =
            create_named_batch_in_partition(request, &producer, &main, PartitionId(1), &keys);
        main_entities.extend(entities);
        main_names.extend(names);
    }
    let groups: Vec<Vec<(PersistentNameIdentity, AggregateMemberRole)>> = main_names
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
                            AggregateMemberRole::Ordinary
                        },
                    )
                })
                .collect()
        })
        .collect();
    publish_group_partition(request, &producer, &main, seed[0], "base-groups", groups);

    let mut fork_samples: Vec<(BranchId, Option<(EntityId, PersistentNameIdentity)>)> =
        Vec::with_capacity(fork_count);
    for f in 0..fork_count {
        let branch = fork_branch(request, &producer, &main, format!("delta-fork-{f:03}"));
        let own = if f % 2 == 0 {
            let keys: Vec<String> = (0..FORK_DELTA_NAMES)
                .map(|j| format!("fork-{f:03}-n-{j}"))
                .collect();
            let (added_e, added_n) =
                create_named_batch_in_partition(request, &producer, &branch, PartitionId(1), &keys);
            Some((added_e[0], added_n[0]))
        } else {
            None
        };
        fork_samples.push((branch, own));
    }

    let (_, capture_alloc) = measure(|| producer.seal_naming_snapshot().unwrap());
    let (checkpoint, _) = capture_external_checkpoint(request, &producer);
    let source: Arc<ImmutableCheckpointMemberSource> = immutable_source(&checkpoint, &medium);
    let page_locs = naming_pages(&source);
    let naming_page_count = page_locs.len();
    let naming_member_bytes: usize = page_locs
        .iter()
        .map(|loc| source.members[loc].bytes().len())
        .sum();
    let manifest_bytes = checkpoint.bytes().len();
    let body = checkpoint.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let naming_manifest_bytes = field(field(body, "manifest"), "naming").len();

    let fresh = durable_runtime_with(&medium, u64::MAX / 4, naming_schema(true));
    let ck_clone: RelationalNativeCheckpoint = checkpoint.clone();
    let src_clone = source.clone();
    let (reopened, reopen_alloc) = measure(|| reopen_with(request, ck_clone, src_clone, fresh));
    assert_eq!(
        source.naming_page_reads.lock().unwrap().len(),
        0,
        "cold reopen must read 0 naming pages"
    );

    let main_obs = observe_branch(&reopened, &main);
    let main_group = reopened
        .with_persistent_names_at_observation(request, &main_obs.observation(), |view| {
            assert_exact_entity(
                &view
                    .resolve(
                        PersistentNameKey::Entity(main_names[0]),
                        &PersistentNameSelection::Exact,
                    )
                    .result,
                main_entities[0],
            );
            let grp = view
                .group_of_member(GROUP_SCHEME, &main_names[0])
                .expect("main group survives");
            if let Some((_, own)) = fork_samples[0].1 {
                assert_unknown_entity(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(own),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    own,
                );
            }
            grp
        })
        .unwrap();
    for (branch, own_pair) in [&fork_samples[0], fork_samples.last().unwrap()] {
        let obs = observe_branch(&reopened, branch);
        reopened
            .with_persistent_names_at_observation(request, &obs.observation(), |view| {
                assert_exact_entity(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(main_names[0]),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    main_entities[0],
                );
                assert_eq!(
                    view.group_of_member(GROUP_SCHEME, &main_names[0]),
                    Some(main_group)
                );
                if let Some((own_e, own_n)) = own_pair {
                    assert_exact_entity(
                        &view
                            .resolve(
                                PersistentNameKey::Entity(*own_n),
                                &PersistentNameSelection::Exact,
                            )
                            .result,
                        *own_e,
                    );
                }
            })
            .unwrap();
    }

    ForkScalingRun {
        naming_pages: naming_page_count,
        naming_member_bytes,
        naming_manifest_bytes,
        manifest_bytes,
        capture_alloc,
        reopen_alloc,
    }
}

#[test]
fn fork_delta_checkpoint_pages_bytes_manifest_and_reopen_scale_sublinearly_across_1x_vs_10x_branches(
) {
    ensure_tracker_installed();
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);

    let _ = run_fork_scaling(request, 2);
    let small = run_fork_scaling(request, 4);
    let large = run_fork_scaling(request, 40);

    let page_ratio = large.naming_pages as f64 / small.naming_pages.max(1) as f64;
    let member_ratio = large.naming_member_bytes as f64 / small.naming_member_bytes.max(1) as f64;
    let naming_manifest_ratio =
        large.naming_manifest_bytes as f64 / small.naming_manifest_bytes.max(1) as f64;
    let manifest_ratio = large.manifest_bytes as f64 / small.manifest_bytes.max(1) as f64;
    let capture_ratio =
        large.capture_alloc.alloc_bytes as f64 / small.capture_alloc.alloc_bytes.max(1) as f64;
    let reopen_ratio =
        large.reopen_alloc.alloc_bytes as f64 / small.reopen_alloc.alloc_bytes.max(1) as f64;

    eprintln!(
        "[CHK24-FORK-DELTA] 1x(4 forks) vs 10x(40 forks) over 600 names:\n  \
         naming_pages: {} -> {} ({page_ratio:.3}x)\n  \
         naming_member_bytes: {} -> {} ({member_ratio:.3}x)\n  \
         naming_manifest_bytes: {} -> {} ({naming_manifest_ratio:.3}x)\n  \
         total_manifest_bytes: {} -> {} ({manifest_ratio:.3}x)\n  \
         capture_alloc_bytes: {} -> {} ({capture_ratio:.3}x)\n  \
         reopen_alloc_bytes: {} -> {} ({reopen_ratio:.3}x)",
        small.naming_pages,
        large.naming_pages,
        small.naming_member_bytes,
        large.naming_member_bytes,
        small.naming_manifest_bytes,
        large.naming_manifest_bytes,
        small.manifest_bytes,
        large.manifest_bytes,
        small.capture_alloc.alloc_bytes,
        large.capture_alloc.alloc_bytes,
        small.reopen_alloc.alloc_bytes,
        large.reopen_alloc.alloc_bytes,
    );

    assert!(page_ratio < 2.2, "page_ratio={page_ratio:.3}x");
    assert!(member_ratio < 1.25, "member_ratio={member_ratio:.3}x");
    assert!(
        naming_manifest_ratio < 3.0,
        "naming_manifest_ratio={naming_manifest_ratio:.3}x"
    );
    assert!(
        capture_ratio < 3.0,
        "10x forks capture allocs must scale sublinearly: {capture_ratio:.3}x"
    );
    assert!(
        reopen_ratio <= manifest_ratio * 1.15,
        "reopen={reopen_ratio:.3}x vs manifest={manifest_ratio:.3}x"
    );
}

#[test]
fn interleaved_historical_forks_scale_capture_work_and_bytes_sublinearly_1x_vs_10x() {
    ensure_tracker_installed();
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);

    let run = |fork_every: Option<usize>| {
        let medium = TestDurableMedium::new();
        let producer = durable_runtime_with(&medium, u64::MAX / 4, naming_schema(true));
        let main = producer.main_branch_identity().branch_id().clone();
        let mut batch_first: Vec<(EntityId, PersistentNameIdentity)> = Vec::new();
        let mut forks = Vec::new();
        for b in 0..40usize {
            let keys: Vec<String> = (0..10).map(|i| format!("ihf-{b:02}-{i}")).collect();
            let (entities, names) =
                create_named_batch_in_partition(request, &producer, &main, PartitionId(1), &keys);
            batch_first.push((entities[0], names[0]));
            if fork_every.is_some_and(|step| (b + 1) % step == 0) {
                let br = fork_branch(request, &producer, &main, format!("hist-fork-{b:02}"));
                forks.push((b, br));
            }
        }
        let (_, capture_alloc) = measure(|| producer.seal_naming_snapshot().unwrap());
        let (ck, _) = capture_external_checkpoint(request, &producer);
        let src = immutable_source(&ck, &medium);
        let locs = naming_pages(&src);
        let member_bytes: usize = locs.iter().map(|l| src.members[l].bytes().len()).sum();
        let fresh = durable_runtime_with(&medium, u64::MAX / 4, naming_schema(true));
        let reopened = reopen_with(request, ck, src, fresh);
        for (cutoff_batch, br) in &forks {
            let obs = observe_branch(&reopened, br);
            let (at_e, at_n) = batch_first[*cutoff_batch];
            reopened
                .with_persistent_names_at_observation(request, &obs.observation(), |view| {
                    assert_exact_entity(
                        &view
                            .resolve(
                                PersistentNameKey::Entity(at_n),
                                &PersistentNameSelection::Exact,
                            )
                            .result,
                        at_e,
                    );
                    if *cutoff_batch + 1 < batch_first.len() {
                        let (_, later_n) = batch_first[*cutoff_batch + 1];
                        assert_unknown_entity(
                            &view
                                .resolve(
                                    PersistentNameKey::Entity(later_n),
                                    &PersistentNameSelection::Exact,
                                )
                                .result,
                            later_n,
                        );
                    }
                })
                .unwrap();
        }
        (locs.len(), member_bytes, capture_alloc.alloc_bytes)
    };

    let _ = run(Some(20)); // warmup
    let (p0, b0, c0) = run(None);
    let (p4, b4, c4) = run(Some(10)); // 1x = 4 interleaved forks
    let (p40, b40, c40) = run(Some(1)); // 10x = 40 interleaved forks
    let capture_ratio = c40 as f64 / c4.max(1) as f64;
    let bytes_ratio = b40 as f64 / b4.max(1) as f64;
    eprintln!(
        "[CHK24-INTERLEAVED] 40x10 names: no_forks={p0}p/{b0}B (cap={c0}B), \
         1x(4 forks)={p4}p/{b4}B (cap={c4}B), \
         10x(40 forks)={p40}p/{b40}B (cap={c40}B) -> cap_ratio={capture_ratio:.3}x, bytes_ratio={bytes_ratio:.3}x"
    );
    assert!(
        capture_ratio < 3.5,
        "10x interleaved forks capture allocations must scale O(names + fork deltas), not Theta(branches * names): {capture_ratio:.3}x ({c4} B -> {c40} B)"
    );
    assert!(
        bytes_ratio < 2.75,
        "10x interleaved forks member bytes must stay sublinear: {bytes_ratio:.3}x ({b4} B -> {b40} B)"
    );
    assert!(
        (b4 as f64) < (b0 as f64) * 2.5,
        "4 interleaved forks must stay bounded vs no-fork baseline: {b0} B -> {b4} B"
    );
}
