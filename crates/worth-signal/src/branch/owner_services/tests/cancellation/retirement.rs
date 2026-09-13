use worth_proof::TransitionOutcome;

use crate::branch::{
    admit_runtime_signal_branch_observation, validate_signal_branch_name,
    PlannedSignalBranchRetirement, SignalBranchRetirementDenial, SignalBranchRetirementReason,
};
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::logic::transaction::SignalRuntime;
use crate::state::SignalBranchHandle;

use super::super::super::{SignalBranchRegistryDenial, SignalOwnerCancellationSource};

type TestRuntime = SignalRuntime<(), (), (), (), ()>;

struct PopulatedRetirementSemantics {
    selected_branch: SignalBranchHandle,
    root_source: NodeId,
    target_source: NodeId,
    dispatch: NodeId,
}

#[test]
fn retirement_cancellation_before_contact_and_at_cutoff_returns_every_reservation() {
    let (mut runtime, mut planned, semantics) = planned_retirements(&[
        "cancel-before-contact",
        "cancel-before-movement",
        "healthy-retirement-twin",
    ]);
    let (before_branch, before_observation, before_plan) = planned.remove(0);
    let (cutoff_branch, cutoff_observation, cutoff_plan) = planned.remove(0);
    let (healthy_branch, _, healthy_plan) = planned.remove(0);
    let (_, _, lifecycle) = runtime
        .owner_port_slots()
        .expect("retirement fixture seals");
    let owner = lifecycle
        .upgrade_owner()
        .expect("retirement owner remains live");
    let admission = owner.admit().expect("retirement cancellation admits");
    let before_cell = owner
        .lookup_cell(&admission, before_branch.id)
        .expect("pre-contact target is live");
    let cutoff_cell = owner
        .lookup_cell(&admission, cutoff_branch.id)
        .expect("pre-movement target is live");
    let metadata_before = owner
        .metadata
        .retirement_contract_observation(&admission, before_branch.id)
        .expect("metadata baseline is admitted");
    let live_before = owner.live_count();

    let cancelled = SignalOwnerCancellationSource::new();
    cancelled.cancel();
    let before_cost = before_cell.cost_snapshot();
    let retirement = owner
        .reserve_retirement(&admission, before_branch.id)
        .expect("pre-contact retirement reserves every contract");
    let reserved = owner
        .metadata
        .retirement_contract_observation(&admission, before_branch.id)
        .expect("reserved metadata remains observable");
    assert_eq!(
        reserved.active_reservations,
        metadata_before.active_reservations + 1
    );
    assert_eq!(
        reserved.reserved_receipt_count,
        metadata_before.reserved_receipt_count + 1
    );
    assert!(matches!(
        retirement.execute_with_cancellation_observers(
            before_plan,
            &cancelled.token(),
            || panic!("pre-contact cancellation cannot reach movement preparation"),
            || panic!("pre-contact cancellation cannot perform"),
        ),
        Err(SignalBranchRetirementDenial::CancelledNoMovement)
    ));
    assert_eq!(before_cell.cost_snapshot(), before_cost);
    assert_cancelled_retirement_cleanup(
        &owner,
        &admission,
        &before_branch,
        &before_observation,
        metadata_before,
        live_before,
        &semantics,
    );

    let cutoff = SignalOwnerCancellationSource::new();
    let cutoff_token = cutoff.token();
    let cutoff_cost = cutoff_cell.cost_snapshot();
    let retirement = owner
        .reserve_retirement(&admission, cutoff_branch.id)
        .expect("pre-movement retirement reserves every contract");
    assert!(matches!(
        retirement.execute_with_cancellation_observers(
            cutoff_plan,
            &cutoff_token,
            || cutoff.cancel(),
            || panic!("pre-movement cancellation cannot perform"),
        ),
        Err(SignalBranchRetirementDenial::CancelledNoMovement)
    ));
    let cutoff_denied = cutoff_cell.cost_snapshot();
    assert_eq!(cutoff_denied.contacts(), cutoff_cost.contacts() + 1);
    assert_eq!(cutoff_denied.movements(), cutoff_cost.movements());
    assert_cancelled_retirement_cleanup(
        &owner,
        &admission,
        &cutoff_branch,
        &cutoff_observation,
        metadata_before,
        live_before,
        &semantics,
    );

    assert_dependency(
        &owner,
        &admission,
        healthy_branch.id,
        semantics.dispatch,
        semantics.target_source,
    );
    let healthy_receipt = owner
        .reserve_retirement(&admission, healthy_branch.id)
        .expect("independent healthy retirement reserves returned capacity")
        .execute(healthy_plan, &SignalOwnerCancellationSource::new().token())
        .expect("independent healthy retirement performs");
    assert_eq!(healthy_receipt.retired_branch(), &healthy_branch);
    assert_eq!(owner.live_count(), live_before - 1);
    assert!(matches!(
        owner.lookup_cell(&admission, healthy_branch.id),
        Err(SignalBranchRegistryDenial::UnknownBranch(id)) if id == healthy_branch.id
    ));
    assert_dependency(
        &owner,
        &admission,
        semantics.selected_branch.id,
        semantics.dispatch,
        semantics.root_source,
    );
}

