//! Round 2 hardening courts for Store M4.3f (`[P1-3]`, `[P1-4]`, `[P1-5]`, `[P1-8]`):
//! - Fresh-runtime C1 ACK preserves historical group on checkout and fork.
//! - Retirement between C1 capture and C1 ACK does not reappear in C2.
//! - Cold historical fork with inherited groups survives child checkpoint, ACK,
//!   reopen, and subsequent group edit.
//! - Retention floor > head refuses cleanly; fork below floor refuses even
//!   without a later batch on the source index.
use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalCheckpointArtifactPolicy, RelationalCheckpointMember,
    RelationalDurableColdSource,
};
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::lineage::{
    CommitNamingPlan, PersistentNameIndexDenial, PersistentNameKey, PersistentNameResolution,
    PersistentNameSelection,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::transactions::{
    EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::field_patch;

use super::bounded_naming_fixture::{
    create_named_entities, delete_entity, durable_named_runtime, group_scheme, observe_branch,
    publish_group_partition, GROUP_SCHEME,
};
use super::bounded_naming_r2::fork_retained_at;

fn recover_from_medium(
    request: &ExecutionRequest<'_, '_>,
    medium: &Arc<TestDurableMedium>,
) -> worth_relational::facade::runtime::RelationalRuntime {
    let mut rt = durable_named_runtime(medium, u64::MAX / 4);
    let plan = rt
        .durability()
        .recovery_plan_from_source(
            request,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    rt.durability_recovery().recover(request, plan).unwrap();
    rt
}

fn capture_and_file(
    request: &ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    medium: &Arc<TestDurableMedium>,
    prior_members: &[worth_relational::facade::durability::RelationalDurableMemberLocator],
) -> worth_relational::facade::durability::RelationalNativeCheckpoint {
    let source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    let ck = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: prior_members,
                source: Some(source),
            },
        )
        .unwrap();
    medium.file_checkpoint(ck.clone());
    ck
}

fn ack(
    request: &ExecutionRequest<'_, '_>,
    ck: &worth_relational::facade::durability::RelationalNativeCheckpoint,
) {
    request
        .run_native_with_checkpoint(|context| ck.acknowledge_selected(request, context))
        .unwrap()
        .0
        .unwrap();
}

#[test]
fn fresh_runtime_c1_ack_preserves_historical_group_on_checkout_and_fork() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_named_runtime(&medium, u64::MAX / 4);
    let main = runtime.main_branch_identity().branch_id().clone();

    let (_, e1, n1) = create_named_entities(
        request,
        &runtime,
        &main,
        &["ackg-1".into(), "ackg-2".into()],
    );
    publish_group_partition(
        request,
        &runtime,
        &main,
        e1[0],
        "ackg-c1",
        vec![vec![
            (n1[0], AggregateMemberRole::Anchor),
            (n1[1], AggregateMemberRole::Ordinary),
        ]],
    );
    let basis_c1 = observe_branch(&runtime, &main);
    let g1 = match runtime
        .with_persistent_names_at_observation(request, &basis_c1.observation(), |v| {
            v.group_of_member(GROUP_SCHEME, &n1[0]).unwrap()
        })
        .unwrap()
    {
        PersistentNameKey::Group(id) => id,
        other => panic!("expected group key, got {other:?}"),
    };

    let c1 = capture_and_file(request, &runtime, &medium, &[]);
    ack(request, &c1);

    // Dissolve the group at C2 after ACK.
    let basis_now = observe_branch(&runtime, &main);
    let mut tx = runtime
        .begin_branch_transaction(&basis_now, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new("dissolve-c2").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: e1[0],
                fields: field_patch("name", "name", "dissolve-c2"),
            }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    let mut plan = CommitNamingPlan::default();
    plan.declare_group_partition(group_scheme(), vec![g1], Vec::new());
    tx.attach_naming_plan(plan);
    tx.commit(request, &runtime).unwrap();

    // Historical observation at C1 AND historical fork at C1 must both preserve g1!
    runtime
        .with_persistent_names_at_observation(request, &basis_c1.observation(), |v| {
            assert_eq!(
                v.group_of_member(GROUP_SCHEME, &n1[0]),
                Some(PersistentNameKey::Group(g1))
            );
        })
        .unwrap();
    let fork_c1 = BranchId("fork-at-c1".into());
    fork_retained_at(request, &runtime, &basis_c1, &fork_c1)
        .expect("historical fork at C1 after ACK must succeed");
    let fork_obs = observe_branch(&runtime, &fork_c1);
    runtime
        .with_persistent_names_at_observation(request, &fork_obs.observation(), |v| {
            assert_eq!(
                v.group_of_member(GROUP_SCHEME, &n1[0]),
                Some(PersistentNameKey::Group(g1))
            );
            assert_eq!(
                v.group_members(PersistentNameKey::Group(g1)).unwrap().len(),
                2
            );
        })
        .unwrap();
    let (_, ef, nf) = create_named_entities(request, &runtime, &fork_c1, &["ackg-3".into()]);
    publish_group_partition(
        request,
        &runtime,
        &fork_c1,
        ef[0],
        "ackg-fork-edit",
        vec![vec![
            (n1[0], AggregateMemberRole::Anchor),
            (n1[1], AggregateMemberRole::Ordinary),
            (nf[0], AggregateMemberRole::Ordinary),
        ]],
    );
    let fork_after = observe_branch(&runtime, &fork_c1);
    runtime
        .with_persistent_names_at_observation(request, &fork_after.observation(), |v| {
            assert_eq!(
                v.group_of_member(GROUP_SCHEME, &nf[0]),
                Some(PersistentNameKey::Group(g1)),
                "anchor continuity must keep g1 identity on fork after C1 ACK"
            );
        })
        .unwrap();
}

