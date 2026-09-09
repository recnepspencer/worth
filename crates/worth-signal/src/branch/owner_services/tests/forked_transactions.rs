use crate::branch::validate_signal_branch_name;
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::facade::SignalError;

use super::super::SignalOwnerCancellationSource;
use super::{runtime_root::runtime_with_two_branches_from_graph, with_movement_permit};

#[test]
fn forked_owner_cell_transactions_restore_abort_and_isolate_commit() {
    let mut graph = SignalGraph::new();
    let source_a = graph.create_node();
    let source_b = graph.create_node();
    let derived = graph.create_node();
    graph
        .set_dependencies(derived, [DependencyEdge::new(source_a, Aspect::new(0))])
        .expect("inherited dependency installs");

    let (mut runtime, source_branch, _, _) = runtime_with_two_branches_from_graph(graph);
    let source_basis = runtime
        .observe_signal_branch_basis(source_branch.clone())
        .expect("source basis observes");
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let source_admission = owner.admit().expect("source inspection admits");
    let source_cell = owner
        .lookup_cell(&source_admission, source_branch.id)
        .expect("source cell is live");
    let ready = owner
        .reserve_fork_output(&source_admission, &source_cell)
        .expect("fork output retention reserves")
        .fork(
            &source_basis,
            validate_signal_branch_name("transaction-fork")
                .expect("destination identity validates"),
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("persistent destination installs");
    let (destination_handle, destination_basis) = ready.into_destination_parts();
    let destination = owner
        .lookup_cell(&source_admission, destination_handle.id)
        .expect("the handed-off destination is installed");
    let destination_admission = owner.admit().expect("destination inspection admits");

    let source_identity = source_cell
        .with_state(&source_admission, |source, _| {
            source.state().persistent_identity()
        })
        .expect("source identity observes");
    let destination_identity = destination
        .with_state(&destination_admission, |destination, _| {
            destination.state().persistent_identity()
        })
        .expect("destination identity observes");
    let sharing = source_identity.sharing_with(&destination_identity);
    assert!(sharing.graph.topology_root_shared);
    assert!(sharing.graph.cause_root_shared);

    let mut aborted = None;
    with_movement_permit(|permit| {
        aborted = Some(
            destination.with_state(&destination_admission, |destination_state, _| {
                let operation_scope = super::super::conditional_execution::SignalConditionalOperationScopeBinding::issue(
                    &destination_basis,
                    destination.incarnation(),
                    destination_state.state().installed_definition().cloned(),
                );
                destination_state.execute_canonical_transaction::<(), (), _>(
                    permit,
                    operation_scope,
                    &mut (),
                    |transaction| {
                        transaction.set_dependencies(
                            derived,
                            [DependencyEdge::new(source_b, Aspect::new(0))],
                        )?;
                        Err(SignalError::invalid_input("force rollback"))
                    },
                )
            }),
        );
    });
    assert!(aborted
        .expect("aborted transaction executes")
        .expect("destination cell admits")
        .expect("callback error rollback does not unwind")
        .is_err());
    assert_destination_dependency(&destination, &destination_admission, derived, source_a);
    let source_after_abort = source_cell
        .with_state(&source_admission, |source, _| {
            source.state().persistent_identity()
        })
        .expect("source identity remains observable after abort");
    let destination_after_abort = destination
        .with_state(&destination_admission, |destination, _| {
            destination.state().persistent_identity()
        })
        .expect("destination identity remains observable after abort");
    assert!(
        source_after_abort
            .sharing_with(&destination_after_abort)
            .graph
            .cause_root_shared,
        "rollback must reinstall the shared cause-authority baseline"
    );

    let mut committed = None;
    with_movement_permit(|permit| {
        committed = Some(
            destination.with_state(&destination_admission, |destination_state, _| {
                let operation_scope = super::super::conditional_execution::SignalConditionalOperationScopeBinding::issue(
                    &destination_basis,
                    destination.incarnation(),
                    destination_state.state().installed_definition().cloned(),
                );
                destination_state.execute_canonical_transaction::<(), (), _>(
                    permit,
                    operation_scope,
                    &mut (),
                    |transaction| {
                        transaction.set_dependencies(
                            derived,
                            [DependencyEdge::new(source_b, Aspect::new(0))],
                        )
                    },
                )
            }),
        );
    });
    committed
        .expect("committed transaction executes")
        .expect("destination cell admits")
        .expect("successful callback does not unwind")
        .expect("destination transaction commits");
    assert_destination_dependency(&destination, &destination_admission, derived, source_b);
    source_cell
        .with_state(&source_admission, |source, _| {
            assert_eq!(
                source.state().graph().dependency_sources_of(derived),
                Ok(vec![source_a])
            );
        })
        .expect("source sibling remains isolated");
    drop(destination_basis);
}

fn assert_destination_dependency(
    destination: &std::sync::Arc<
        super::super::SignalBranchExecutionCell<super::super::SignalBranchCellState<(), (), ()>>,
    >,
    admission: &super::super::SignalOwnerOperationAdmission<'_>,
    derived: crate::data::handle::NodeId,
    expected: crate::data::handle::NodeId,
) {
    destination
        .with_state(admission, |destination, _| {
            assert_eq!(
                destination.state().graph().dependency_sources_of(derived),
                Ok(vec![expected])
            );
        })
        .expect("destination truth observes");
}