#[test]
fn retirement_cancellation_after_movement_preserves_receipt_and_result() {
    let (mut runtime, mut planned, semantics) = planned_retirements(&["performed-wins-retirement"]);
    let (branch, _, plan) = planned.remove(0);
    let (_, _, lifecycle) = runtime
        .owner_port_slots()
        .expect("retirement fixture seals");
    let owner = lifecycle
        .upgrade_owner()
        .expect("retirement owner remains live");
    let admission = owner.admit().expect("performed retirement admits");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("performed target is live");
    let metadata_before = owner
        .metadata
        .retirement_contract_observation(&admission, branch.id)
        .expect("performed baseline is admitted");
    assert_dependency(
        &owner,
        &admission,
        branch.id,
        semantics.dispatch,
        semantics.target_source,
    );
    let cell_before = cell.cost_snapshot();
    let cancellation = SignalOwnerCancellationSource::new();
    let token = cancellation.token();
    let receipt = owner
        .reserve_retirement(&admission, branch.id)
        .expect("performed retirement reserves")
        .execute_with_cancellation_observers(plan, &token, || {}, || cancellation.cancel())
        .expect("cancellation after movement cannot erase the result");
    assert_eq!(receipt.retired_branch(), &branch);
    assert_eq!(
        cell.cost_snapshot().movements(),
        cell_before.movements() + 1
    );
    assert!(token.preflight_cell_wait().is_err());
    let metadata_after = owner
        .metadata
        .retirement_contract_observation(&admission, branch.id)
        .expect("performed receipt remains owner-admitted");
    assert_eq!(
        metadata_after.active_reservations,
        metadata_before.active_reservations
    );
    assert_eq!(
        metadata_after.reserved_receipt_count,
        metadata_before.reserved_receipt_count
    );
    assert_eq!(
        metadata_after.retained_receipt_count,
        metadata_before.retained_receipt_count + 1
    );
    assert_eq!(
        owner
            .metadata
            .retirement_receipt(&admission, branch.id)
            .expect("receipt recovery is admitted"),
        Some(receipt)
    );
    assert!(matches!(
        owner.lookup_cell(&admission, branch.id),
        Err(SignalBranchRegistryDenial::UnknownBranch(id)) if id == branch.id
    ));
    assert_dependency(
        &owner,
        &admission,
        semantics.selected_branch.id,
        semantics.dispatch,
        semantics.root_source,
    );
}

