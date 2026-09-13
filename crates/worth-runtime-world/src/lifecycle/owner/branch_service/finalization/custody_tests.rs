use std::panic::{catch_unwind, AssertUnwindSafe};

use super::super::execution::{execute_creation, BranchCreationExecution, CreationDestination};
use super::state::ForkedBranchFinalization;
use super::ForkedBranchInstallation;
use crate::branch::registry::installation_unwind;
use crate::branch::{OwnerCreatedComponentCustodyRecord, ProductBranchObservation};
use crate::lifecycle::owner::branch_service::tests::fork_creation::{
    fork_intent, relational_fork, setup_with_relational_source, signal_fork,
};
use crate::lifecycle::{RuntimeWorldBranchService, RuntimeWorldPreparationService};
use crate::publication::{ReservedBranchCreationAttempt, RuntimeWorldCancellationSource};
use crate::recovery::{ProductUnpublishedCause, ProductUnpublishedRetentionPosture};

type TestOwner = crate::lifecycle::RuntimeWorldOwnerRoot<(), (), (), (), ()>;

fn settled_creation(
    owner: &TestOwner,
    source: &ProductBranchObservation,
) -> ForkedBranchInstallation {
    let intent = fork_intent(
        "creation-custody",
        relational_fork("relational-custody-drop"),
        signal_fork("signal-custody-drop"),
    );
    let mut reservation = owner
        .state
        .branches
        .reserve_branch(owner.owner_identity(), intent.name().clone())
        .unwrap();
    let (branch, lifecycle) = owner
        .issue_branch_identities(intent.name().clone())
        .unwrap();
    let witness = reservation
        .bind_creation_destination(branch.clone(), lifecycle)
        .unwrap();
    let cancellation = RuntimeWorldCancellationSource::new();
    let attempt: ReservedBranchCreationAttempt = owner
        .prepare_creation(source.clone(), intent, &cancellation.token(), None)
        .unwrap();
    let destination = CreationDestination {
        witness,
        branch: branch.clone(),
        incarnation: lifecycle,
    };
    match execute_creation(owner, attempt, &destination, &cancellation.token()) {
        BranchCreationExecution::Settled {
            attempt,
            progress,
            successor_basis,
        } => ForkedBranchInstallation {
            branch,
            lifecycle,
            reservation,
            attempt,
            progress,
            successor_basis,
        },
        _ => panic!("the real two-owner creation settles"),
    }
}

#[test]
fn settled_creation_drop_retains_exact_forks_and_destination_without_binding_pins() {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let pins = owner.state.retention.active_component_obligation_count();
    let observations = owner.state.retention.active_observation_count();
    let installation = settled_creation(&owner, &source);
    let destination = (installation.branch.clone(), installation.lifecycle);
    let successor = installation.successor_basis.clone();
    let (_, results) = installation.progress.ready_results().unwrap();
    let rel = results.relational_fork_target_identity().unwrap().clone();
    let signal = results.signal_publication_identity().unwrap();
    let owed = owner
        .state
        .custody
        .installed_records()
        .into_iter()
        .map(OwnerCreatedComponentCustodyRecord::into_retirement_work)
        .collect::<Vec<_>>();
    drop(installation);
    let handle = owner.recovery_handles().pop().unwrap();
    let record = owner.inspect_recovery(&handle).unwrap();
    assert_eq!(record.cause(), ProductUnpublishedCause::CallerAbandoned);
    assert_eq!(
        record.destination_branch(),
        Some((&destination.0, destination.1))
    );
    assert_eq!(record.owner_effect_count(), 2);
    assert_eq!(
        record.component_results().relational_fork_target_identity(),
        Some(&rel)
    );
    assert_eq!(
        record.component_results().signal_publication_identity(),
        Some(signal)
    );
    assert_eq!(record.successor_basis(), Some(&successor));
    assert_eq!(
        record.retention_posture(),
        ProductUnpublishedRetentionPosture::BindingReserved
    );
    assert!(record.successor_commit().is_none());
    assert_eq!(
        owner.state.retention.active_component_obligation_count(),
        pins
    );
    assert_eq!(
        owner.state.retention.active_observation_count(),
        observations
    );
    assert_eq!(owner.state.history.reserved_len(), 1);
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
    drop(record);
    assert_eq!(owner.cleanup_recovery_handle(&handle).unwrap(), owed);
    assert_eq!(owner.state.custody.installed(), 0);
    assert_eq!(owner.state.history.reserved_len(), 0);
    assert_eq!(owner.state.recovery.metadata_bytes(), 0);
}

