use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use crate::aggregate_identity::{
    AggregateContinuityRule, AggregateGroupLineageEvent, AggregateGroupScheme,
    AggregateGroupSchemeId, AggregateGroupTransition,
};
use worth_foundational::facade::AggregateIdentity;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_durability::internal_port::durability::authority::runtime_rebuild::recovered_group_lineage::install_recovered_group_lineage;
use worth_relational_lineage::internal_port::durability::naming::encode_group_lineage_frame;
use worth_relational_lineage::internal_port::lineage::groups::{
    CommittedGroupTransition, GroupTransitionKind,
};
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::LineageSubsystem;

#[test]
fn tail_created_lineage_regression_below_checkpoint_high_water_denies() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "lineage-regression-checkpoint");
    medium.file_embedded_checkpoint(execution, &runtime);
    create_entity_outcome(execution, &runtime, "lineage-regression-tail");
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    plan.tail_log_mut_for_test()[0]
        .readmitted_envelope_mut_for_test(execution)
        .published_lineage_mut_for_test()
        .lineage_events_mut_for_test()[0]
        .owner_targets_mut()[0] = crate::identity::data::LineageId(0);
    plan.tail_log_mut_for_test()[0]
        .readmitted_envelope_mut_for_test(execution)
        .rebuild_lineage_basis_for_test();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(
        error
            .detail
            .contains("recovered allocated lineage id 0 does not advance"),
        "{error:?}"
    );
}

#[test]
fn tail_lineage_event_regression_below_checkpoint_high_water_denies() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "event-regression-checkpoint");
    medium.file_embedded_checkpoint(execution, &runtime);
    create_entity_outcome(execution, &runtime, "event-regression-tail");
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    *plan.tail_log_mut_for_test()[0]
        .readmitted_envelope_mut_for_test(execution)
        .published_lineage_mut_for_test()
        .lineage_events_mut_for_test()[0]
        .event_id_mut_for_test() = 0;
    plan.tail_log_mut_for_test()[0]
        .readmitted_envelope_mut_for_test(execution)
        .rebuild_lineage_basis_for_test();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(error.detail.contains("lineage event id 0 does not advance"));
}

#[test]
fn tail_lineage_commit_and_branch_cross_splices_deny() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "cross-splice-checkpoint");
    medium.file_embedded_checkpoint(execution, &runtime);
    create_entity_outcome(execution, &runtime, "cross-splice-tail");
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let event = plan.tail_log_mut_for_test()[0]
        .readmitted_envelope_mut_for_test(execution)
        .published_lineage_mut_for_test()
        .lineage_events_mut_for_test()
        .first_mut()
        .expect("tail event");
    event.owner_commit_mut().branch_id = BranchId("spliced-sibling".to_owned());
    *event.lineage_event_branch_mut_for_test() = BranchId("spliced-sibling".to_owned());
    plan.tail_log_mut_for_test()[0]
        .readmitted_envelope_mut_for_test(execution)
        .rebuild_lineage_basis_for_test();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(error.detail.contains("cross-spliced"));
}

// Recovered group lineage installs only when every branch can apply it.
//
// A checkpoint envelope's group frame is covered by the envelope's payload
// digest, so altered frame bytes refuse recovery before any group state is
// built. A frame sealed with a valid digest whose posture its branch
// contradicts refuses `CorruptCheckpoint` at installation, never leaving a
// recovered branch whose group state is unavailable.

/// A canonical frame, with valid identities, retiring a group no branch held.
fn retiring_a_never_live_group() -> Vec<u8> {
    let group = AggregateIdentity::from_bytes([0x47; 32]);
    let id = AggregateGroupSchemeId::new("loop").expect("a valid scheme id");
    let committed = CommittedGroupTransition {
        kind: GroupTransitionKind::Partition,
        scheme: AggregateGroupScheme::new(id, AggregateContinuityRule::Anchor),
        transition: AggregateGroupTransition {
            assignments: Vec::new(),
            retired: [group].into(),
            lineage: [AggregateGroupLineageEvent::Retired { group }].into(),
        },
    };
    encode_group_lineage_frame(&[committed]).expect("a canonical frame")
}

/// A checkpoint plan whose first envelope carries `frame` as group lineage.
fn plan_with_group_frame(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    frame: Vec<u8>,
) -> RecoveryPlan {
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "group-posture");
    medium.file_embedded_checkpoint(execution, &runtime);
    let mode = RecoveryMode::NormalRecoveryVerification;
    let mut plan = medium.recovery_plan(execution, &runtime, mode).unwrap();
    let envelopes = plan.checkpoint.as_mut().unwrap().envelopes_mut_for_test();
    let body = mutable_positioned_body_for_test(&mut envelopes[0], execution);
    *body
        .published_lineage_mut_for_test()
        .group_lineage_mut_for_test() = frame;
    plan
}

#[test]
fn checkpoint_group_frame_bytes_are_covered_by_envelope_integrity() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    for frame in [retiring_a_never_live_group(), vec![1, 2, 3]] {
        let error = recover_error(execution, plan_with_group_frame(execution, frame));
        let unavailable = error.storage_unavailable.as_ref();
        assert!(
            unavailable.is_some_and(|stop| stop.cause == Cause::IntegrityMismatch),
            "{error:?}"
        );
    }
}

#[test]
fn recovered_group_lineage_its_branch_cannot_apply_refuses_corrupt_checkpoint() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let mut plan = plan_with_group_frame(execution, retiring_a_never_live_group());
    let envelopes = plan.checkpoint.as_mut().unwrap().envelopes_mut_for_test();
    let lineage = LineageSubsystem::new();
    let error = install_recovered_group_lineage(
        envelopes.iter().map(|positioned| positioned.envelope()),
        |records| lineage.replace_group_lineage(records),
    )
    .expect_err("a posture its branch contradicts never installs");
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("GroupPosture"), "{error:?}");
}
