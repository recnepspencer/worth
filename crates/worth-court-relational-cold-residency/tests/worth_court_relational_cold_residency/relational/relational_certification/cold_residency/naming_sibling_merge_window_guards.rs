//! Cold-residency courts for sibling-merge ancestor cutoff windowing and
//! checkpoint-order `k`-way `GROUP_LOG` traversal:
//! - Sibling merge skips ancestor `main` event pages and `GROUP_LOG` pages
//!   wholly after the source's fork cutoff.
//! - Checkpoint-order `k`-way `GROUP_LOG` traversal refuses `NamedGroupLineage`
//!   on an earlier readable ancestor page before touching a later unavailable
//!   source page.

use std::collections::BTreeSet;
use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
    RelationalNativeCheckpoint,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::merge::{
    MergeExecutionPreparationError, MergeExecutionRequest, MergeIntent,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::{
    CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch,
};
use worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot;
use worth_testkit_relational::cold_recovery::{
    capture_external_carrying, capture_external_checkpoint,
};
use worth_testkit_relational::durable_medium::immutable_members::ImmutableCheckpointMemberSource;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::field_patch;

use super::bounded_naming_fixture::{
    create_named_entities, group_scheme, observe_branch, publish_group_partition, GROUP_SCHEME,
};
use super::naming_branch_root_and_dissolve::fork_retained_at;
use super::naming_historical_fork_integrity::durable_mixed_runtime;

fn reopen_mixed(
    request: &ExecutionRequest<'_, '_>,
    medium: &Arc<TestDurableMedium>,
    ck: RelationalNativeCheckpoint,
    src: Arc<ImmutableCheckpointMemberSource>,
) -> RelationalRuntime {
    let mut rt = durable_mixed_runtime(medium);
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            ck.covered_position().unwrap(),
            ck.covered_branch_transitions().unwrap(),
            ck,
        )),
        Vec::new(),
        Vec::new(),
        src,
    );
    let plan = rt
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    rt.durability_recovery().recover(request, plan).unwrap();
    rt
}

fn commit_unnamed(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    label: &str,
) {
    let basis = observe_branch(runtime, branch);
    let mut tx = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new(label).push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(3),
                client_key: label.into(),
                fields: field_patch("name", "name", label),
            },
        ))),
        request.allocation_policy(),
    )
    .unwrap();
    let committed = tx.commit(request, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}

#[test]
fn sibling_merge_skips_ancestor_event_pages_past_source_fork_cutoff() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_mixed_runtime(&medium);
    let main = producer.main_branch_identity().branch_id().clone();

    commit_unnamed(request, &producer, &main, "r12-main-c1");
    let sibling1 = BranchId("r12-sibling-1".into());
    fork_retained_at(
        request,
        &producer,
        &observe_branch(&producer, &main),
        &sibling1,
    )
    .unwrap();

    commit_unnamed(request, &producer, &main, "r12-main-c2");
    let sibling2 = BranchId("r12-sibling-2".into());
    fork_retained_at(
        request,
        &producer,
        &observe_branch(&producer, &main),
        &sibling2,
    )
    .unwrap();

    let (ck1, src1) = capture_external_checkpoint(request, &producer);
    let ck1_event_members: BTreeSet<_> = super::selected_directory::read_native(&ck1)
        .lineage_events
        .iter()
        .map(|p| p.member)
        .collect();
    let mid = reopen_mixed(request, &medium, ck1, src1.clone());

    // C3 on main (strictly past sibling2's fork cutoff C2).
    commit_unnamed(request, &mid, &main, "r12-main-c3-beyond-sibling2");

    let (ck2, src2) = capture_external_carrying(request, &mid, &src1);
    let post_c2_main_members: BTreeSet<_> = super::selected_directory::read_native(&ck2)
        .lineage_events
        .iter()
        .map(|p| p.member)
        .filter(|m| !ck1_event_members.contains(m))
        .collect();
    assert!(
        !post_c2_main_members.is_empty(),
        "C2 must emit a distinct main event page wholly after sibling2's fork cutoff"
    );

    let reopened = reopen_mixed(request, &medium, ck2, src2.clone());
    *src2.failed_member.lock().unwrap() = Some(*post_c2_main_members.iter().next().unwrap());
    let reads_before = src2.member_reads.lock().unwrap().len();

    let bound = reopened
        .bind_merge_execution_request(MergeExecutionRequest::new(
            sibling1,
            sibling2,
            MergeIntent::ReconcileIntoTarget,
        ))
        .unwrap();
    reopened
        .prepare_merge_execution(request, bound)
        .expect("sibling merge must skip main event pages past source's fork cutoff");

    let post_c2_reads = src2.member_reads.lock().unwrap()[reads_before..]
        .iter()
        .filter(|m| post_c2_main_members.contains(m))
        .count();
    assert_eq!(post_c2_reads, 0);
}

