use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::branch::admit_runtime_signal_branch_observation;
use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;
use crate::branch::owner_services::SignalOwnerCancellationSource;
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;

use super::super::runtime_root::runtime_with_two_branches_from_graph;

#[test]
fn advance_outcome_construction_fault_preserves_performed_truth_and_releases_output_custody() {
    let mut graph = SignalGraph::new();
    let original = graph.create_node();
    let replacement = graph.create_node();
    let dependent = graph.create_node();
    graph
        .set_dependencies(dependent, [DependencyEdge::new(original, Aspect::new(1))])
        .expect("advance fault fixture installs");
    let (mut runtime, sibling, branch, basis) = runtime_with_two_branches_from_graph(graph);
    let (_, mutation, _) = runtime.owner_port_slots().expect("advance owner seals");
    let owner = mutation
        .upgrade_owner()
        .expect("advance owner remains live");
    let admission = owner.admit().expect("advance fault admits");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("advance target is live");
    let sibling_cell = owner
        .lookup_cell(&admission, sibling.id)
        .expect("advance sibling is live");
    let ledger_before = owner.retention_ledger_observation();
    let cell_before = cell.cost_snapshot();
    owner
        .operation_control()
        .inject_panic_once(SignalOwnerOperationBoundary::OutcomeConstruction);

    let fault = catch_unwind(AssertUnwindSafe(|| {
        let ready = owner
            .reserve_advance_output(&admission, &cell)
            .expect("advance output custody reserves")
            .advance::<(), (), _>(
                &basis,
                &mut (),
                &SignalOwnerCancellationSource::new().token(),
                |transaction| {
                    transaction.set_dependencies(
                        dependent,
                        [DependencyEdge::new(replacement, Aspect::new(2))],
                    )
                },
            )
            .into_result()
            .expect("advance reaches outcome construction");
        let _ = ready.into_parts();
    }));
    assert!(fault.is_err());
    assert_eq!(cell.poison_recovery(), None);
    assert_eq!(
        cell.cost_snapshot().movements(),
        cell_before.movements() + 1
    );
    let observation = cell
        .observe_exact(&admission)
        .expect("performed advance remains healthy");
    assert_eq!(
        observation.generation().get(),
        basis.observation().generation().get() + 1
    );
    cell.with_state(&admission, |state, _| {
        assert_eq!(
            state.state().graph().dependency_sources_of(dependent),
            Ok(vec![replacement])
        );
        assert!(!state
            .state()
            .mutation_ledger()
            .structural_merge_journal()
            .records
            .is_empty());
    })
    .expect("performed advance truth is exact");
    let mut released = ledger_before.clone();
    released.next_lease_id += 1;
    assert_eq!(owner.retention_ledger_observation(), released);
    sibling_cell
        .observe_exact(&admission)
        .expect("advance unwind does not block its sibling");
    assert_eq!(sibling_cell.branch_id(), sibling.id);

    let readmitted = admit_runtime_signal_branch_observation(
        observation,
        branch.id,
        owner
            .acquire_admitted_retention(&admission, branch.id)
            .expect("performed advance can be readmitted"),
    );
    let healthy = owner
        .reserve_advance_output(&admission, &cell)
        .expect("outcome unwind returns output capacity")
        .advance::<(), (), _>(
            &readmitted,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .into_result()
        .expect("a healthy advance follows outcome unwind")
        .into_parts();
    assert_eq!(healthy.0.observation().generation().get(), 2);
}
