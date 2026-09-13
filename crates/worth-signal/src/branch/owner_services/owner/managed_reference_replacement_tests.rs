use crate::branch::ManagedSignalBranchReferenceAdmissionDenial;
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::data::node::NodeState;
use crate::facade::{NodeContract, SignalRuntime};
use crate::schema::data::{
    SignalSchemaDescriptor, SignalSchemaId, SignalSchemaName, SignalSchemaRegistration,
    SignalSchemaRegistry, SignalSchemaVersion,
};
use std::sync::Arc;

use super::super::tests::fork_sharing::seed_nonempty_persistent_branch_state;
use super::super::tests::runtime_root::runtime_with_two_branches;

#[test]
fn managed_reference_clones_do_not_strongly_own_the_owner_lifecycle() {
    let (mut runtime, _, _, basis) = runtime_with_two_branches();
    let (port, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = port.upgrade_owner().expect("sealed owner remains live");
    let admission = owner.admit().expect("cell inspection admits");
    let cell = owner
        .lookup_cell(&admission, basis.owner_branch_id())
        .expect("the real registry supplies the referenced cell");
    let strong_before = Arc::strong_count(&owner.lifecycle);
    let cell_strong_before = Arc::strong_count(&cell);
    let reference = owner
        .issue_managed_branch_reference(&basis)
        .expect("a real admitted basis issues its branch reference");
    let cloned = reference.clone();

    assert_eq!(Arc::strong_count(&owner.lifecycle), strong_before);
    assert_eq!(Arc::strong_count(&cell), cell_strong_before);
    drop(reference);
    drop(cloned);
    assert_eq!(Arc::strong_count(&owner.lifecycle), strong_before);
    assert_eq!(Arc::strong_count(&cell), cell_strong_before);
}

#[test]
fn replaced_cell_incarnation_cannot_reauthorize_an_owner_issued_reference() {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let (port, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = port.upgrade_owner().expect("sealed owner remains live");
    let reference = owner
        .issue_managed_branch_reference(&basis)
        .expect("a real admitted basis issues its branch reference");
    let admission = owner.admit().expect("replacement setup admits");
    owner
        .replace_branch_incarnation_for_test(&admission, branch.id)
        .expect("the owner replaces the cell through its canonical registry");

    assert!(matches!(
        owner.admit_managed_branch_reference(&reference),
        Err(ManagedSignalBranchReferenceAdmissionDenial::BranchIncarnationReplaced)
    ));
}

#[test]
fn replacement_helper_preserves_complete_cell_truth_with_a_new_incarnation() {
    let schema = SignalSchemaRegistry::from_registrations(vec![SignalSchemaRegistration::new(
        SignalSchemaDescriptor::new(
            SignalSchemaId(917),
            SignalSchemaName::new("signal.owner.replacement"),
            SignalSchemaVersion::new(3, 4),
            NodeContract::wildcard(),
        ),
    )
    .expect("replacement schema registration is valid")])
    .expect("replacement schema registry is valid");
    let mut graph = SignalGraph::new().with_schema_registry(schema);
    let source = graph.create_node();
    let dependent = graph.create_node();
    graph
        .set_dependencies(
            dependent,
            [crate::data::dependency::DependencyEdge::new(
                source,
                crate::data::aspect::Aspect::new(7),
            )],
        )
        .expect("replacement fixture installs adversarial topology");
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(graph);
    seed_nonempty_persistent_branch_state(&mut runtime, source);
    let branch = runtime.current_branch();
    let mut context = ();
    runtime
        .transaction(&mut context, |transaction| {
            transaction.set_dependencies(dependent, [DependencyEdge::new(source, Aspect::new(6))])
        })
        .expect("replacement fixture creates a real nonempty mutation journal");
    let basis = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("replacement branch basis is owner-issued");
    let (port, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = port.upgrade_owner().expect("sealed owner remains live");
    let admission = owner.admit().expect("replacement proof admits");
    let original = owner
        .lookup_cell(&admission, branch.id)
        .expect("the original cell is live");
    let original_incarnation = original.incarnation();
    original
        .with_state(&admission, |state, _| {
            state
                .state_mut()
                .graph_mut()
                .set_node_state(dependent, NodeState::Clean)
                .expect("replacement fixture distinguishes graph content");
            state
                .state_mut()
                .graph_mut()
                .populate_replacement_clone_contract(dependent);
            state
                .state_mut()
                .populate_replacement_ancestry_contract(crate::state::SignalBranchId(8_271));
        })
        .expect("replacement fixture populates canonical cell truth");
    let before = original
        .with_state(&admission, |state, _| {
            (
                state.handle().clone(),
                state
                    .state()
                    .replacement_observation(&[source, dependent], &[()]),
                state.head_generation(),
                state.restore_snapshot_id(),
                state
                    .observation()
                    .expect("original observation is complete"),
            )
        })
        .expect("the original state is observable");

    owner
        .replace_branch_incarnation_for_test(&admission, branch.id)
        .expect("the canonical replacement path succeeds");
    let replacement = owner
        .lookup_cell(&admission, branch.id)
        .expect("the replacement is live under the same identity");
    assert_ne!(replacement.incarnation(), original_incarnation);
    let after = replacement
        .with_state(&admission, |state, _| {
            (
                state.handle().clone(),
                state
                    .state()
                    .replacement_observation(&[source, dependent], &[()]),
                state.head_generation(),
                state.restore_snapshot_id(),
                state
                    .observation()
                    .expect("replacement observation is complete"),
            )
        })
        .expect("the replacement state is observable");
    assert_eq!(after.0, before.0);
    assert_eq!(after.1.graph_retained, before.1.graph_retained);
    assert_eq!(after.1.config, before.1.config);
    assert_eq!(after.1.checkpoint, before.1.checkpoint);
    assert!(after.1.resource_matches(&before.1));
    assert!(after.1.temporal_matches(&before.1));
    assert_eq!(after.1.telemetry, before.1.telemetry);
    assert!(after.1.ancestry_matches(&before.1));
    assert_eq!(after.1.mutation_ledger, before.1.mutation_ledger);
    assert!(
        !before
            .1
            .mutation_ledger
            .structural_merge_journal()
            .records
            .is_empty(),
        "the retained mutation-ledger oracle is adversarially nonempty"
    );
    assert!(before.1.graph_clone_local.aspect_lowering_owner_present);
    assert_eq!(before.1.graph_clone_local.invalidation_readiness_epoch, 41);
    assert_ne!(before.1.graph_clone_local.observation_active_generation, 0);
    assert_eq!(
        before.1.graph_clone_local.observation_active_request,
        crate::logic::transaction::SignalObservationRequest::counters().with_performed_work()
    );
    assert_eq!(before.1.graph_clone_local.completed_execution_boundaries, 1);
    assert_eq!(
        before.1.graph_clone_local.last_completion,
        Some(crate::logic::transaction::SignalObservationCompletion::Completed)
    );
    assert!(!before.1.graph_clone_local.performed_work.is_empty());
    assert!(!before
        .1
        .graph_clone_local
        .pending_repeated_admissions
        .is_empty());
    assert_ne!(
        after.1.graph_clone_local.lifecycle_token_identity,
        before.1.graph_clone_local.lifecycle_token_identity
    );
    assert_ne!(
        after.1.graph_clone_local.graph_instance_id,
        before.1.graph_clone_local.graph_instance_id
    );
    assert_ne!(
        after.1.graph_clone_local.schema_registry_identity,
        before.1.graph_clone_local.schema_registry_identity
    );
    assert!(!after.1.graph_clone_local.aspect_lowering_owner_present);
    assert_eq!(after.1.graph_clone_local.invalidation_readiness_epoch, 0);
    assert_eq!(after.1.graph_clone_local.observation_active_generation, 0);
    assert_eq!(
        after.1.graph_clone_local.observation_active_request,
        crate::logic::transaction::SignalObservationRequest::operation()
    );
    assert_eq!(after.1.graph_clone_local.completed_execution_boundaries, 0);
    assert_eq!(after.1.graph_clone_local.last_completion, None);
    assert_ne!(
        after.1.graph_clone_local.observation_cleanup_identity,
        before.1.graph_clone_local.observation_cleanup_identity
    );
    assert_eq!(
        after.1.graph_clone_local.performed_counters,
        Default::default()
    );
    assert!(after.1.graph_clone_local.performed_work.is_empty());
    assert!(after
        .1
        .graph_clone_local
        .pending_repeated_admissions
        .is_empty());
    assert_eq!(after.2, before.2);
    assert_eq!(after.3, before.3);
    assert_eq!(after.4, before.4);
    assert_eq!(basis.owner_branch_id(), branch.id);
}
