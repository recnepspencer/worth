use worth_proof::TransitionOutcome;

use crate::facade::*;

#[test]
fn current_and_explicit_parent_head_forks_share_parent_basis_without_switching_active_branch() {
    let graph = SignalGraph::new();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();

    let head_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let parent = runtime.current_branch();
    let expected_parent_basis = runtime.current_branch_basis_artifact();

    let current_receipt = match runtime.fork_branch(
        SignalBranchForkRequest::from_current_branch_head("feature-current"),
    ) {
        TransitionOutcome::Success(receipt) => receipt,
        other => panic!("expected current-branch fork success, got {other:?}"),
    };
    let explicit_receipt = match runtime.fork_branch(
        SignalBranchForkRequest::from_parent_branch_head("feature-explicit", parent.id),
    ) {
        TransitionOutcome::Success(receipt) => receipt,
        other => panic!("expected explicit-parent fork success, got {other:?}"),
    };

    assert_eq!(
        current_receipt.parent_basis().payload().basis_digest(),
        expected_parent_basis.payload().basis_digest()
    );
    assert_eq!(
        explicit_receipt.parent_basis().payload().basis_digest(),
        expected_parent_basis.payload().basis_digest()
    );
    assert_eq!(
        current_receipt
            .active_branch_after_fork_basis()
            .payload()
            .basis_digest(),
        explicit_receipt
            .active_branch_after_fork_basis()
            .payload()
            .basis_digest()
    );
    assert_eq!(runtime.current_branch().id, parent.id);
    assert_eq!(
        runtime.branch_head_snapshot_id(parent.id),
        Some(head_snapshot.snapshot_id())
    );
    assert_eq!(
        current_receipt.created_branch().parent_branch_id,
        Some(parent.id)
    );
    assert_eq!(
        explicit_receipt.created_branch().parent_branch_id,
        Some(parent.id)
    );
    assert_eq!(
        current_receipt.created_branch().head_snapshot_id,
        Some(head_snapshot.snapshot_id())
    );
    assert_eq!(
        explicit_receipt.created_branch().head_snapshot_id,
        Some(head_snapshot.snapshot_id())
    );
    assert_eq!(
        current_receipt
            .created_branch_basis()
            .payload()
            .snapshot_component_digest(),
        explicit_receipt
            .created_branch_basis()
            .payload()
            .snapshot_component_digest()
    );
    assert_eq!(runtime.observe().known_branches().len(), 3);
    assert_eq!(runtime.telemetry().transaction.explicit_fork_count, 2);
    assert_eq!(
        runtime.telemetry().transaction.explicit_snapshot_fork_count,
        0
    );
}

#[test]
fn explicit_snapshot_fork_anchors_child_to_requested_snapshot_without_active_branch_side_effects() {
    use std::sync::Arc;
    use worth_execution::time::{ControlledTime, TimeSampler};

    let sampler = Arc::new(TimeSampler::new(Arc::new(ControlledTime::at_unix_secs(1))));
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_time_sampler(sampler.clone())
        .with_kernel_defaults()
        .build();

    let base_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let parent = runtime.current_branch();
    let newer_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");

    let expected_snapshot_basis =
        match runtime.snapshot_branch_basis_artifact(parent.clone(), &base_snapshot) {
            TransitionOutcome::Success(basis) => basis,
            other => panic!("expected tracked snapshot basis, got {other:?}"),
        };

    let receipt = match runtime.fork_branch_with_snapshot(
        SignalBranchForkRequest::from_parent_branch_snapshot(
            "feature-from-snapshot",
            parent.id,
            base_snapshot.snapshot_id(),
        ),
        &base_snapshot,
    ) {
        TransitionOutcome::Success(receipt) => receipt,
        other => panic!("expected explicit snapshot fork success, got {other:?}"),
    };

    assert_eq!(
        receipt
            .requested_snapshot_basis()
            .expect("snapshot request should retain explicit snapshot basis")
            .payload()
            .basis_digest(),
        expected_snapshot_basis.payload().basis_digest()
    );
    assert_eq!(
        receipt.created_branch().head_snapshot_id,
        Some(base_snapshot.snapshot_id())
    );
    assert_eq!(
        receipt.created_branch_basis().payload().snapshot_id(),
        Some(base_snapshot.snapshot_id())
    );
    assert_eq!(runtime.current_branch().id, parent.id);
    assert_eq!(
        runtime.branch_head_snapshot_id(parent.id),
        Some(newer_snapshot.snapshot_id())
    );
    assert_eq!(
        receipt
            .active_branch_after_fork_basis()
            .payload()
            .basis_digest(),
        runtime
            .current_branch_basis_artifact()
            .payload()
            .basis_digest()
    );
    assert_eq!(runtime.telemetry().transaction.explicit_fork_count, 1);
    assert_eq!(
        runtime.telemetry().transaction.explicit_snapshot_fork_count,
        1
    );
    assert_eq!(runtime.observe().known_branches().len(), 2);

    runtime
        .switch_branch(receipt.created_branch().clone())
        .unwrap();
    assert_eq!(runtime.time_sampler().monotonic(), sampler.monotonic());
}

