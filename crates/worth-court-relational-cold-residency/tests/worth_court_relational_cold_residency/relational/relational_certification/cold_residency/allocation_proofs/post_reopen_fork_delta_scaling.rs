//! Layer 3 `#[global_allocator]` + byte/page scaling court for retained
//! historical forks created AFTER a cold reopen (`C1 -> reopen -> forks -> C2`).

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::{
    AggregateContinuityRule, AggregateGroupScheme, AggregateGroupSchemeId, AggregateMemberRole,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::lineage::{
    CommitNamingPlan, PersistentNameIdentity, PersistentNameKey, PersistentNameSelection,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::transactions::{
    EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot;
use worth_testkit_relational::cold_recovery::{
    capture_external_carrying, capture_external_checkpoint,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::field_patch;

use super::super::bounded_naming_fixture::{
    durable_runtime_with, naming_pages, naming_schema, observe_branch, publish_group_partition,
    reopen_with, GROUP_SCHEME,
};
use super::super::naming_branch_root_and_dissolve::fork_retained_at;
use super::fixture::create_named_batch_in_partition;
use super::fork_delta_checkpoint_scaling::{assert_exact_entity, assert_unknown_entity};
use super::{ensure_tracker_installed, measure};

struct PostReopenForkRun {
    c2_pages: usize,
    c2_member_bytes: usize,
    c2_capture_alloc_bytes: usize,
}

fn run_post_reopen_historical_forks(
    request: &ExecutionRequest<'_, '_>,
    fork_count: usize,
) -> PostReopenForkRun {
    let medium = TestDurableMedium::new();
    let producer = durable_runtime_with(&medium, u64::MAX / 4, naming_schema(true));
    let main = producer.main_branch_identity().branch_id().clone();

    let mut base_entities = Vec::new();
    let mut base_names = Vec::new();
    for b in 0..16usize {
        let keys: Vec<String> = (0..25).map(|i| format!("c1-base-{b:02}-{i:02}")).collect();
        let (e, n) =
            create_named_batch_in_partition(request, &producer, &main, PartitionId(1), &keys);
        base_entities.extend(e);
        base_names.extend(n);
    }

    let (c1, src1) = capture_external_checkpoint(request, &producer);
    drop(producer);
    let fresh1 = durable_runtime_with(&medium, u64::MAX / 4, naming_schema(true));
    let reopened = reopen_with(request, c1, src1.clone(), fresh1);

    // Advance `main` in small batches on the cold-reopened runtime while capturing
    // historical observations, then fork ALL retained historical branches BELOW head.
    let mut cutoffs = Vec::with_capacity(fork_count);
    for step in 0..fork_count {
        let basis = observe_branch(&reopened, &main);
        let keys = vec![
            format!("post-reopen-{step:02}-a"),
            format!("post-reopen-{step:02}-b"),
        ];
        let (e, n) =
            create_named_batch_in_partition(request, &reopened, &main, PartitionId(1), &keys);
        cutoffs.push((basis, e[0], n[0]));
    }

    let mut forks = Vec::with_capacity(fork_count);
    for (idx, (basis, post_e, post_n)) in cutoffs.iter().enumerate() {
        let target = BranchId(format!("post-c1-hist-{idx:02}"));
        fork_retained_at(request, &reopened, basis, &target).unwrap();
        forks.push((target, *post_e, *post_n));
    }

    let (_, cap_win) = measure(|| reopened.seal_naming_snapshot().unwrap());
    let (c2, src2) = capture_external_carrying(request, &reopened, &src1);
    let locs = naming_pages(&src2);
    let c2_member_bytes: usize = locs.iter().map(|l| src2.members[l].bytes().len()).sum();

    let fresh2 = durable_runtime_with(&medium, u64::MAX / 4, naming_schema(true));
    let reopened2 = reopen_with(request, c2, src2, fresh2);
    for (idx, (branch, _, post_n)) in forks.iter().enumerate() {
        let obs = observe_branch(&reopened2, branch);
        reopened2
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
                assert_unknown_entity(
                    &view
                        .resolve(
                            PersistentNameKey::Entity(*post_n),
                            &PersistentNameSelection::Exact,
                        )
                        .result,
                    *post_n,
                );
                if idx > 0 {
                    let (prev_e, prev_n) = (forks[idx - 1].1, forks[idx - 1].2);
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

    PostReopenForkRun {
        c2_pages: locs.len(),
        c2_member_bytes,
        c2_capture_alloc_bytes: cap_win.alloc_bytes,
    }
}

#[test]
fn post_reopen_retained_historical_forks_seal_as_sublinear_deltas_at_second_checkpoint() {
    ensure_tracker_installed();
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);

    let _ = run_post_reopen_historical_forks(request, 2); // warmup
    let small = run_post_reopen_historical_forks(request, 2);
    let large = run_post_reopen_historical_forks(request, 20);

    let page_ratio = large.c2_pages as f64 / small.c2_pages.max(1) as f64;
    let bytes_ratio = large.c2_member_bytes as f64 / small.c2_member_bytes.max(1) as f64;
    let cap_ratio =
        large.c2_capture_alloc_bytes as f64 / small.c2_capture_alloc_bytes.max(1) as f64;
    eprintln!(
        "[CHK24-POST-REOPEN-HIST] 1x(2 forks) vs 10x(20 forks) over 400 C1 names:\n  \
         c2_pages: {} -> {} ({page_ratio:.3}x)\n  \
         c2_member_bytes: {} -> {} ({bytes_ratio:.3}x)\n  \
         c2_capture_alloc_bytes: {} -> {} ({cap_ratio:.3}x)",
        small.c2_pages,
        large.c2_pages,
        small.c2_member_bytes,
        large.c2_member_bytes,
        small.c2_capture_alloc_bytes,
        large.c2_capture_alloc_bytes,
    );
    assert!(
        page_ratio < 3.0,
        "10x post-reopen historical forks C2 pages must not materialize full tables: {page_ratio:.3}x"
    );
    assert!(
        bytes_ratio < 1.8,
        "10x post-reopen historical forks C2 member bytes must scale O(names + fork deltas): {bytes_ratio:.3}x"
    );
    assert!(
        cap_ratio < 3.5,
        "10x post-reopen historical forks C2 capture allocs must scale sublinearly: {cap_ratio:.3}x"
    );
}

fn run_post_reopen_pre_scheme_b_forks(
    request: &ExecutionRequest<'_, '_>,
    fork_count: usize,
) -> (PostReopenForkRun, usize) {
    let medium = TestDurableMedium::new();
    let producer = durable_runtime_with(&medium, u64::MAX / 4, naming_schema(true));
    let main = producer.main_branch_identity().branch_id().clone();

    let mut base_entities = Vec::new();
    let mut base_names = Vec::new();
    for b in 0..16usize {
        let keys: Vec<String> = (0..25).map(|i| format!("psb-b{b:02}-{i:02}")).collect();
        let (e, n) =
            create_named_batch_in_partition(request, &producer, &main, PartitionId(1), &keys);
        base_entities.extend(e);
        base_names.extend(n);
    }
    let scheme_a_groups: Vec<Vec<(PersistentNameIdentity, AggregateMemberRole)>> = base_names
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
    publish_group_partition(
        request,
        &producer,
        &main,
        base_entities[0],
        "scheme-a-init",
        scheme_a_groups,
    );

    let (c1, src1) = capture_external_checkpoint(request, &producer);
    drop(producer);
    let fresh1 = durable_runtime_with(&medium, u64::MAX / 4, naming_schema(true));
    let reopened = reopen_with(request, c1, src1.clone(), fresh1);

    // Record `fork_count` historical cutoffs BEFORE adding scheme B on `main`.
    let mut pre_b_cutoffs = Vec::with_capacity(fork_count);
    for step in 0..fork_count {
        let basis = observe_branch(&reopened, &main);
        pre_b_cutoffs.push(basis);
        let keys = vec![format!("pre-b-{step:02}")];
        let _ = create_named_batch_in_partition(request, &reopened, &main, PartitionId(1), &keys);
    }

    // Add brand-new scheme B on `main` AFTER all recorded cutoffs.
    let scheme_b = "worth.postreopen.scheme.b.v1";
    let main_basis = observe_branch(&reopened, &main);
    let mut tx = reopened
        .begin_branch_transaction(&main_basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new("add-scheme-b").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: base_entities[0],
                fields: field_patch("name", "name", "add-scheme-b"),
            }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    let mut plan = CommitNamingPlan::default();
    plan.declare_group_partition(
        AggregateGroupScheme::new(
            AggregateGroupSchemeId::new(scheme_b).unwrap(),
            AggregateContinuityRule::Anchor,
        ),
        Vec::new(),
        vec![vec![
            (base_names[0], AggregateMemberRole::Anchor),
            (base_names[1], AggregateMemberRole::Ordinary),
        ]],
    );
    tx.attach_naming_plan(plan);
    let committed_b = tx.commit(request, &reopened).unwrap();
    release_test_commit_snapshot(&reopened, &committed_b);

    // Fork ALL historical branches predating scheme B!
    let mut forks = Vec::with_capacity(fork_count);
    for (idx, basis) in pre_b_cutoffs.iter().enumerate() {
        let target = BranchId(format!("pre-b-fork-{idx:02}"));
        fork_retained_at(request, &reopened, basis, &target).unwrap();
        forks.push(target);
    }

    let ((_, _, visits), cap_win) =
        measure(|| reopened.seal_naming_snapshot_with_visits().unwrap());
    let (c2, src2) = capture_external_carrying(request, &reopened, &src1);
    let locs = naming_pages(&src2);
    let c2_member_bytes: usize = locs.iter().map(|l| src2.members[l].bytes().len()).sum();

    let fresh2 = durable_runtime_with(&medium, u64::MAX / 4, naming_schema(true));
    let reopened2 = reopen_with(request, c2, src2, fresh2);
    let main_obs = observe_branch(&reopened2, &main);
    reopened2
        .with_persistent_names_at_observation(request, &main_obs.observation(), |view| {
            assert!(view.group_of_member(GROUP_SCHEME, &base_names[0]).is_some());
            assert!(view.group_of_member(scheme_b, &base_names[0]).is_some());
        })
        .unwrap();
    for br in [&forks[0], forks.last().unwrap()] {
        let obs = observe_branch(&reopened2, br);
        reopened2
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
                assert!(
                    view.group_of_member(GROUP_SCHEME, &base_names[0]).is_some(),
                    "pre-B fork must retain C1 scheme A"
                );
                assert_eq!(
                    view.group_of_member(scheme_b, &base_names[0]),
                    None,
                    "pre-B fork must NOT see post-cutoff scheme B"
                );
            })
            .unwrap();
    }

    (
        PostReopenForkRun {
            c2_pages: locs.len(),
            c2_member_bytes,
            c2_capture_alloc_bytes: cap_win.alloc_bytes,
        },
        visits,
    )
}

#[test]
fn post_reopen_historical_forks_predating_new_group_scheme_stay_delta_sealed() {
    ensure_tracker_installed();
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);

    let _ = run_post_reopen_pre_scheme_b_forks(request, 2);
    let (small, small_visits) = run_post_reopen_pre_scheme_b_forks(request, 2);
    let (large, large_visits) = run_post_reopen_pre_scheme_b_forks(request, 20);

    let page_ratio = large.c2_pages as f64 / small.c2_pages.max(1) as f64;
    let bytes_ratio = large.c2_member_bytes as f64 / small.c2_member_bytes.max(1) as f64;
    let cap_ratio =
        large.c2_capture_alloc_bytes as f64 / small.c2_capture_alloc_bytes.max(1) as f64;
    let visits_ratio = large_visits as f64 / small_visits.max(1) as f64;
    eprintln!(
        "[CHK24-PRE-SCHEME-B] 1x(2 forks) vs 10x(20 forks) predating new scheme B:\n  \
         c2_pages: {} -> {} ({page_ratio:.3}x)\n  \
         c2_member_bytes: {} -> {} ({bytes_ratio:.3}x)\n  \
         c2_capture_alloc_bytes: {} -> {} ({cap_ratio:.3}x)\n  \
         diff_visits: {} -> {} ({visits_ratio:.3}x)",
        small.c2_pages,
        large.c2_pages,
        small.c2_member_bytes,
        large.c2_member_bytes,
        small.c2_capture_alloc_bytes,
        large.c2_capture_alloc_bytes,
        small_visits,
        large_visits,
    );
    assert!(
        page_ratio < 3.0,
        "10x pre-scheme-B forks C2 pages must not flatten scheme A: {page_ratio:.3}x"
    );
    assert!(
        bytes_ratio < 1.8,
        "10x pre-scheme-B forks C2 member bytes must scale O(names + fork deltas): {bytes_ratio:.3}x"
    );
    assert!(
        cap_ratio < 3.5,
        "10x pre-scheme-B forks C2 capture allocs must scale sublinearly: {cap_ratio:.3}x"
    );
}
