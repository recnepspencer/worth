#[allow(unused_imports)]
use crate::branch_basis_contract::ordinary_context::*;
use std::ptr;

use worth_proof::TransitionOutcome;
use worth_signal::facade::runtime::SignalBranchRetirementDenial;
use {
    worth_foundational::FoundationalBranchReferenceMismatchAxis,
    worth_foundational::FoundationalBranchTarget,
};
use {
    worth_signal::facade::branch::SignalBranchAdvanceDenial,
    worth_signal::facade::branch::SignalBranchBasisCompatibilityDenial,
    worth_signal::facade::branch::SignalBranchBasisReadmissionDenial,
    worth_signal::facade::branch::SignalBranchForkOperationDenial,
    worth_signal::facade::branch::SignalBranchRetentionReleaseDenial,
    worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome,
    worth_signal::facade::branch::SignalBranchTarget,
    worth_signal::facade::branch::SignalOwnerServiceCostSnapshot,
    worth_signal::facade::branch::SignalOwnerUnavailable,
    worth_signal::facade::branch::SIGNAL_BRANCH_BASIS_DESCRIPTOR_SCHEMA_VERSION,
};
use {
    worth_signal::facade::SignalBranchId, worth_signal::facade::SignalBranchRetirementReason,
    worth_signal::facade::SignalError, worth_signal::facade::SignalGraph,
    worth_signal::facade::SignalRuntime, worth_signal::facade::TransactionOutcome,
};

#[test]
fn admitted_basis_clones_share_one_owner_admission() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<worth_signal::facade::branch::AdmittedSignalBranchBasis>();
    assert_send_sync::<SignalOwnerServiceCostSnapshot>();
    assert_send_sync::<SignalOwnerUnavailable>();
    let runtime = runtime();
    let branch = runtime.current_branch();
    let basis = runtime
        .observe_signal_branch_basis(branch)
        .expect("owner should observe its bootstrap branch");
    let shared = basis.clone();

    assert!(std::ptr::eq(basis.observation(), shared.observation()));
    assert_eq!(basis.descriptor(), shared.descriptor());
}

#[test]
fn unsupported_version_and_unknown_branch_are_distinct() {
    let runtime = runtime();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("owner observation should succeed");

    let mut unsupported =
        serde_json::to_value(basis.descriptor()).expect("descriptor should serialize");
    let unsupported_version = SIGNAL_BRANCH_BASIS_DESCRIPTOR_SCHEMA_VERSION + 1;
    unsupported["schema_version"] = serde_json::Value::from(unsupported_version);
    let unsupported =
        serde_json::from_value(unsupported).expect("altered descriptor remains structural");
    assert!(matches!(
        runtime.readmit_signal_branch_basis(unsupported),
        Err(worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::UnsupportedDescriptorVersion {
            observed,
            supported,
        }) if observed == unsupported_version
            && supported == SIGNAL_BRANCH_BASIS_DESCRIPTOR_SCHEMA_VERSION
    ));

    let mut unknown =
        serde_json::to_value(basis.descriptor()).expect("descriptor should serialize");
    let unknown_branch_id = SignalBranchId(u64::MAX);
    unknown["owner_branch_id"] = serde_json::Value::from(unknown_branch_id.0);
    let unknown = serde_json::from_value(unknown).expect("altered descriptor remains structural");
    assert!(matches!(
        runtime.readmit_signal_branch_basis(unknown),
        Err(worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::UnknownBranch {
            branch_id,
        }) if branch_id == unknown_branch_id
    ));
}

#[test]
fn descriptor_round_trip_requires_owner_readmission() {
    let runtime = runtime();
    let branch = runtime.current_branch();
    let basis = runtime
        .observe_signal_branch_basis(branch)
        .expect("owner observation should succeed");
    let json = serde_json::to_string(basis.descriptor()).expect("descriptor is transportable");
    let descriptor = serde_json::from_str(&json).expect("descriptor should round-trip");

    let readmitted = runtime
        .readmit_signal_branch_basis(descriptor)
        .expect("the live owner should readmit its exact descriptor");
    assert_eq!(readmitted.observation(), basis.observation());
}