#[test]
fn retire_between_c1_capture_and_ack_does_not_reappear_in_c2() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_named_runtime(&medium, u64::MAX / 4);
    let main = runtime.main_branch_identity().branch_id().clone();

    let (_, e1, n1) = create_named_entities(
        request,
        &runtime,
        &main,
        &["keep-1".into(), "retire-mid".into()],
    );
    let c1 = capture_and_file(request, &runtime, &medium, &[]);
    // Retire between C1 capture and C1 ACK (branch still pure resident).
    delete_entity(request, &runtime, &main, e1[1]);
    ack(request, &c1);

    let c1_members: Vec<_> = c1
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect();
    let c2 = capture_and_file(request, &runtime, &medium, &c1_members);
    ack(request, &c2);

    let reopened = recover_from_medium(request, &medium);
    let live = reopened.branch_live_persistent_names(&main).unwrap();
    assert!(live.contains_key(&n1[0]));
    assert!(
        !live.contains_key(&n1[1]),
        "name retired between C1 capture and C1 ACK must NOT reappear in C2"
    );
}

#[test]
fn cold_historical_fork_with_inherited_groups_survives_child_checkpoint_reopen_and_edit() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, e1, n1) =
        create_named_entities(request, &producer, &main, &["chf-1".into(), "chf-2".into()]);
    publish_group_partition(
        request,
        &producer,
        &main,
        e1[0],
        "chf-g1",
        vec![vec![
            (n1[0], AggregateMemberRole::Anchor),
            (n1[1], AggregateMemberRole::Ordinary),
        ]],
    );
    let c1 = capture_and_file(request, &producer, &medium, &[]);
    drop(producer);

    let reopened = recover_from_medium(request, &medium);
    let retained = observe_branch(&reopened, &main);
    let _ = create_named_entities(request, &reopened, &main, &["chf-later-only-name".into()]);

    let child = BranchId("cold-hist-child".into());
    fork_retained_at(request, &reopened, &retained, &child).unwrap();

    let c1_members: Vec<_> = c1
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect();
    let c2 = capture_and_file(request, &reopened, &medium, &c1_members);
    ack(request, &c2);
    drop(reopened);

    let reopened2 = recover_from_medium(request, &medium);
    let child_obs = observe_branch(&reopened2, &child);
    let g1 = reopened2
        .with_persistent_names_at_observation(request, &child_obs.observation(), |v| {
            let g = v
                .group_of_member(GROUP_SCHEME, &n1[0])
                .expect("inherited group row must survive child checkpoint + reopen");
            assert_eq!(v.group_members(g).unwrap().len(), 2);
            g
        })
        .unwrap();
    let (_, ec, nc) = create_named_entities(request, &reopened2, &child, &["chf-3".into()]);
    publish_group_partition(
        request,
        &reopened2,
        &child,
        ec[0],
        "chf-edit",
        vec![vec![
            (n1[0], AggregateMemberRole::Anchor),
            (n1[1], AggregateMemberRole::Ordinary),
            (nc[0], AggregateMemberRole::Ordinary),
        ]],
    );
    let child_after = observe_branch(&reopened2, &child);
    reopened2
        .with_persistent_names_at_observation(request, &child_after.observation(), |v| {
            assert_eq!(v.group_of_member(GROUP_SCHEME, &nc[0]), Some(g1));
        })
        .unwrap();
}

