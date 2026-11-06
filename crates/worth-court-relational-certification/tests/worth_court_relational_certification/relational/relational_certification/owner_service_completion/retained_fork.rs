//! Retained fork truth and explicit persistence limits use the actual public owner.
use crate::relational_certification::ordinary_context::owner_service_completion::world::{
    commit_through_services, empty_runtime,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::{
    branch::{RelationalBranchTransition, RelationalBranchTransitionStop, RelationalForkDenial},
    durability::{
        RecoveryFailureClass, RecoveryVerificationMode, RelationalBranchTransitionArtifact,
        RelationalRecoveryStop,
    },
};
mod medium;

#[test]
fn retained_root_fork_preserves_old_truth_and_current_fork_still_refuses_drift() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let schema = worth_testkit_relational::aspect_storage::AspectSchemaFixture::default();
    let runtime = schema.build_runtime();
    let services = runtime.owner_component_services();
    let first = commit_through_services(execution, &runtime, &services, "first");
    let old = services
        .basis_port()
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let (_, current_only) = services
        .fork_port()
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    let second = commit_through_services(execution, &runtime, &services, "second");
    let current = services
        .basis_port()
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    assert_ne!(old.reference(), current.reference());
    assert!(matches!(
        services.fork_port().fork_branch(
            execution,
            BranchId("ordinary-stale".into()),
            current_only
        ),
        Err(RelationalBranchTransitionStop::Denied(
            RelationalForkDenial::StaleSource
        ))
    ));
    let source = services
        .fork_port()
        .admit_retained_fork_source(&old)
        .unwrap();
    let (fork, destination) = services
        .fork_port()
        .fork_retained_branch(execution, BranchId("retained-child".into()), source)
        .unwrap();
    assert_eq!(fork.shared_commit_id(), Some(first.commit.commit_id));
    assert_eq!(
        destination.descriptor().root_identity(),
        old.descriptor().root_identity()
    );
    assert_eq!(
        destination.observation().commit_id(),
        old.observation().commit_id()
    );
    assert_ne!(destination.identity(), old.identity());
    assert_eq!(
        services
            .basis_port()
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap()
            .reference(),
        current.reference()
    );
    let artifacts = runtime
        .durability()
        .branch_transition_artifacts(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(artifacts.len(), 1);
    let recorded =
        RelationalBranchTransitionArtifact::readmit(artifacts[0].as_bytes(), execution).unwrap();
    let RelationalBranchTransition::RetainedFork {
        target,
        source_generation,
        source_truth_version,
        source_head,
        ..
    } = recorded.transition()
    else {
        panic!("canonical meaning must identify retained selection")
    };
    assert_eq!(target, destination.identity().branch_id());
    assert_eq!(*source_generation, old.reference().generation());
    assert_eq!(*source_truth_version, old.truth_version());
    assert_eq!(source_head.selected_commit_id(), first.commit.commit_id.0);
    assert_ne!(
        source_head.runtime_instance_id(),
        old.identity().runtime_instance_id()
    );
    assert_eq!(
        source_head.roots(),
        old.reference().target().as_basis().unwrap().roots()
    );
    let plan = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut unsupported = schema.build_runtime();
    let RelationalRecoveryStop::Durability(cause) = unsupported
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err()
    else {
        panic!("specific unsupported reconstruction")
    };
    assert_eq!(
        cause.class,
        RecoveryFailureClass::UnsupportedRetainedForkReplay,
        "the actual recovery must reach the retained transition: {cause:?}"
    );
    assert!(unsupported
        .branch_identity(destination.identity().branch_id())
        .is_err());
    // A checkpoint after the fork captures its actual live destination root.
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let plan = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut restored = schema.build_runtime();
    restored
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let restored_child = restored
        .branch_identity(destination.identity().branch_id())
        .unwrap();
    let restored_child = restored.admit_branch_basis(&restored_child).unwrap();
    assert_eq!(
        restored_child.observation().commit_id(),
        Some(first.commit.commit_id)
    );
    assert_eq!(
        restored_child.descriptor().schema_commitment(),
        old.descriptor().schema_commitment()
    );
    assert_eq!(
        restored
            .admit_branch_basis(&restored.main_branch_identity())
            .unwrap()
            .observation()
            .commit_id(),
        Some(second.commit.commit_id)
    );
    runtime
        .snapshots()
        .release_snapshot(&first.snapshot)
        .unwrap();
    runtime
        .snapshots()
        .release_snapshot(&second.snapshot)
        .unwrap();
}