fn assert_cancelled_retirement_cleanup(
    owner: &std::sync::Arc<super::super::super::SignalOwner<(), (), ()>>,
    admission: &super::super::super::SignalOwnerOperationAdmission<'_>,
    branch: &SignalBranchHandle,
    expected_observation: &crate::branch::SignalBranchObservation,
    expected_metadata: super::super::super::owner_metadata::SignalOwnerRetirementContractObservation,
    expected_live_count: usize,
    semantics: &PopulatedRetirementSemantics,
) {
    assert_eq!(owner.live_count(), expected_live_count);
    let cell = owner
        .lookup_cell(admission, branch.id)
        .expect("cancelled retirement restores exact registry membership");
    assert_eq!(
        owner
            .metadata
            .retirement_contract_observation(admission, branch.id)
            .expect("cancelled metadata remains admitted"),
        expected_metadata
    );
    assert_eq!(
        owner
            .metadata
            .retirement_receipt(admission, branch.id)
            .expect("cancelled receipt lookup is admitted"),
        None
    );

    let observation = cell
        .observe_exact(admission)
        .expect("cancelled target remains semantically observable");
    assert_eq!(
        &observation, expected_observation,
        "cancelled retirement preserves the exact semantic state and generation"
    );
    let basis = admit_runtime_signal_branch_observation(
        observation,
        branch.id,
        owner
            .acquire_admitted_retention(admission, branch.id)
            .expect("cancelled target can issue a fresh admitted basis"),
    );
    let registry_reservations = owner.reservation_count();
    let lineage = owner
        .reserve_fork_destination(
            admission,
            &basis,
            validate_signal_branch_name(format!("lineage-after-cancel-{}", branch.id.0))
                .expect("lineage identity validates"),
        )
        .expect("cancelled retirement returns its lineage reservation");
    drop(lineage);
    assert_eq!(owner.reservation_count(), registry_reservations);
    assert_eq!(
        owner
            .metadata
            .branch_children(admission, branch.id)
            .expect("lineage inspection is admitted"),
        Vec::new()
    );
    assert_dependency(
        owner,
        admission,
        branch.id,
        semantics.dispatch,
        semantics.target_source,
    );
    assert_dependency(
        owner,
        admission,
        semantics.selected_branch.id,
        semantics.dispatch,
        semantics.root_source,
    );
}

fn planned_retirements(
    names: &[&str],
) -> (
    TestRuntime,
    Vec<(
        SignalBranchHandle,
        crate::branch::SignalBranchObservation,
        PlannedSignalBranchRetirement,
    )>,
    PopulatedRetirementSemantics,
) {
    let mut graph = SignalGraph::new();
    let root_source = graph.create_node();
    let target_source = graph.create_node();
    let dispatch = graph.create_node();
    graph
        .set_dependencies(dispatch, [DependencyEdge::new(root_source, Aspect::new(0))])
        .expect("the populated retirement root installs");
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    let selected = runtime.current_branch();
    let selected_basis = runtime
        .observe_signal_branch_basis(selected.clone())
        .expect("retirement source basis admits");
    let mut planned = Vec::new();
    for name in names {
        let (branch, fork_basis) = runtime
            .fork_signal_branch(*name, &selected_basis)
            .expect("retirement target forks")
            .into_parts();
        runtime
            .switch_branch(branch.clone())
            .expect("the populated retirement target activates");
        runtime
            .graph_mut()
            .set_dependencies(
                dispatch,
                [DependencyEdge::new(target_source, Aspect::new(0))],
            )
            .expect("the target receives a real branch-local semantic mutation");
        drop(fork_basis);
        let basis = runtime
            .observe_signal_branch_basis(branch.clone())
            .expect("the mutated retirement target refreshes its exact basis");
        let expected_observation = basis.observation().clone();
        assert_eq!(
            runtime.graph().dependency_sources_of(dispatch),
            Ok(vec![target_source])
        );
        runtime
            .switch_branch(selected.clone())
            .expect("the unrelated populated source remains selected");
        assert_eq!(
            runtime.graph().dependency_sources_of(dispatch),
            Ok(vec![root_source])
        );
        let plan = match runtime.plan_signal_branch_retirement(
            branch.clone(),
            basis,
            SignalBranchRetirementReason::DependencyCancellation,
        ) {
            TransitionOutcome::Success(plan) => plan,
            other => panic!("retirement plan should issue: {other:?}"),
        };
        planned.push((branch, expected_observation, plan));
    }
    (
        runtime,
        planned,
        PopulatedRetirementSemantics {
            selected_branch: selected,
            root_source,
            target_source,
            dispatch,
        },
    )
}

fn assert_dependency(
    owner: &std::sync::Arc<super::super::super::SignalOwner<(), (), ()>>,
    admission: &super::super::super::SignalOwnerOperationAdmission<'_>,
    branch_id: crate::state::SignalBranchId,
    dispatch: NodeId,
    expected_source: NodeId,
) {
    owner
        .lookup_cell(admission, branch_id)
        .expect("the independently checked branch remains installed")
        .with_state(admission, |state, _| {
            assert_eq!(
                state.state().graph().dependency_sources_of(dispatch),
                Ok(vec![expected_source])
            );
        })
        .expect("the populated retirement branch remains inspectable");
}
