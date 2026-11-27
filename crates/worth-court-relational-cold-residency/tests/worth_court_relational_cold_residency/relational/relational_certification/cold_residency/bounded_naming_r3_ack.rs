//! Round 3 hardening courts for Store M4.3f (`[P1-4]`, `[P1-5]`, `[P1-6]`, `[P1-8]`):
//! - `[P1-4]` Detached runtime (`RelationalRuntime::fork`) isolates its cold
//!   naming source from later checkpoints acknowledged on the original runtime.
//! - `[P1-5]` Concurrent `advance_naming_retention_floor` vs `fork_retained_branch`
//!   coordination never durably publishes a child whose naming state is pruned.
//! - `[P1-6]` Equal-coverage ACK cannot roll back a persisted `retention_floor`.
//! - `[P1-8]` Post-capture fork before C2 ACK rebinds onto its nearest C2
//!   ancestor so parent deletions in C2 cannot bleed into the descendant.

use std::sync::{Arc, Barrier};
use std::thread;

use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalCheckpointAcknowledgementDenial,
    RelationalCheckpointArtifactPolicy, RelationalCheckpointMember, RelationalDurableColdSource,
    RelationalNativeCheckpoint,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::lineage::{
    PersistentNameKey, PersistentNameResolution, PersistentNameResolutionDenial,
    PersistentNameSelection,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;

use super::bounded_naming_fixture::{
    create_named_entities, delete_entity, durable_named_runtime, observe_branch,
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

pub(super) fn capture_only(
    request: &ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    medium: &Arc<TestDurableMedium>,
    prior_members: &[worth_relational::facade::durability::RelationalDurableMemberLocator],
) -> RelationalNativeCheckpoint {
    let source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: prior_members,
                source: Some(source),
            },
        )
        .unwrap()
}

pub(super) fn try_ack(
    request: &ExecutionRequest<'_, '_>,
    ck: &RelationalNativeCheckpoint,
) -> Result<(), RelationalCheckpointAcknowledgementDenial> {
    request
        .run_native_with_checkpoint(|context| ck.acknowledge_selected(request, context))
        .unwrap()
        .0
}

pub(super) fn ack_retry_busy(request: &ExecutionRequest<'_, '_>, ck: &RelationalNativeCheckpoint) {
    loop {
        match try_ack(request, ck) {
            Ok(()) => return,
            Err(RelationalCheckpointAcknowledgementDenial::Busy) => thread::yield_now(),
            Err(other) => panic!("unexpected checkpoint ACK denial: {other:?}"),
        }
    }
}

#[test]
fn detached_runtime_fork_cold_naming_isolated_from_later_parent_checkpoints() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_named_runtime(&medium, u64::MAX / 4);
    let main = runtime.main_branch_identity().branch_id().clone();

    let (_, _, n1) = create_named_entities(request, &runtime, &main, &["det-1".into()]);
    let ck1 = capture_only(request, &runtime, &medium, &[]);
    medium.file_checkpoint(ck1.clone());
    try_ack(request, &ck1).unwrap();

    // Fork a detached runtime snapshot bound to C1.
    let detached = runtime.fork().unwrap();

    // Advance the original runtime with a new name and acknowledge C2.
    let (_, _, n2) = create_named_entities(request, &runtime, &main, &["det-2".into()]);
    let ck1_members: Vec<_> = ck1
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect();
    let ck2 = capture_only(request, &runtime, &medium, &ck1_members);
    medium.file_checkpoint(ck2.clone());
    try_ack(request, &ck2).unwrap();

    // Detached runtime must see ONLY C1's names and NEVER bleed C2's name!
    let detached_live = detached.branch_live_persistent_names(&main).unwrap();
    assert!(detached_live.contains_key(&n1[0]));
    assert!(
        !detached_live.contains_key(&n2[0]),
        "detached runtime fork must not observe names from parent's later acknowledged checkpoint"
    );
    let det_obs = observe_branch(&detached, &main);
    detached
        .with_persistent_names_at_observation(request, &det_obs.observation(), |v| {
            assert!(matches!(
                v.resolve(
                    PersistentNameKey::Entity(n1[0]),
                    &PersistentNameSelection::Exact,
                )
                .result,
                Ok(PersistentNameResolution::Exact(_))
            ));
            assert!(v
                .resolve(
                    PersistentNameKey::Entity(n2[0]),
                    &PersistentNameSelection::Exact,
                )
                .result
                .is_err());
        })
        .unwrap();
}

