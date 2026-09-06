use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::branch::{SignalBranchAdvanceDenial, SignalOwnerLifecycleObservation};
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;

use super::super::{SignalOwnerAdmissionDenial, SignalOwnerCancellationSource};
use super::runtime_root::{runtime_with_two_branches, runtime_with_two_branches_from_graph};

const MAXIMUM_ACTIVE_LEASES: usize = 4_096;

#[test]
fn named_output_capacity_returns_after_real_cancellation_and_callback_panic() {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("output cleanup operation admits");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("the output cleanup target cell is live");

    let cancelled_output = owner
        .reserve_advance_output(&admission, &cell)
        .expect("advance output reserves before cancellation");
    let cancelled = SignalOwnerCancellationSource::new();
    cancelled.cancel();
    let before_cancel = cell.cost_snapshot();
    assert!(matches!(
        cancelled_output
            .advance::<(), (), _>(&basis, &mut (), &cancelled.token(), |_| panic!(
                "cancelled callback must remain unreachable"
            ),)
            .into_result(),
        Err(SignalBranchAdvanceDenial::CancelledNoMovement)
    ));
    assert_eq!(cell.cost_snapshot(), before_cancel);
    let all_after_cancel = owner
        .reserve_admitted_output_slots_for_test(&admission, branch.id, MAXIMUM_ACTIVE_LEASES - 1)
        .expect("real cancellation returns the named output slot exactly");
    drop(all_after_cancel);

    let panicking_output = owner
        .reserve_advance_output(&admission, &cell)
        .expect("advance output reserves before callback unwind");
    let open = SignalOwnerCancellationSource::new();
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = panicking_output
            .advance::<(), (), _>(&basis, &mut (), &open.token(), |_| {
                panic!("inject output-reservation callback unwind")
            })
            .into_result();
    }));
    assert!(panic.is_err());
    let all_after_panic = owner
        .reserve_admitted_output_slots_for_test(&admission, branch.id, MAXIMUM_ACTIVE_LEASES - 1)
        .expect("callback unwind returns the named output slot exactly");
    drop(all_after_panic);
}

#[test]
fn pending_output_keeps_its_admitted_call_in_closing_until_capacity_releases() {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("snapshot output preflight admits");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("snapshot output target cell is live");
    let output = owner
        .reserve_snapshot_outputs(&admission, &cell)
        .expect("snapshot reserves both outputs before owner loss");
    assert_eq!(owner.admitted_or_reserved_retention_count(branch.id), 3);
    drop(runtime);
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closing
    );
    assert_eq!(
        owner.admitted_or_reserved_retention_count(branch.id),
        3,
        "Closing preserves the real retention ledger and pending reservation"
    );
    drop(output);
    assert_eq!(
        owner.admitted_or_reserved_retention_count(branch.id),
        1,
        "post-close reservation drop returns both unused slots exactly"
    );
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closing,
        "the borrowed output cannot outlive its admitted call"
    );
    drop(admission);
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    drop(basis);
    assert_eq!(owner.admitted_or_reserved_retention_count(branch.id), 0);
}

