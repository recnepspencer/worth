use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc;
use std::thread;

use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;
use crate::branch::owner_services::{
    SignalOwnerCancellationSource, SignalOwnerServiceCostSnapshot,
};
use crate::branch::validate_signal_branch_name;
use crate::branch::SignalBranchBasisObservationDenial;
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::state::SignalBranchId;

use super::super::progress_bound::PROGRESS_BOUND;
use super::super::runtime_root::runtime_with_two_branches_from_graph;

#[test]
fn fork_post_capture_faults_preserve_performed_destination_and_release_source_custody() {
    for boundary in [
        SignalOwnerOperationBoundary::ForkDestinationInstallation,
        SignalOwnerOperationBoundary::OutcomeConstruction,
    ] {
        exercise_fork_post_install_fault(boundary);
    }
}

fn exercise_fork_post_install_fault(boundary: SignalOwnerOperationBoundary) {
    let mut graph = SignalGraph::new();
    let upstream = graph.create_node();
    let replacement = graph.create_node();
    let dependent = graph.create_node();
    graph
        .set_dependencies(dependent, [DependencyEdge::new(upstream, Aspect::new(1))])
        .expect("fork journal fixture installs its initial dependency");
    let (mut runtime, sibling, source, basis) = runtime_with_two_branches_from_graph(graph);
    let (_, mutation, _) = runtime.owner_port_slots().expect("fork owner seals");
    let owner = mutation.upgrade_owner().expect("fork owner remains live");
    let admission = owner.admit().expect("fork fault admits");
    let source_cell = owner
        .lookup_cell(&admission, source.id)
        .expect("fork source cell is live");
    let sibling_cell = owner
        .lookup_cell(&admission, sibling.id)
        .expect("fork sibling cell is live");
    let cancellation = SignalOwnerCancellationSource::new();
    let advanced = owner
        .reserve_advance_output(&admission, &source_cell)
        .expect("source output retention reserves")
        .advance::<(), (), _>(&basis, &mut (), &cancellation.token(), |transaction| {
            transaction.set_dependencies(
                dependent,
                [DependencyEdge::new(replacement, Aspect::new(7))],
            )
        })
        .into_result()
        .expect("source journal receives a real canonical mutation");
    let (basis, transaction) = advanced.into_parts();
    assert!(transaction.touched_nodes > 0);
    let source_before = source_cell.fork_source_state_truth_after_fault();
    assert!(
        !source_before
            .mutation_ledger
            .structural_merge_journal()
            .records
            .is_empty(),
        "fork rollback sensitivity requires a genuinely nonempty source journal"
    );
    let original_children = owner
        .metadata
        .branch_children(&admission, source.id)
        .expect("source lineage is observable");
    let ledger_before = owner.retention_ledger_observation();
    let cost_before = owner.cost_snapshot();
    let failed_destination_id = SignalBranchId(source.id.0.max(sibling.id.0) + 1);
    let pause = owner.operation_control().arm_pause_once(boundary);
    owner.operation_control().inject_panic_once(boundary);
    let (fault_tx, fault_rx) = mpsc::sync_channel(1);
    let fault_owner = owner.clone();
    let fault_cell = source_cell.clone();
    let fault_basis = basis.clone();
    thread::spawn(move || {
        let fault = catch_unwind(AssertUnwindSafe(|| {
            let fault_admission = fault_owner.admit().expect("fork fault thread admits");
            let ready = fault_owner
                .reserve_fork_output(&fault_admission, &fault_cell)
                .expect("fork output custody reserves")
                .fork(
                    &fault_basis,
                    validate_signal_branch_name("faulted-fork-destination")
                        .expect("fork identity validates"),
                    &SignalOwnerCancellationSource::new().token(),
                )
                .expect("fork reaches outcome construction");
            let _ = ready.into_destination_parts();
        }));
        let _ = fault_tx.send(fault.is_err());
    });
    assert!(pause.wait_until_reached(PROGRESS_BOUND));

    let source_waits_before = source_cell.cost_snapshot().waits();
    let (same_source_tx, same_source_rx) = mpsc::sync_channel(1);
    let waiting_owner = owner.clone();
    let waiting_cell = source_cell.clone();
    thread::spawn(move || {
        let waiting_admission = waiting_owner.admit().expect("same-source waiter admits");
        let result = waiting_cell.with_state(&waiting_admission, |state, _| state.branch_id());
        let _ = same_source_tx.send(result);
    });
    assert_eq!(
        same_source_rx.recv_timeout(PROGRESS_BOUND),
        Ok(Ok(source.id)),
        "source custody must end after capture, before destination installation or outcome construction"
    );
    assert_eq!(
        source_cell.cost_snapshot().waits(),
        source_waits_before,
        "post-capture destination work must not impose a source-cell wait"
    );
    sibling_cell
        .observe_exact(&admission)
        .expect("unrelated sibling progresses during fork handoff");
    assert_eq!(sibling_cell.branch_id(), sibling.id);
    pause.release();
    assert_eq!(fault_rx.recv_timeout(PROGRESS_BOUND), Ok(true));
    assert_eq!(owner.live_count(), 3);
    assert_eq!(owner.reservation_count(), 0);
    let mut performed_children = original_children;
    performed_children.push(failed_destination_id);
    assert_eq!(
        owner
            .metadata
            .branch_children(&admission, source.id)
            .expect("performed lineage remains observable"),
        performed_children
    );
    let installed = owner
        .lookup_cell(&admission, failed_destination_id)
        .expect("post-linearization panic preserves the installed destination");
    let installed_handle = installed
        .with_state(&admission, |state, _| state.handle().clone())
        .expect("the performed destination remains readable");
    assert_eq!(installed_handle.id, failed_destination_id);
    assert_eq!(installed_handle.parent_branch_id, Some(source.id));
    let mut released = ledger_before.clone();
    released.next_lease_id += 1;
    assert_eq!(owner.retention_ledger_observation(), released);
    assert_fork_fault_costs(owner.cost_snapshot(), cost_before);
    assert_eq!(source_cell.poison_recovery(), None);
    let mut expected_source = source_before;
    expected_source
        .mutation_ledger
        .clear_all(source.head_snapshot_id);
    assert_eq!(
        source_cell.fork_source_state_truth_after_fault(),
        expected_source,
        "capture establishes the source journal boundary once; later destination or handoff panic cannot roll it back"
    );
    source_cell
        .observe_exact(&admission)
        .expect("the released source remains healthy after the performed fork");
}