#[test]
fn sibling_merge_skips_post_cutoff_main_group_log_pages() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_mixed_runtime(&medium);
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, entities, names) = create_named_entities(
        request,
        &producer,
        &main,
        &["r12-g-a".into(), "r12-g-b".into()],
    );
    let sibling1 = BranchId("r12-grp-sibling-1".into());
    fork_retained_at(
        request,
        &producer,
        &observe_branch(&producer, &main),
        &sibling1,
    )
    .unwrap();

    commit_unnamed(request, &producer, &main, "r12-grp-main-c2");
    let sibling2 = BranchId("r12-grp-sibling-2".into());
    fork_retained_at(
        request,
        &producer,
        &observe_branch(&producer, &main),
        &sibling2,
    )
    .unwrap();

    let (ck1, src1) = capture_external_checkpoint(request, &producer);
    let mid = reopen_mixed(request, &medium, ck1, src1.clone());

    let _ = publish_group_partition(
        request,
        &mid,
        &main,
        entities[0],
        "r12-main-post-c2-group",
        vec![vec![
            (names[0], AggregateMemberRole::Anchor),
            (names[1], AggregateMemberRole::Ordinary),
        ]],
    );

    let (ck2, src2) = capture_external_carrying(request, &mid, &src1);
    let post_c2_group_log_members: BTreeSet<_> = super::selected_directory::read_native(&ck2)
        .naming
        .group_log
        .iter()
        .filter(|p| p.first.first() == Some(&14))
        .map(|p| p.member)
        .collect();
    assert!(
        !post_c2_group_log_members.is_empty(),
        "C2 must emit a primary GROUP_LOG page for main's post-C2 group transition"
    );

    let reopened = reopen_mixed(request, &medium, ck2, src2.clone());
    *src2.failed_member.lock().unwrap() = Some(*post_c2_group_log_members.iter().next().unwrap());
    let reads_before = src2.member_reads.lock().unwrap().len();

    let bound = reopened
        .bind_merge_execution_request(MergeExecutionRequest::new(
            sibling1,
            sibling2,
            MergeIntent::ReconcileIntoTarget,
        ))
        .unwrap();
    reopened
        .prepare_merge_execution(request, bound)
        .expect("sibling merge must skip post-cutoff main GROUP_LOG pages before opening");

    let post_c2_group_reads = src2.member_reads.lock().unwrap()[reads_before..]
        .iter()
        .filter(|m| post_c2_group_log_members.contains(m))
        .count();
    assert_eq!(post_c2_group_reads, 0);
}

#[test]
fn earlier_ancestor_group_transition_refuses_before_later_unavailable_source_page() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_mixed_runtime(&medium);
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, _, names) = create_named_entities(
        request,
        &producer,
        &main,
        &["r14-a".into(), "r14-b".into(), "r14-c".into()],
    );
    let sibling1 = BranchId("r14-target".into());
    fork_retained_at(
        request,
        &producer,
        &observe_branch(&producer, &main),
        &sibling1,
    )
    .unwrap();

    // C2 on main: earlier group transition (paired with an unnamed KindId(3)
    // creation so merge readiness passes and reaches `named_outputs::refuse`).
    let commit_group = |branch: &BranchId, label: &str, members| {
        let basis = observe_branch(&producer, branch);
        let mut tx = producer
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        tx.push_batch(
            WorkerIntentBatch::new(label).push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(3),
                    client_key: label.into(),
                    fields: field_patch("name", "name", label),
                },
            ))),
            request.allocation_policy(),
        )
        .unwrap();
        let mut plan = worth_relational::facade::lineage::CommitNamingPlan::default();
        plan.declare_group_partition(group_scheme(), Vec::new(), vec![members]);
        tx.attach_naming_plan(plan);
        let committed = tx.commit(request, &producer).unwrap();
        release_test_commit_snapshot(&producer, &committed);
        committed.commit.commit_id
    };
    let c2_at = commit_group(
        &main,
        "r14-main-group",
        vec![
            (names[0], AggregateMemberRole::Anchor),
            (names[1], AggregateMemberRole::Ordinary),
        ],
    );
    let sibling2 = BranchId("r14-source".into());
    fork_retained_at(
        request,
        &producer,
        &observe_branch(&producer, &main),
        &sibling2,
    )
    .unwrap();

    // C3 on sibling2: later group transition on a separate single-branch GROUP_LOG page.
    let _ = commit_group(
        &sibling2,
        "r14-source-group",
        vec![
            (names[0], AggregateMemberRole::Anchor),
            (names[2], AggregateMemberRole::Ordinary),
        ],
    );

    let (ck, src) = capture_external_checkpoint(request, &producer);
    let primary_pages: Vec<_> = super::selected_directory::read_native(&ck)
        .naming
        .group_log
        .into_iter()
        .filter(|p| p.first.first() == Some(&14))
        .collect();
    assert_eq!(
        primary_pages.len(),
        2,
        "expected separate main and source GROUP_LOG pages"
    );
    let later_source_page = primary_pages[1].member;

    let reopened = reopen_mixed(request, &medium, ck, src.clone());
    // Mark sibling2's later GROUP_LOG page unavailable: checkpoint-order merge
    // must hit main's earlier readable page and return NamedGroupLineage first.
    *src.failed_member.lock().unwrap() = Some(later_source_page);

    let bound = reopened
        .bind_merge_execution_request(MergeExecutionRequest::new(
            sibling1,
            sibling2.clone(),
            MergeIntent::ReconcileIntoTarget,
        ))
        .unwrap();
    let err = reopened
        .prepare_merge_execution(request, bound)
        .expect_err("earlier ancestor group transition must refuse NamedGroupLineage");
    assert_eq!(
        err,
        MergeExecutionPreparationError::NamedGroupLineage(sibling2, c2_at, GROUP_SCHEME.into())
    );
}