#[test]
fn prior_admission_reserves_and_converts_populated_advance_after_closing() {
    let mut graph = SignalGraph::new();
    let weather = graph.create_node();
    let berth = graph.create_node();
    let dispatch = graph.create_node();
    graph
        .set_dependencies(dispatch, [DependencyEdge::new(weather, Aspect::new(0))])
        .expect("the Closing output fixture installs meaningful state");
    let (mut runtime, _, branch, basis) = runtime_with_two_branches_from_graph(graph);
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("advance output operation admits");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("advance output target cell is live");
    drop(runtime);
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closing
    );
    let before_denial = owner.retention_ledger_observation();
    let before_contacts = owner.cost_snapshot().retention_registry_contacts();
    assert!(matches!(
        owner.admit(),
        Err(SignalOwnerAdmissionDenial::OwnerUnavailable)
    ));
    assert_eq!(owner.retention_ledger_observation(), before_denial);
    assert_eq!(
        owner.cost_snapshot().retention_registry_contacts(),
        before_contacts
    );

    let output = owner
        .reserve_advance_output(&admission, &cell)
        .expect("the pre-close admission reserves its named output during Closing");
    assert_eq!(
        owner.admitted_or_reserved_retention_count(branch.id),
        2,
        "the pending Closing output owns one exact reserved slot"
    );
    let cancellation = SignalOwnerCancellationSource::new();
    let completed = output
        .advance::<(), (), _>(&basis, &mut (), &cancellation.token(), |transaction| {
            transaction.set_dependencies(dispatch, [DependencyEdge::new(berth, Aspect::new(0))])
        })
        .into_result()
        .expect("meaningful pre-admitted work performs while Closing");
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closing,
        "performed work cannot complete the lifecycle before its admitted call ends"
    );
    let (advanced_basis, transaction) = completed.into_parts();
    assert!(transaction.touched_nodes > 0);
    cell.with_state(&admission, |state, _| {
        assert_eq!(
            state.state().graph().dependency_sources_of(dispatch),
            Ok(vec![berth]),
            "the Closing operation commits its independently expected semantic value"
        );
    })
    .expect("the performed Closing cell remains inspectable");
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closing,
        "the original synchronous call remains admitted after output handoff"
    );
    drop(advanced_basis);
    drop(basis);
    drop(admission);
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    assert_eq!(owner.admitted_or_reserved_retention_count(branch.id), 0);
    let closed_ledger = owner.retention_ledger_observation();
    let closed_contacts = owner.cost_snapshot().retention_registry_contacts();
    assert!(matches!(
        owner.admit(),
        Err(SignalOwnerAdmissionDenial::OwnerUnavailable)
    ));
    assert_eq!(owner.retention_ledger_observation(), closed_ledger);
    assert_eq!(
        owner.cost_snapshot().retention_registry_contacts(),
        closed_contacts,
        "Closed admission denial cannot touch the retention ledger"
    );
}

#[test]
fn output_preflight_rejects_same_branch_id_from_a_foreign_owner_before_capacity_or_cell_work() {
    let (mut runtime_a, _, branch_a, basis_a) = runtime_with_two_branches();
    let (mut runtime_b, _, branch_b, basis_b) = runtime_with_two_branches();
    assert_eq!(
        branch_a.id, branch_b.id,
        "the adversarial IDs intentionally collide"
    );
    let (_, mutation_a, _) = runtime_a.owner_port_slots().expect("owner A seals");
    let (_, mutation_b, _) = runtime_b.owner_port_slots().expect("owner B seals");
    let owner_a = mutation_a.upgrade_owner().expect("owner A remains live");
    let owner_b = mutation_b.upgrade_owner().expect("owner B remains live");
    let admission_a = owner_a.admit().expect("owner A operation admits");
    let admission_b = owner_b.admit().expect("owner B operation admits");
    let cell_a = owner_a
        .lookup_cell(&admission_a, branch_a.id)
        .expect("owner A cell is live");
    let cell_b = owner_b
        .lookup_cell(&admission_b, branch_b.id)
        .expect("owner B cell is live");
    let before_a = owner_a.admitted_or_reserved_retention_count(branch_a.id);
    let before_b = cell_b.cost_snapshot();

    assert!(matches!(
        owner_a.reserve_advance_output(&admission_a, &cell_b),
        Err(SignalBranchAdvanceDenial::OwnerUnavailable(_))
    ));
    assert_eq!(
        owner_a.admitted_or_reserved_retention_count(branch_a.id),
        before_a
    );
    assert_eq!(cell_b.cost_snapshot(), before_b);

    let healthy = owner_a
        .reserve_advance_output(&admission_a, &cell_a)
        .expect("the exact owner/cell pairing remains healthy");
    drop(healthy);
    drop((basis_a, basis_b));
}