fn assert_fork_fault_costs(
    after: SignalOwnerServiceCostSnapshot,
    before: SignalOwnerServiceCostSnapshot,
) {
    assert_eq!(
        after.branch_registry_reservations(),
        before.branch_registry_reservations() + 1
    );
    assert_eq!(
        after.fork_source_captures(),
        before.fork_source_captures() + 1
    );
    assert_eq!(
        after.fork_destination_installations(),
        before.fork_destination_installations() + 1
    );
}

#[test]
fn fork_source_capture_fault_quarantines_only_source_and_releases_destination_custody() {
    let mut graph = SignalGraph::new();
    let upstream = graph.create_node();
    let replacement = graph.create_node();
    let dependent = graph.create_node();
    graph
        .set_dependencies(dependent, [DependencyEdge::new(upstream, Aspect::new(2))])
        .expect("source-fault fixture installs its initial dependency");
    let (mut runtime, sibling, source, basis) = runtime_with_two_branches_from_graph(graph);
    let sibling_basis = runtime
        .observe_signal_branch_basis(sibling.clone())
        .expect("unrelated sibling issues a real basis");
    let (_, mutation, _) = runtime.owner_port_slots().expect("fork owner seals");
    let owner = mutation.upgrade_owner().expect("fork owner remains live");
    let admission = owner.admit().expect("fork source fault admits");
    let source_cell = owner
        .lookup_cell(&admission, source.id)
        .expect("fork source is live");
    let sibling_cell = owner
        .lookup_cell(&admission, sibling.id)
        .expect("fork sibling is live");
    let advanced = owner
        .reserve_advance_output(&admission, &source_cell)
        .expect("source-fault output retention reserves")
        .advance::<(), (), _>(
            &basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |transaction| {
                transaction.set_dependencies(
                    dependent,
                    [DependencyEdge::new(replacement, Aspect::new(3))],
                )
            },
        )
        .into_result()
        .expect("source-fault fixture creates a real source journal");
    let (basis, transaction) = advanced.into_parts();
    assert!(transaction.touched_nodes > 0);
    let source_before = source_cell.fork_source_state_truth_after_fault();
    assert!(!source_before
        .mutation_ledger
        .structural_merge_journal()
        .records
        .is_empty());
    let children_before = owner
        .metadata
        .branch_children(&admission, source.id)
        .expect("source lineage is observable");
    let ledger_before = owner.retention_ledger_observation();
    let cost_before = owner.cost_snapshot();
    owner
        .operation_control()
        .inject_panic_once(SignalOwnerOperationBoundary::ForkSourceCapture);

    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = owner
            .reserve_fork_output(&admission, &source_cell)
            .expect("fork output reserves")
            .fork(
                &basis,
                validate_signal_branch_name("faulted-source-capture").expect("identity validates"),
                &SignalOwnerCancellationSource::new().token(),
            );
    }))
    .is_err());
    assert_eq!(owner.live_count(), 2);
    assert_eq!(owner.reservation_count(), 0);
    assert_eq!(
        owner
            .metadata
            .branch_children(&admission, source.id)
            .expect("faulted lineage rolls back"),
        children_before
    );
    let mut released = ledger_before.clone();
    released.next_lease_id += 1;
    assert_eq!(owner.retention_ledger_observation(), released);
    assert!(matches!(
        source_cell.observe_exact(&admission),
        Err(SignalBranchBasisObservationDenial::QuarantinedBranch { branch_id })
            if branch_id == source.id
    ));
    assert!(source_cell.poison_recovery().is_some());
    assert_eq!(
        source_cell.fork_source_state_truth_after_fault(),
        source_before,
        "ForkSourceCapture panic preserves exact graph, head, observation, and journal while quarantining the incarnation"
    );
    assert_eq!(
        owner.cost_snapshot().fork_source_captures(),
        cost_before.fork_source_captures() + 1
    );
    assert_eq!(
        owner.cost_snapshot().fork_destination_installations(),
        cost_before.fork_destination_installations()
    );

    let retry = owner
        .reserve_fork_output(&admission, &sibling_cell)
        .expect("unrelated fork output capacity is reusable")
        .fork(
            &sibling_basis,
            validate_signal_branch_name("healthy-sibling-fork").expect("retry identity validates"),
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("unrelated sibling forks after source quarantine");
    let (handle, retry_basis) = retry.into_destination_parts();
    assert_eq!(handle.parent_branch_id, Some(sibling.id));
    assert_eq!(retry_basis.owner_branch_id(), handle.id);
    assert_eq!(owner.live_count(), 3);
}