#[test]
fn foreign_owner_and_definition_substitutions_are_distinct() {
    let first = runtime();
    let second = runtime();
    let basis = first
        .observe_signal_branch_basis(first.current_branch())
        .expect("owner observation should succeed");
    assert!(matches!(
        second.readmit_signal_branch_basis(basis.descriptor().clone()),
        Err(worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::OwnerMismatch { .. })
    ));

    let mut transported =
        serde_json::to_value(basis.descriptor()).expect("descriptor should serialize");
    let definition = transported["observation"]["target"]["Basis"]["definition_basis"]
        .as_u64()
        .expect("Signal target carries a definition basis");
    transported["observation"]["target"]["Basis"]["definition_basis"] =
        serde_json::Value::from(definition.wrapping_add(1));
    let altered =
        serde_json::from_value(transported).expect("altered descriptor remains structural");
    assert!(matches!(
        first.readmit_signal_branch_basis(altered),
        Err(
            worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::DefinitionMismatch { .. }
        )
    ));
}

#[test]
fn transported_lifecycle_posture_cannot_reopen_a_basis() {
    let runtime = runtime();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("owner observation should succeed");
    let mut transported =
        serde_json::to_value(basis.descriptor()).expect("descriptor should serialize");
    transported["lifecycle_posture"] = serde_json::Value::from("Retired");
    let retired = serde_json::from_value(transported).expect("retired posture remains descriptive");

    assert!(matches!(
        runtime.readmit_signal_branch_basis(retired),
        Err(worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::LifecycleMismatch)
    ));
}

#[test]
fn unavailable_snapshot_is_distinct_from_reference_drift() {
    let runtime = runtime();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("owner observation should succeed");
    let mut transported =
        serde_json::to_value(basis.descriptor()).expect("descriptor should serialize");
    transported["observation"]["target"]["Basis"]["snapshot_id"] =
        serde_json::Value::from(u64::MAX);
    let unavailable =
        serde_json::from_value(transported).expect("altered descriptor remains structural");

    assert!(matches!(
        runtime.readmit_signal_branch_basis(unavailable),
        Err(
            worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::UnavailableSnapshot { .. }
        )
    ));
}

#[test]
fn advance_moves_generation_and_stales_the_previous_descriptor() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            worth_signal::facade::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = runtime();
    let branch = runtime.current_branch();
    let before = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("owner observation should succeed");
    let descriptor = before.descriptor().clone();
    let advance = runtime
        .advance_signal_branch(request_execution, &mut (), &before, |_| Ok(()))
        .expect("owner advance should return a new admitted basis");
    assert_eq!(
        advance.transaction().outcome,
        worth_signal::facade::TransactionOutcome::Committed
    );
    let after = advance.into_basis();

    assert_eq!(after.observation().generation().get(), 1);
    assert!(matches!(
        runtime.readmit_signal_branch_basis(descriptor),
        Err(
            worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::ReferenceMismatch { .. }
        )
    ));

    // Ordinary readmission is a statement about the branch's current state, so
    // it must refuse a superseded descriptor. An exact component obligation is
    // a statement about one immutable target, so staleness alone must not deny
    // it: the runtime still holds that target, and the caller still needs it.
    let superseded_lease = runtime
        .retain_signal_component_basis(&before)
        .expect("an exact obligation over a superseded basis stays legitimate");
    assert_eq!(
        superseded_lease.retained_target(),
        before
            .observation()
            .target()
            .as_basis()
            .expect("an owner observation carries a basis target")
    );
    assert!(matches!(
        runtime.release_signal_component_basis(superseded_lease),
        worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome::Released(_)
    ));
}

#[test]
fn snapshot_and_restore_each_move_the_exact_reference() {
    let mut runtime = runtime();
    let branch = runtime.current_branch();
    let initial = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("owner observation should succeed");
    let (snapshot, captured) = runtime
        .capture_signal_branch_snapshot(&initial)
        .expect("snapshot should succeed through the owner basis")
        .into_parts();
    let observed_after_capture = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("captured branch should remain observable");

    assert_eq!(captured.observation().generation().get(), 1);
    assert_eq!(captured.observation(), observed_after_capture.observation());
    assert_eq!(
        runtime.validate_signal_basis_compatibility(&initial, &captured),
        Err(worth_signal::facade::branch::SignalBranchBasisCompatibilityDenial::SnapshotMismatch)
    );
    let restored = runtime
        .restore_signal_branch(&captured, &snapshot)
        .expect("exact owner basis should authorize restore");
    assert_eq!(restored.observation().generation().get(), 2);
    assert_eq!(
        target_basis(&restored).snapshot_id(),
        Some(snapshot.snapshot().meta.snapshot_id.0)
    );
    assert_eq!(
        target_basis(&restored).restore_snapshot_id(),
        Some(snapshot.snapshot().meta.snapshot_id.0)
    );
    assert_eq!(
        runtime.validate_signal_basis_compatibility(&captured, &restored),
        Err(worth_signal::facade::branch::SignalBranchBasisCompatibilityDenial::RestoreMismatch)
    );
    let readmitted = runtime
        .readmit_signal_branch_basis(restored.descriptor().clone())
        .expect("exact restored posture should readmit");
    assert_eq!(readmitted.observation(), restored.observation());
}