#[test]
fn assembled_creation_unwind_retains_original_head_and_observation_authority() {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let observations = owner.state.retention.active_observation_count();
    let installation = settled_creation(&owner, &source);
    let destination = (installation.branch.clone(), installation.lifecycle);
    let finalization = ForkedBranchFinalization::from_installation(installation)
        .bind_publication()
        .unwrap();
    let observed = finalization
        .observe(&owner.state.retention, &owner.state.history)
        .unwrap();
    let costs = owner.state.retention.cost_snapshot();
    assert_eq!(
        owner.state.retention.active_observation_count(),
        observations + 1
    );
    assert!(catch_unwind(AssertUnwindSafe(move || {
        let _owned = observed;
        panic!("caller leaves with fully assembled creation authority");
    }))
    .is_err());
    let handle = owner.recovery_handles().pop().unwrap();
    let record = owner.inspect_recovery(&handle).unwrap();
    assert_eq!(record.cause(), ProductUnpublishedCause::CallerAbandoned);
    assert_eq!(
        record.destination_branch(),
        Some((&destination.0, destination.1))
    );
    assert_eq!(record.owner_effect_count(), 2);
    assert_eq!(
        record.retention_posture(),
        ProductUnpublishedRetentionPosture::ProductHeadPinsRetained
    );
    assert!(record.successor_commit().is_some());
    assert_eq!(
        record.live_obligation_count(),
        7,
        "head pair + head history + observation pair + observation history + recovery slot"
    );
    assert_eq!(
        owner.state.retention.cost_snapshot(),
        costs,
        "abandonment neither acquires nor retags"
    );
    assert_eq!(
        owner.state.retention.active_observation_count(),
        observations + 1
    );
    assert_eq!(owner.state.branches.branch_count(), 1);
    assert_eq!(
        owner.state.branches.root_cell().unwrap().atomic_snapshot(),
        *source.snapshot()
    );
    drop(record);
    assert_eq!(owner.cleanup_recovery_handle(&handle).unwrap().len(), 2);
    assert_eq!(
        owner.state.retention.active_observation_count(),
        observations
    );
    assert_eq!(owner.state.operation.active(), 0);
}

#[test]
fn actual_registry_insertion_unwind_never_creates_an_unpublished_record() {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let observations = owner.state.retention.active_observation_count();
    let installation = settled_creation(&owner, &source);
    let branch = installation.branch.clone();
    let successor = installation.successor_basis.clone();
    let cancellation = RuntimeWorldCancellationSource::new();
    let _armed = installation_unwind::arm();
    assert!(catch_unwind(AssertUnwindSafe(|| {
        super::install_forked_branch(&owner, installation, &cancellation.token())
    }))
    .is_err());
    let installed = owner
        .state
        .branches
        .branch_cell(&branch)
        .unwrap()
        .atomic_snapshot();
    assert_eq!(installed.commit().basis(), &successor);
    assert!(owner.recovery_handles().is_empty());
    assert_eq!(owner.state.branches.branch_count(), 2);
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(owner.state.publication_capacity.active(), 0);
    assert_eq!(owner.state.recovery.metadata_bytes(), 0);
    assert_eq!(
        owner.state.retention.active_observation_count(),
        observations
    );
    assert_eq!(
        owner.state.custody.installed(),
        2,
        "installed creation still owes its actual owner branches"
    );
}

#[test]
fn retirement_before_caller_drop_does_not_erase_actual_installation() {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let installation = settled_creation(&owner, &source);
    let name = installation.branch.name().clone();
    let observed = ForkedBranchFinalization::from_installation(installation)
        .bind_publication()
        .unwrap()
        .observe(&owner.state.retention, &owner.state.history)
        .unwrap();
    let ForkedBranchFinalization {
        reservation,
        mut custody,
        ..
    } = observed.state;
    let cancellation = RuntimeWorldCancellationSource::new();
    let child = custody
        .install_creation_cell(reservation, &cancellation.token())
        .unwrap();
    let old_incarnation = child.lifecycle_incarnation();
    let retired = owner.retire_product_branch(&child).unwrap();
    assert_eq!(retired.owner_retirement_work().len(), 2);
    drop(child);
    let replacement = owner
        .create_reused_branch(source.clone(), name, &cancellation.token())
        .unwrap();
    assert_ne!(replacement.lifecycle_incarnation(), old_incarnation);
    drop(custody);
    assert!(
        owner.recovery_handles().is_empty(),
        "past installation survives retirement and same-name reuse"
    );
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(owner.state.recovery.metadata_bytes(), 0);
    assert_eq!(
        owner
            .state
            .branches
            .branch_cell(replacement.branch_identity())
            .unwrap()
            .atomic_snapshot(),
        *replacement.snapshot()
    );
}
