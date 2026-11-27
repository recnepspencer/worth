//! Cold-residency courts for merge preparation refusal and pre-base page
//! skipping across cold naming and `GROUP_LOG` storage:
//! - Merge preparation still refuses `NamedGroupLineage` after checkpoint ACK
//!   trims the resident group log and after cold reopen.
//! - Merge preparation still refuses `NamedLineage` after cold reopen when a
//!   keep-in-place source merge's lineage event lives only in cold storage.
//! - Merge preparation skips pre-base event and `GROUP_LOG` pages after reopen.

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::lineage::{
    CommitNamingPlan, LineageMergeKeeper, LineagePostureClaim, LineageRecordTarget,
};
use worth_relational::facade::merge::{
    MergeExecutionPreparationError, MergeExecutionRequest, MergeIntent,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::{
    CreateIntent, DeleteEntityIntent, EntityMutationIntent, EntitySpec, MutationIntent,
    WorkerIntentBatch,
};
use worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot;
use worth_testkit_relational::cold_recovery::{
    capture_external_carrying, capture_external_checkpoint,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::field_patch;

use super::bounded_naming_fixture::{
    create_named_entities, delete_entity, durable_named_runtime, group_scheme, observe_branch,
    publish_group_partition, reopen_named, GROUP_SCHEME,
};
use super::naming_ack_isolation_and_rebind::{capture_only, try_ack};
use super::naming_branch_root_and_dissolve::fork_retained_at;
use super::naming_historical_fork_integrity::durable_mixed_runtime;

fn prepare_merge_feature_into_main(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: &BranchId,
) -> Result<(), MergeExecutionPreparationError> {
    let target = runtime.main_branch_identity().branch_id().clone();
    let bound = runtime
        .bind_merge_execution_request(MergeExecutionRequest::new(
            target,
            source.clone(),
            MergeIntent::ReconcileIntoTarget,
        ))
        .unwrap();
    runtime.prepare_merge_execution(request, bound).map(|_| ())
}

#[test]
fn merge_preparation_refuses_named_group_lineage_after_checkpoint_ack_and_cold_reopen() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_mixed_runtime(&medium);
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, _, names) = create_named_entities(
        request,
        &producer,
        &main,
        &["r10-g-a".into(), "r10-g-b".into()],
    );
    let feature = BranchId("r10-feature-group".into());
    let main_obs = observe_branch(&producer, &main);
    fork_retained_at(request, &producer, &main_obs, &feature).unwrap();

    // Publish group lineage on `feature` using members named BEFORE the fork,
    // paired with an unnamed adoption (KindId(3)) so merge preparation reaches `named_outputs::refuse`.
    let feature_obs = observe_branch(&producer, &feature);
    let mut tx = producer
        .begin_branch_transaction(&feature_obs, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new("r10-feature-group-commit").push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(3),
                client_key: "r10-unnamed-part".into(),
                fields: field_patch("name", "name", "r10-unnamed-part"),
            }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    let mut plan = CommitNamingPlan::default();
    plan.declare_group_partition(
        group_scheme(),
        Vec::new(),
        vec![vec![
            (names[0], AggregateMemberRole::Anchor),
            (names[1], AggregateMemberRole::Ordinary),
        ]],
    );
    tx.attach_naming_plan(plan);
    let grouped = tx.commit(request, &producer).unwrap();
    let grouped_at = grouped.commit.commit_id;
    release_test_commit_snapshot(&producer, &grouped);

    // 1. ACK checkpoint on producer (trims covered resident published_group_lineage).
    let ck_ack = capture_only(request, &producer, &medium, &[]);
    medium.file_checkpoint(ck_ack.clone());
    try_ack(request, &ck_ack).unwrap();

    let ack_err = prepare_merge_feature_into_main(request, &producer, &feature)
        .expect_err("ACKed runtime must still refuse NamedGroupLineage from cold GROUP_LOG");
    assert_eq!(
        ack_err,
        MergeExecutionPreparationError::NamedGroupLineage(
            feature.clone(),
            grouped_at,
            GROUP_SCHEME.into(),
        )
    );

    // 2. Cold reopen starts with empty resident published_group_lineage.
    let (ck, source) = capture_external_checkpoint(request, &producer);
    let mut reopened = durable_mixed_runtime(&medium);
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            ck.covered_position().unwrap(),
            ck.covered_branch_transitions().unwrap(),
            ck,
        )),
        Vec::new(),
        Vec::new(),
        source,
    );
    let plan = reopened
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    reopened
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    let reopen_err = prepare_merge_feature_into_main(request, &reopened, &feature).expect_err(
        "cold-reopened runtime must still refuse NamedGroupLineage from cold GROUP_LOG",
    );
    assert_eq!(
        reopen_err,
        MergeExecutionPreparationError::NamedGroupLineage(feature, grouped_at, GROUP_SCHEME.into(),)
    );
}