#[test]
fn stale_fork_and_failed_advance_are_typed_no_movement_outcomes() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            worth_signal::facade::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = runtime();
    let branch = runtime.current_branch();
    let stale = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("initial owner observation should succeed");
    let current = runtime
        .advance_signal_branch(request_execution, &mut (), &stale, |_| Ok(()))
        .expect("first advance should move the branch")
        .into_basis();

    assert!(matches!(
        runtime.fork_signal_branch("stale-fork", &stale),
        Err(worth_signal::facade::branch::SignalBranchForkOperationDenial::BasisMismatch { ref axes })
            if axes == &[worth_foundational::FoundationalBranchReferenceMismatchAxis::ReferenceGeneration]
    ));
    let after_fork_denial = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("branch should remain observable");
    assert_eq!(after_fork_denial.observation(), current.observation());

    assert!(matches!(
        runtime.advance_signal_branch(request_execution, &mut (), &current, |_| {
            Err(worth_signal::facade::SignalError::invalid_input(
                "injected mutation failure",
            ))
        }),
        Err(
            worth_signal::facade::branch::SignalBranchAdvanceDenial::MutationFailedNoMovement { .. }
        )
    ));
    let after_failed_advance = runtime
        .observe_signal_branch_basis(branch)
        .expect("failed advance must preserve the branch");
    assert_eq!(after_failed_advance.observation(), current.observation());
}

#[test]
fn foreign_runtime_cannot_release_an_owner_lease() {
    let owner = runtime();
    let foreign = runtime();
    let basis = owner
        .observe_signal_branch_basis(owner.current_branch())
        .expect("owner observation should succeed");
    let lease = owner
        .retain_signal_component_basis(&basis)
        .expect("live basis should be retainable");

    let lease = match foreign.release_signal_component_basis(lease) {
        worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome::Denied {
            lease,
            denial: worth_signal::facade::branch::SignalBranchRetentionReleaseDenial::ForeignRuntime,
        } => lease,
        other => panic!("foreign release should preserve the owner lease: {other:?}"),
    };
    assert!(matches!(
        owner.release_signal_component_basis(lease),
        worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome::Released(_)
    ));
}

#[test]
fn retention_lease_blocks_retirement_until_exact_release() {
    let mut runtime = runtime();
    let main = runtime.current_branch();
    let main_basis = runtime
        .observe_signal_branch_basis(main)
        .expect("owner observation should succeed");
    let fork = runtime
        .fork_signal_branch("retained-component", &main_basis)
        .expect("owner fork should succeed");
    let (branch, basis) = fork.into_parts();
    let descriptor = basis.descriptor().clone();
    let lease = runtime
        .retain_signal_component_basis(&basis)
        .expect("live owner basis should be retainable");

    let denied = runtime.plan_signal_branch_retirement(
        branch.clone(),
        basis,
        worth_signal::facade::SignalBranchRetirementReason::Superseded,
    );
    assert!(matches!(
        denied,
        worth_proof::TransitionOutcome::Denied(
            worth_signal::facade::runtime::SignalBranchRetirementDenial::RetainedComponentBasis {
                active_leases: 1,
                ..
            }
        )
    ));

    assert!(matches!(
        runtime.release_signal_component_basis(lease),
        worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome::Released(_)
    ));
    let basis = runtime
        .readmit_signal_branch_basis(descriptor.clone())
        .expect("release should permit a fresh owner admission");
    let plan = runtime.plan_signal_branch_retirement(
        branch,
        basis,
        worth_signal::facade::SignalBranchRetirementReason::Superseded,
    );
    let plan = match plan {
        worth_proof::TransitionOutcome::Success(plan) => plan,
        other => panic!("released branch should become retireable: {other:?}"),
    };
    assert!(matches!(
        runtime.retire_signal_branch(plan),
        worth_proof::TransitionOutcome::Success(_)
    ));
    assert!(matches!(
        runtime.readmit_signal_branch_basis(descriptor),
        Err(worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::RetiredBranch { .. })
    ));
}

#[path = "admitted_basis_clone_blocks_retirement_until_one_holder_remains/mod.rs"]
pub(crate) mod admitted_basis_clone_blocks_retirement_until_one_holder_remains;
