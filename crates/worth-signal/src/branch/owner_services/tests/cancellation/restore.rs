use std::sync::Arc;

use crate::branch::{
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot, SignalBranchRestoreDenial,
};
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::logic::transaction::SignalRuntime;
use crate::state::SignalBranchHandle;

use super::super::super::{
    SignalBranchCellState, SignalBranchExecutionCell, SignalOwner, SignalOwnerCancellationSource,
    SignalOwnerOperationAdmission,
};
use super::super::runtime_root::runtime_with_two_branches_from_graph;

type TestRuntime = SignalRuntime<(), (), (), (), ()>;
type TestOwner = SignalOwner<(), (), ()>;
type TestCell = SignalBranchExecutionCell<SignalBranchCellState<(), (), ()>>;

pub(in crate::branch::owner_services::tests) struct PopulatedRestoreFixture {
    pub(crate) _runtime: TestRuntime,
    pub(crate) owner: Arc<TestOwner>,
    pub(crate) cell: Arc<TestCell>,
    pub(crate) branch: SignalBranchHandle,
    pub(crate) snapshot: AdmittedSignalBranchSnapshot,
    pub(crate) current_basis: AdmittedSignalBranchBasis,
    pub(crate) snapshot_source: NodeId,
    pub(crate) live_source: NodeId,
    pub(crate) dispatch: NodeId,
}

#[test]
fn restore_cancellation_before_contact_returns_output_capacity_and_preserves_state() {
    let PopulatedRestoreFixture {
        _runtime,
        owner,
        cell,
        branch,
        snapshot,
        current_basis,
        snapshot_source,
        live_source,
        dispatch,
    } = restore_fixture();
    let admission = owner.admit().expect("restore cancellation admits");
    assert_dependency(&cell, &admission, dispatch, live_source);
    let retention_before = owner.admitted_or_reserved_retention_count(branch.id);
    let cell_before = cell.cost_snapshot();
    let reservation = owner
        .reserve_restore_output(&admission, &cell)
        .expect("restore output capacity reserves before cancellation");
    assert_eq!(
        owner.admitted_or_reserved_retention_count(branch.id),
        retention_before + 1
    );
    let cancelled = SignalOwnerCancellationSource::new();
    cancelled.cancel();
    assert!(matches!(
        reservation.restore(&current_basis, &snapshot, &cancelled.token()),
        Err(SignalBranchRestoreDenial::CancelledNoMovement)
    ));
    assert_eq!(cell.cost_snapshot(), cell_before);
    assert_dependency(&cell, &admission, dispatch, live_source);
    assert_eq!(
        owner.admitted_or_reserved_retention_count(branch.id),
        retention_before
    );

    let healthy = SignalOwnerCancellationSource::new();
    let restored = owner
        .reserve_restore_output(&admission, &cell)
        .expect("healthy twin reserves the returned output slot")
        .restore(&current_basis, &snapshot, &healthy.token())
        .expect("healthy twin restores the populated snapshot")
        .into_basis();
    assert_eq!(
        restored.observation().generation().get(),
        current_basis.observation().generation().get() + 1
    );
    assert_eq!(
        cell.cost_snapshot().movements(),
        cell_before.movements() + 1
    );
    assert_dependency(&cell, &admission, dispatch, snapshot_source);
}