#[test]
fn merge_preparation_refuses_cold_named_lineage_after_reopen() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, entities, names) = create_named_entities(
        request,
        &producer,
        &main,
        &["r10-keeper".into(), "r10-absorbed".into()],
    );
    let (keeper, absorbed) = (entities[0], entities[1]);
    let (kept_name, gone_name) = (names[0], names[1]);

    let feature = BranchId("r10-feature-merge".into());
    let main_obs = observe_branch(&producer, &main);
    fork_retained_at(request, &producer, &main_obs, &feature).unwrap();

    // Retire `absorbed` on main so target already deleted it, while `feature`
    // publishes a keep-in-place Merge (absorbed -> keeper) with no new record to adopt.
    delete_entity(request, &producer, &main, absorbed);

    let feature_basis = observe_branch(&producer, &feature);
    let mut tx = producer
        .begin_branch_transaction(&feature_basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new("r10-keep-in-place-merge").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent {
                entity_id: absorbed,
            }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    tx.attach_naming_plan(CommitNamingPlan {
        postures: vec![LineagePostureClaim::Merge {
            parents: vec![
                (LineageRecordTarget::Entity(keeper), kept_name),
                (LineageRecordTarget::Entity(absorbed), gone_name),
            ],
            target: LineageRecordTarget::Entity(keeper),
            keeper: LineageMergeKeeper::KeepParent(kept_name),
        }],
        ..CommitNamingPlan::default()
    });
    let committed = tx.commit(request, &producer).unwrap();
    let merged_at = committed.commit.commit_id;
    release_test_commit_snapshot(&producer, &committed);

    // Cold reopen leaves the keep-in-place Merge event strictly in cold paged event storage.
    let (ck, source) = capture_external_checkpoint(request, &producer);
    let reopened = reopen_named(request, ck, source, &medium, u64::MAX / 4);
    let err = prepare_merge_feature_into_main(request, &reopened, &feature)
        .expect_err("cold-reopened merge preparation must refuse cold NamedLineage");
    assert_eq!(
        err,
        MergeExecutionPreparationError::NamedLineage(feature, merged_at)
    );
}

#[test]
fn merge_preparation_skips_pre_base_event_and_group_log_pages_after_reopen() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_mixed_runtime(&medium);
    let main = producer.main_branch_identity().branch_id().clone();

    // Populate pre-base named lineage events AND a pre-base GROUP_LOG partition on main.
    let (_, entities, names) = create_named_entities(
        request,
        &producer,
        &main,
        &["r11-pre-a".into(), "r11-pre-b".into()],
    );
    let _ = publish_group_partition(
        request,
        &producer,
        &main,
        entities[0],
        "r11-pre-group",
        vec![vec![
            (names[0], AggregateMemberRole::Anchor),
            (names[1], AggregateMemberRole::Ordinary),
        ]],
    );

    // Seal main's pre-base event & GROUP_LOG pages in C1 and reopen into `mid`
    // so C2 carries C1's pre-base pages separately from feature's post-base commit.
    let (ck1, src1) = capture_external_checkpoint(request, &producer);
    let pre_base_dir = super::selected_directory::read_native(&ck1);
    assert!(
        !pre_base_dir.lineage_events.is_empty(),
        "fixture must emit cold pre-base lineage event pages"
    );
    let pre_base_event_members: std::collections::BTreeSet<_> = pre_base_dir
        .lineage_events
        .iter()
        .map(|page| page.member)
        .collect();
    let pre_base_group_log_members: std::collections::BTreeSet<_> = pre_base_dir
        .naming
        .group_log
        .iter()
        .map(|page| page.member)
        .collect();
    assert!(
        !pre_base_group_log_members.is_empty(),
        "fixture must emit cold pre-base GROUP_LOG pages"
    );
    let mut mid = durable_mixed_runtime(&medium);
    let mid_input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            ck1.covered_position().unwrap(),
            ck1.covered_branch_transitions().unwrap(),
            ck1,
        )),
        Vec::new(),
        Vec::new(),
        src1.clone(),
    );
    let mid_plan = mid
        .durability()
        .recovery_plan_from_source(
            request,
            mid_input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    mid.durability_recovery()
        .recover(request, mid_plan)
        .unwrap();

    // Fork feature at the merge base and commit only an unnamed record (KindId(3)).
    let feature = BranchId("r11-feature-unnamed".into());
    let main_obs = observe_branch(&mid, &main);
    fork_retained_at(request, &mid, &main_obs, &feature).unwrap();

    let feature_obs = observe_branch(&mid, &feature);
    let mut tx = mid
        .begin_branch_transaction(&feature_obs, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new("r11-unnamed-commit").push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(3),
                client_key: "r11-unnamed-only".into(),
                fields: field_patch("name", "name", "r11-unnamed-only"),
            }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    let committed = tx.commit(request, &mid).unwrap();
    release_test_commit_snapshot(&mid, &committed);

    let (ck, source) = capture_external_carrying(request, &mid, &src1);
    let directory = super::selected_directory::read_native(&ck);
    assert!(
        directory.lineage_events.len() > pre_base_event_members.len(),
        "C2 must carry pre-base main pages alongside feature's post-base event page"
    );

    let mut reopened = durable_mixed_runtime(&medium);
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            ck.covered_position().unwrap(),
            ck.covered_branch_transitions().unwrap(),
            ck,
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    let plan = reopened
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    reopened
        .durability_recovery()
        .recover(request, plan)
        .unwrap();

    let reads_before = source.member_reads.lock().unwrap().len();

    prepare_merge_feature_into_main(request, &reopened, &feature)
        .expect("unnamed post-base merge must prepare cleanly without reading pre-base pages");

    let new_reads = &source.member_reads.lock().unwrap()[reads_before..];
    let new_pre_base_event_reads = new_reads
        .iter()
        .filter(|locator| pre_base_event_members.contains(locator))
        .count();
    let new_pre_base_group_log_reads = new_reads
        .iter()
        .filter(|locator| pre_base_group_log_members.contains(locator))
        .count();
    assert_eq!(
        new_pre_base_event_reads, 0,
        "merge preparation must not read pre-base lineage event pages"
    );
    assert_eq!(
        new_pre_base_group_log_reads, 0,
        "merge preparation must not read pre-base GROUP_LOG pages"
    );
}