#[test]
fn concurrent_retention_floor_raise_and_historical_fork_never_publishes_pruned_child() {
    for round in 0..12usize {
        let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
        let request = &ExecutionRequest::serial(&host);
        let medium = TestDurableMedium::new();
        let runtime = Arc::new(durable_named_runtime(&medium, u64::MAX / 4));
        let main = runtime.main_branch_identity().branch_id().clone();

        let (_, _, n1) =
            create_named_entities(request, &runtime, &main, &[format!("race-1-{round}")]);
        let basis_c1 = observe_branch(&runtime, &main);
        let (c2, _, _) =
            create_named_entities(request, &runtime, &main, &[format!("race-2-{round}")]);
        let c2_id = c2.commit.commit_id;
        let child = BranchId(format!("race-child-{round}"));

        let barrier = Arc::new(Barrier::new(2));
        let rt_floor = Arc::clone(&runtime);
        let main_floor = main.clone();
        let b_floor = Arc::clone(&barrier);
        let floor_thread = thread::spawn(move || {
            b_floor.wait();
            rt_floor.advance_naming_retention_floor(&main_floor, c2_id)
        });

        let rt_fork = Arc::clone(&runtime);
        let child_fork = child.clone();
        let b_fork = Arc::clone(&barrier);
        let fork_thread = thread::spawn(move || {
            let h = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
            let req = ExecutionRequest::serial(&h);
            b_fork.wait();
            fork_retained_at(&req, &rt_fork, &basis_c1, &child_fork)
        });

        let floor_res = floor_thread.join().unwrap();
        let fork_res = fork_thread.join().unwrap();
        assert!(
            floor_res.is_ok(),
            "round {round}: floor raise must succeed: {floor_res:?}"
        );

        match fork_res {
            Ok(()) => {
                // Fork won coordination before floor rose: child MUST have intact naming at C1.
                let child_obs = observe_branch(&runtime, &child);
                runtime
                    .with_persistent_names_at_observation(request, &child_obs.observation(), |v| {
                        assert!(matches!(
                            v.resolve(
                                PersistentNameKey::Entity(n1[0]),
                                &PersistentNameSelection::Exact,
                            )
                            .result,
                            Ok(PersistentNameResolution::Exact(_))
                        ));
                    })
                    .expect("published child must never have BasisPruned/Unavailable naming");
                let ck = capture_only(request, &runtime, &medium, &[]);
                medium.file_checkpoint(ck);
                let reopened = recover_from_medium(request, &medium);
                assert!(reopened
                    .branch_live_persistent_names(&child)
                    .unwrap()
                    .contains_key(&n1[0]));
            }
            Err(_) => {
                // Floor won coordination before fork cutover: child MUST NOT exist live or durably!
                assert!(
                    runtime.branch_identity(&child).is_err(),
                    "round {round}: refused fork must not install child branch"
                );
                let reopened = recover_from_medium(request, &medium);
                assert!(
                    reopened.branch_identity(&child).is_err(),
                    "round {round}: refused fork must not have durably appended branch transition"
                );
            }
        }
    }
}