#[test]
fn restore_cancellation_at_cutoff_denies_but_after_movement_is_performed_wins() {
    let PopulatedRestoreFixture {
        _runtime,
        owner,
        cell,
        branch,
        snapshot,
        current_basis,
        snapshot_source,
        live_source,
        dispatch,
    } = restore_fixture();
    let admission = owner.admit().expect("restore cutoff admits");
    assert_dependency(&cell, &admission, dispatch, live_source);
    let retention_before = owner.admitted_or_reserved_retention_count(branch.id);
    let cell_before = cell.cost_snapshot();
    let before_cutoff = SignalOwnerCancellationSource::new();
    let before_token = before_cutoff.token();
    let reservation = owner
        .reserve_restore_output(&admission, &cell)
        .expect("pre-movement restore output reserves");
    assert!(matches!(
        reservation.restore_with_cancellation_observers(
            &current_basis,
            &snapshot,
            &before_token,
            || before_cutoff.cancel(),
            || panic!("denied restore cannot cross the movement cutoff"),
        ),
        Err(SignalBranchRestoreDenial::CancelledNoMovement)
    ));
    let denied = cell.cost_snapshot();
    assert_eq!(denied.contacts(), cell_before.contacts() + 1);
    assert_eq!(denied.movements(), cell_before.movements());
    assert_dependency(&cell, &admission, dispatch, live_source);
    assert_eq!(
        owner.admitted_or_reserved_retention_count(branch.id),
        retention_before
    );

    let performed = SignalOwnerCancellationSource::new();
    let performed_token = performed.token();
    let restored = owner
        .reserve_restore_output(&admission, &cell)
        .expect("performed-wins twin reserves output capacity")
        .restore_with_cancellation_observers(
            &current_basis,
            &snapshot,
            &performed_token,
            || {},
            || performed.cancel(),
        )
        .expect("cancellation after movement cannot erase the restore")
        .into_basis();
    assert_eq!(
        restored.observation().generation().get(),
        current_basis.observation().generation().get() + 1
    );
    assert_eq!(cell.cost_snapshot().movements(), denied.movements() + 1);
    assert_dependency(&cell, &admission, dispatch, snapshot_source);
    assert!(performed_token.preflight_cell_wait().is_err());
}

pub(in crate::branch::owner_services::tests) fn restore_fixture() -> PopulatedRestoreFixture {
    let mut graph = SignalGraph::new();
    let root_source = graph.create_node();
    let snapshot_source = graph.create_node();
    let live_source = graph.create_node();
    let dispatch = graph.create_node();
    graph
        .set_dependencies(dispatch, [DependencyEdge::new(root_source, Aspect::new(0))])
        .expect("the populated restore root installs");
    let (mut runtime, _, branch, starting_basis) = runtime_with_two_branches_from_graph(graph);
    let (_, mutation, _) = runtime.owner_port_slots().expect("restore fixture seals");
    let owner = mutation
        .upgrade_owner()
        .expect("restore owner remains live");
    let admission = owner.admit().expect("restore fixture admits capture");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("restore target cell is live");
    let snapshot_ready = owner
        .reserve_advance_output(&admission, &cell)
        .expect("snapshot semantic output reserves")
        .advance::<(), (), _>(
            &starting_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |transaction| {
                transaction.set_dependencies(
                    dispatch,
                    [DependencyEdge::new(snapshot_source, Aspect::new(0))],
                )
            },
        )
        .into_result()
        .expect("the snapshot semantic value performs");
    let (snapshot_basis, _) = snapshot_ready.into_parts();
    let capture = owner
        .reserve_snapshot_outputs(&admission, &cell)
        .expect("fixture reserves snapshot outputs")
        .capture(
            &snapshot_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("fixture captures a real populated snapshot")
        .into_outcome();
    let (snapshot, captured_basis) = capture.into_parts();
    let live_ready = owner
        .reserve_advance_output(&admission, &cell)
        .expect("intervening semantic output reserves")
        .advance::<(), (), _>(
            &captured_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |transaction| {
                transaction
                    .set_dependencies(dispatch, [DependencyEdge::new(live_source, Aspect::new(0))])
            },
        )
        .into_result()
        .expect("the live cell genuinely diverges after capture");
    let (current_basis, _) = live_ready.into_parts();
    assert_dependency(&cell, &admission, dispatch, live_source);
    drop(starting_basis);
    drop(snapshot_basis);
    drop(captured_basis);
    drop(admission);
    PopulatedRestoreFixture {
        _runtime: runtime,
        owner,
        cell,
        branch,
        snapshot,
        current_basis,
        snapshot_source,
        live_source,
        dispatch,
    }
}

pub(in crate::branch::owner_services::tests) fn assert_dependency(
    cell: &Arc<TestCell>,
    admission: &SignalOwnerOperationAdmission<'_>,
    dispatch: NodeId,
    expected_source: NodeId,
) {
    cell.with_state(admission, |state, _| {
        assert_eq!(
            state.state().graph().dependency_sources_of(dispatch),
            Ok(vec![expected_source])
        );
    })
    .expect("the populated restore cell remains inspectable");
}