#[test]
fn compatibility_create_branch_lowers_to_explicit_current_parent_fork_semantics() {
    let graph = SignalGraph::new();
    let mut compatibility_runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();

    let parent = compatibility_runtime.current_branch();
    let parent_basis = compatibility_runtime.current_branch_basis_artifact();
    let compatibility_branch = compatibility_runtime
        .create_branch("compatibility-feature")
        .expect("compatibility create_branch should still succeed");
    let compatibility_branch_basis =
        match compatibility_runtime.branch_basis_artifact(compatibility_branch.clone()) {
            TransitionOutcome::Success(basis) => basis,
            other => panic!("expected compatibility branch basis success, got {other:?}"),
        };

    let graph = SignalGraph::new();
    let mut explicit_runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let explicit_receipt = match explicit_runtime.fork_branch(
        SignalBranchForkRequest::from_current_branch_head("explicit-feature"),
    ) {
        TransitionOutcome::Success(receipt) => receipt,
        other => panic!("expected explicit fork success, got {other:?}"),
    };

    assert_eq!(
        compatibility_runtime.current_branch().id,
        parent.id,
        "compatibility create_branch must not silently switch the active branch"
    );
    assert_eq!(
        compatibility_runtime
            .telemetry()
            .transaction
            .explicit_fork_count,
        1,
        "compatibility create_branch must lower through explicit fork admission"
    );
    assert_eq!(
        compatibility_branch.parent_branch_id,
        Some(parent.id),
        "compatibility create_branch must retain explicit parent lineage"
    );
    assert_eq!(
        compatibility_branch_basis.payload().basis_digest(),
        explicit_receipt.created_branch_basis().payload().basis_digest(),
        "compatibility create_branch must produce the same child basis meaning as explicit current-parent forking"
    );
    assert_eq!(
        compatibility_branch_basis.payload().snapshot_component_digest(),
        explicit_receipt
            .created_branch_basis()
            .payload()
            .snapshot_component_digest(),
        "compatibility create_branch must preserve the same head-snapshot basis as explicit forking"
    );
    assert_eq!(
        compatibility_runtime
            .current_branch_basis_artifact()
            .payload()
            .basis_digest(),
        parent_basis.payload().basis_digest(),
        "compatibility create_branch must preserve the parent branch basis after fork admission"
    );
    assert_eq!(
        explicit_receipt.parent_basis().payload().basis_digest(),
        parent_basis.payload().basis_digest(),
        "explicit fork receipt should retain the same parent basis as the compatibility path"
    );
}