#[test]
fn equal_coverage_ack_cannot_regress_persisted_retention_floor() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_named_runtime(&medium, u64::MAX / 4);
    let main = runtime.main_branch_identity().branch_id().clone();

    let (_, _, n1) = create_named_entities(request, &runtime, &main, &["eq-1".into()]);
    let basis_c1 = observe_branch(&runtime, &main);
    let (c2, _, _) = create_named_entities(request, &runtime, &main, &["eq-2".into()]);
    let c2_id = c2.commit.commit_id;

    // Capture ck_low (retention_floor = None) WITHOUT acknowledging it yet.
    let ck_low = capture_only(request, &runtime, &medium, &[]);

    // Raise retention_floor without any new commits -> equal commit coverage!
    runtime
        .advance_naming_retention_floor(&main, c2_id)
        .unwrap();
    let ck_high = capture_only(request, &runtime, &medium, &[]);
    assert_eq!(ck_low.covered_position(), ck_high.covered_position());
    medium.file_checkpoint(ck_high.clone());
    try_ack(request, &ck_high).unwrap();

    // Attempting an out-of-order ACK of ck_low (equal coverage, regressed
    // retention_floor None < Some(c2_id)) MUST refuse StaleCapture.
    medium.file_checkpoint(ck_low.clone());
    let err = try_ack(request, &ck_low)
        .expect_err("equal-coverage ACK with lower retention_floor must refuse StaleCapture");
    assert!(
        matches!(err, RelationalCheckpointAcknowledgementDenial::StaleCapture),
        "expected StaleCapture, got {err:?}"
    );
    let high_members: Vec<_> = ck_high
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect();
    medium.file_checkpoint(ck_high);

    // Live runtime still enforces c2_id as the retention floor after refused stale ACK.
    runtime
        .with_persistent_names_at_observation(request, &basis_c1.observation(), |v| {
            assert_eq!(
                v.resolve(
                    PersistentNameKey::Entity(n1[0]),
                    &PersistentNameSelection::Exact,
                )
                .result,
                Err(PersistentNameResolutionDenial::BasisPruned {
                    basis: basis_c1.observation().commit_id().unwrap(),
                    retention_floor: c2_id,
                })
            );
        })
        .unwrap();

    // Reopened runtime retains c2_id as its persisted retention floor.
    let reopened = recover_from_medium(request, &medium);
    let ck_after = capture_only(request, &reopened, &medium, &high_members);
    let body = ck_after
        .bytes()
        .strip_prefix(super::checkpoint_wire_fixture::CHECKPOINT_MAGIC)
        .unwrap();
    let manifest = super::checkpoint_wire_fixture::field(body, "manifest");
    let naming = super::checkpoint_wire_fixture::field(manifest, "naming");
    let main_row = super::checkpoint_wire_fixture::rows(super::checkpoint_wire_fixture::field(
        naming, "branches",
    ))
    .into_iter()
    .find(|row| {
        rmp_serde::from_slice::<BranchId>(super::checkpoint_wire_fixture::field(row, "branch"))
            .unwrap()
            == main
    })
    .unwrap();
    let index_raw = super::checkpoint_wire_fixture::field(main_row, "index");
    let floor: Option<u64> = rmp_serde::from_slice(super::checkpoint_wire_fixture::field(
        index_raw,
        "retention_floor",
    ))
    .unwrap();
    assert_eq!(floor, Some(c2_id.0));
}

#[test]
fn fork_between_c2_capture_and_ack_preserves_parent_data_after_main_deletes() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_named_runtime(&medium, u64::MAX / 4);
    let main = runtime.main_branch_identity().branch_id().clone();

    let (_, e1, n1) = create_named_entities(request, &runtime, &main, &["rebind-k".into()]);
    let ck1 = capture_only(request, &runtime, &medium, &[]);
    medium.file_checkpoint(ck1.clone());
    try_ack(request, &ck1).unwrap();

    // Fork b1 from main at C1 (b1 keeps n1).
    let b1 = BranchId("rebind-b1".into());
    let main_c1 = observe_branch(&runtime, &main);
    fork_retained_at(request, &runtime, &main_c1, &b1).unwrap();

    // Main deletes e1 (retiring n1), then capture C2 (main has retired n1, b1 keeps n1 live).
    delete_entity(request, &runtime, &main, e1[0]);
    let ck1_members: Vec<_> = ck1
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect();
    let ck2 = capture_only(request, &runtime, &medium, &ck1_members);
    medium.file_checkpoint(ck2.clone());

    // Fork b2 from b1 BEFORE C2 is acknowledged, then acknowledge C2.
    let b2 = BranchId("rebind-b2".into());
    let b1_obs = observe_branch(&runtime, &b1);
    fork_retained_at(request, &runtime, &b1_obs, &b2).unwrap();
    try_ack(request, &ck2).unwrap();

    // b2 must resolve n1 via C2's b1 root (NOT C2's main root where n1 was deleted).
    assert!(
        runtime
            .branch_live_persistent_names(&b2)
            .unwrap()
            .contains_key(&n1[0]),
        "b2 forked from b1 before C2 ACK must keep n1 live after C2 ACK"
    );
}