#[test]
fn retention_floor_above_head_refuses_and_fork_below_floor_refuses_without_later_batch() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_named_runtime(&medium, u64::MAX / 4);
    let main = runtime.main_branch_identity().branch_id().clone();

    let (_, e1, n1) = create_named_entities(request, &runtime, &main, &["rf-1".into()]);
    let basis_c1 = observe_branch(&runtime, &main);
    let (_, _, _) = create_named_entities(request, &runtime, &main, &["rf-2".into()]);
    let basis_c2 = observe_branch(&runtime, &main);
    // Commit an unnamed field update at c3 so branch_head (c3) > index.head() (c2).
    let mut tx = runtime
        .begin_branch_transaction(&basis_c2, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new("rf-upd").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: e1[0],
                fields: field_patch("name", "name", "rf-1-updated"),
            }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    let c3 = tx.commit(request, &runtime).unwrap();

    let err = runtime
        .advance_naming_retention_floor(&main, CommitId(c3.commit.commit_id.0 + 1))
        .expect_err("retention floor > branch head must refuse");
    assert!(matches!(
        err,
        PersistentNameIndexDenial::RetentionFloorExceedsHead { .. }
    ));
    runtime
        .with_persistent_names_at_observation(request, &basis_c1.observation(), |v| {
            assert!(matches!(
                v.resolve(
                    PersistentNameKey::Entity(n1[0]),
                    &PersistentNameSelection::Exact,
                )
                .result,
                Ok(PersistentNameResolution::Exact(_))
            ));
        })
        .expect("head and C1 remain unpruned after refused floor");

    // Advance floor to c3 (unnamed head > index.head() == c2), then attempt a
    // retained fork at c2 (where index.head() == c2 <= fork_commit < floor)!
    runtime
        .advance_naming_retention_floor(&main, c3.commit.commit_id)
        .unwrap();
    let denied = fork_retained_at(
        request,
        &runtime,
        &basis_c2,
        &BranchId("pruned-fork".into()),
    );
    assert!(
        denied.is_err(),
        "fork at cutoff < retention_floor must refuse even when index.head() <= fork_commit"
    );
    assert!(runtime
        .branch_identity(&BranchId("pruned-fork".into()))
        .is_err());

    // Checkpoint + reopen with index.head() < retention_floor <= branch_head must succeed cleanly.
    let ck = capture_and_file(request, &runtime, &medium, &[]);
    ack(request, &ck);
    let reopened = recover_from_medium(request, &medium);
    let head_obs = observe_branch(&reopened, &main);
    reopened
        .with_persistent_names_at_observation(request, &head_obs.observation(), |v| {
            assert!(matches!(
                v.resolve(
                    PersistentNameKey::Entity(n1[0]),
                    &PersistentNameSelection::Exact,
                )
                .result,
                Ok(PersistentNameResolution::Exact(_))
            ));
        })
        .expect("reopened head >= retention_floor must still resolve cleanly");
}