#[test]
fn fork_denials_are_typed_and_leave_runtime_state_unchanged() {
    let graph = SignalGraph::new();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();

    let parent = runtime.current_branch();
    let original_head = runtime.branch_head_snapshot_id(parent.id);
    let original_branch_count = runtime.observe().known_branches().len();

    let feature = runtime
        .create_branch("feature")
        .expect("compatibility branch creation should still succeed");
    runtime
        .switch_branch(feature.clone())
        .expect("switching into feature branch should succeed");
    let feature_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    runtime
        .switch_branch(parent.clone())
        .expect("switching back to main should succeed");
    let branch_count_after_feature = runtime.observe().known_branches().len();
    let head_after_feature = runtime.branch_head_snapshot_id(parent.id);

    match runtime.fork_branch(SignalBranchForkRequest::from_parent_branch_head(
        "unknown-parent",
        SignalBranchId(999),
    )) {
        TransitionOutcome::Denied(SignalBranchForkDenial::UnknownParentBranch {
            parent_branch_id,
        }) => assert_eq!(parent_branch_id, SignalBranchId(999)),
        other => panic!("expected unknown-parent denial, got {other:?}"),
    }

    let mut unknown_snapshot = feature_snapshot.clone();
    unknown_snapshot.meta.snapshot_id = SignalSnapshotId(feature_snapshot.snapshot_id().0 + 1000);
    match runtime.fork_branch_with_snapshot(
        SignalBranchForkRequest::from_parent_branch_snapshot(
            "unknown-snapshot",
            feature.id,
            unknown_snapshot.snapshot_id(),
        ),
        &unknown_snapshot,
    ) {
        TransitionOutcome::Denied(SignalBranchForkDenial::UnknownForkSnapshot {
            parent_branch_id,
            snapshot_id,
        }) => {
            assert_eq!(parent_branch_id, feature.id);
            assert_eq!(snapshot_id.0, feature_snapshot.snapshot_id().0 + 1000);
        }
        other => panic!("expected unknown-snapshot denial, got {other:?}"),
    }

    match runtime.fork_branch_with_snapshot(
        SignalBranchForkRequest::from_parent_branch_snapshot(
            "wrong-lineage",
            parent.id,
            feature_snapshot.snapshot_id(),
        ),
        &feature_snapshot,
    ) {
        TransitionOutcome::Denied(SignalBranchForkDenial::IncompatibleForkSnapshotLineage {
            parent_branch_id,
            snapshot_branch_id,
            snapshot_id,
        }) => {
            assert_eq!(parent_branch_id, parent.id);
            assert_eq!(snapshot_branch_id, feature.id);
            assert_eq!(snapshot_id, feature_snapshot.snapshot_id());
        }
        other => panic!("expected incompatible-lineage denial, got {other:?}"),
    }

    assert_eq!(runtime.current_branch().id, parent.id);
    assert_eq!(
        runtime.branch_head_snapshot_id(parent.id),
        head_after_feature
    );
    assert_eq!(
        runtime.observe().known_branches().len(),
        branch_count_after_feature
    );
    assert_eq!(runtime.telemetry().transaction.explicit_fork_count, 1);
    assert_eq!(
        runtime.telemetry().transaction.explicit_fork_denial_count,
        3
    );
    assert_eq!(original_head, None);
    assert_eq!(original_branch_count, 1);
}

#[test]
fn snapshot_basis_request_requires_matching_snapshot_payload() {
    let graph = SignalGraph::new();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();

    let parent = runtime.current_branch();
    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");

    match runtime.fork_branch(SignalBranchForkRequest::from_parent_branch_snapshot(
        "missing-payload",
        parent.id,
        snapshot.snapshot_id(),
    )) {
        TransitionOutcome::Denied(SignalBranchForkDenial::SnapshotPayloadRequiredForFork {
            request,
        }) => {
            assert_eq!(request.branch_name(), "missing-payload");
            assert_eq!(
                request.basis(),
                &SignalBranchForkRequestBasis::ParentBranchSnapshot {
                    parent_branch_id: parent.id,
                    snapshot_id: snapshot.snapshot_id(),
                }
            );
        }
        other => panic!("expected missing-payload denial, got {other:?}"),
    }

    let mut different_snapshot = snapshot.clone();
    different_snapshot.meta.snapshot_id = SignalSnapshotId(snapshot.snapshot_id().0 + 1);
    match runtime.fork_branch_with_snapshot(
        SignalBranchForkRequest::from_parent_branch_snapshot(
            "mismatch-payload",
            parent.id,
            snapshot.snapshot_id(),
        ),
        &different_snapshot,
    ) {
        TransitionOutcome::Denied(SignalBranchForkDenial::SnapshotBasisMismatch {
            requested_snapshot_id,
            provided_snapshot_id,
        }) => {
            assert_eq!(requested_snapshot_id, snapshot.snapshot_id());
            assert_eq!(provided_snapshot_id, different_snapshot.snapshot_id());
        }
        other => panic!("expected snapshot-basis mismatch denial, got {other:?}"),
    }
}
