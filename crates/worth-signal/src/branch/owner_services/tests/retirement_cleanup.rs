use std::sync::mpsc;
use std::thread;

use worth_proof::TransitionOutcome;

use crate::branch::SignalBranchRetirementReason;
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;

use super::super::SignalOwnerCancellationSource;
use super::progress_bound::PROGRESS_BOUND;
use super::retirement_receipt_oracle::{expected_closeout_digest, expected_terminal_basis_digest};

#[test]
fn populated_retirement_snapshot_cleanup_drops_after_metadata_unlock() {
    let mut graph = SignalGraph::new();
    let nodes = (0..96).map(|_| graph.create_node()).collect::<Vec<_>>();
    for pair in nodes.windows(2) {
        graph
            .set_dependencies(pair[1], [DependencyEdge::new(pair[0], Aspect::new(0))])
            .expect("populated retirement graph accepts its dependency chain");
    }
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    let selected = runtime.current_branch();
    let selected_basis = runtime
        .observe_signal_branch_basis(selected.clone())
        .expect("selected basis admits");
    let (retired, fork_basis) = runtime
        .fork_signal_branch("populated-retirement-cleanup", &selected_basis)
        .expect("retirement target forks from populated state")
        .into_parts();
    runtime
        .switch_branch(retired.clone())
        .expect("the retirement target becomes the actual mutation target");
    assert_eq!(
        runtime.graph().dependency_sources_of(nodes[95]),
        Ok(vec![nodes[94]]),
        "the fork begins with the source branch dependency"
    );
    runtime
        .graph_mut()
        .set_dependencies(nodes[95], [DependencyEdge::new(nodes[0], Aspect::new(0))])
        .expect("the retirement target accepts a distinct real mutation");
    assert_eq!(
        runtime.graph().dependency_sources_of(nodes[95]),
        Ok(vec![nodes[0]]),
        "the selected target now carries independent semantic state"
    );
    drop(fork_basis);
    let retired_basis = runtime
        .observe_signal_branch_basis(retired.clone())
        .expect("the mutated retirement target refreshes its basis");
    let (snapshot, captured_basis) = runtime
        .capture_signal_branch_snapshot(&retired_basis)
        .expect("retirement target stores populated snapshot state")
        .into_parts();
    assert_eq!(
        snapshot
            .snapshot()
            .diagnostic_graph
            .dependency_sources_of(nodes[95]),
        Ok(vec![nodes[0]]),
        "the retained snapshot contains the target-local mutation"
    );
    let retired_after_capture = runtime
        .branch_handle(retired.id)
        .expect("the captured retirement target remains live");
    assert_eq!(
        retired_after_capture.head_snapshot_id,
        Some(snapshot.snapshot().meta.snapshot_id),
        "the expected retirement handle carries the actual captured head"
    );
    drop(retired_basis);
    runtime
        .switch_branch(selected.clone())
        .expect("the source branch is selected before retiring its sibling");
    let expected_terminal_digest =
        expected_terminal_basis_digest(&retired_after_capture, captured_basis.observation());
    let expected_parent = retired_after_capture
        .parent_branch_id
        .expect("the populated retirement target is a fork child");
    let expected_fork_snapshot = retired.head_snapshot_id;
    let expected_terminal_snapshot = retired_after_capture.head_snapshot_id;
    let expected_closeout = expected_closeout_digest(
        retired_after_capture.id,
        expected_parent,
        expected_fork_snapshot,
        expected_terminal_snapshot,
        SignalBranchRetirementReason::Rejected,
        &expected_terminal_digest,
    );
    let plan = match runtime.plan_signal_branch_retirement_releasing_snapshots(
        retired.clone(),
        captured_basis,
        &[&snapshot],
        SignalBranchRetirementReason::Rejected,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("populated snapshot should be releasable for retirement: {other:?}"),
    };
    drop(snapshot.into_snapshot());

    let (_, _, lifecycle) = runtime.owner_port_slots().expect("runtime seals");
    let owner = lifecycle.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("retirement admits");
    let retirement = owner
        .reserve_retirement(&admission, retired.id)
        .expect("retirement reserves lineage, receipt, registry, and cleanup capacity");
    let (begin_tx, begin_rx) = mpsc::sync_channel(1);
    let (progress_tx, progress_rx) = mpsc::sync_channel(1);
    let progress_owner = owner.clone();
    let selected_id = selected.id;
    let worker = thread::spawn(move || {
        begin_rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("cleanup boundary becomes observable");
        let admission = progress_owner
            .admit()
            .expect("unrelated metadata work admits in its executing thread");
        let result = progress_owner
            .metadata
            .branch_children(&admission, selected_id);
        let _ = progress_tx.send(result);
    });
    let cancellation = SignalOwnerCancellationSource::new();
    let receipt = retirement
        .execute_with_cleanup_observer(plan, &cancellation.token(), |reclaimed| {
            assert_eq!(
                reclaimed, 1,
                "one populated snapshot is detached for cleanup"
            );
            begin_tx
                .send(())
                .expect("unrelated metadata worker remains connected");
            let progress = progress_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("metadata progresses while detached payload awaits destruction");
            assert!(
                progress.is_ok(),
                "unrelated metadata lookup remains healthy"
            );
        })
        .expect("retirement completes after outside-lock payload destruction");
    worker
        .join()
        .expect("metadata progress worker does not panic");
    assert_eq!(receipt.retired_branch(), &retired_after_capture);
    assert_eq!(receipt.parent_branch_id(), expected_parent);
    assert_eq!(receipt.forked_from_snapshot_id(), expected_fork_snapshot);
    assert_eq!(
        receipt.terminal_head_snapshot_id(),
        expected_terminal_snapshot
    );
    assert_eq!(receipt.reason(), SignalBranchRetirementReason::Rejected);
    assert_eq!(receipt.terminal_basis_digest(), expected_terminal_digest);
    assert_eq!(receipt.closeout_digest(), expected_closeout);
    assert_eq!(receipt.reclaimed_branch_state_count(), 1);
    assert_eq!(receipt.reclaimed_snapshot_state_count(), 1);
    assert_eq!(receipt.reclaimed_runtime_meta_count(), 0);
    assert_eq!(receipt.retained_proof_record_count(), 1);
}
