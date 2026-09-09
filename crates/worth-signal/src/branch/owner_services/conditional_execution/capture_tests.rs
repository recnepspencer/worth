use super::SignalConditionalServiceIssuanceDenial as Denial;
use crate::data::aspect::{Aspect, SignalAspectLoweringOwner};
use crate::data::dependency::DependencyEdge;
use crate::data::graph::{storage::execution_basis::SignalExecutionBasis, SignalGraph};
use crate::data::retained_storage::{
    RetainedStoragePreparation as Work, SignalConditionalRetentionReservation,
};
use crate::logic::transaction::SignalRuntime;
use crate::runtime_policy::{SignalConditionalEvaluationBudget, SignalRuntimePolicy};

fn source_authority() -> worth_proof::ConditionalSourceObservationAuthority {
    worth_proof::ConditionalSourceObservationOwner::fresh().authority()
}

fn graph(claimant: &SignalAspectLoweringOwner) -> SignalGraph {
    let mut graph = SignalGraph::new();
    let producer = graph.create_node();
    let consumer = graph.create_node();
    graph
        .set_dependencies(
            consumer,
            [DependencyEdge::whole_partition(
                producer,
                Aspect::new(0),
                "retained source scope".repeat(256),
            )],
        )
        .unwrap();
    graph.claim_aspect_lowering_owner(claimant).unwrap();
    graph
}

fn budget(bytes: u64, visits: usize) -> SignalConditionalEvaluationBudget {
    SignalConditionalEvaluationBudget {
        maximum_retained_slots: 2,
        maximum_retained_bytes: bytes,
        maximum_attempt_visits: visits,
    }
}

#[test]
fn conditional_capture_reserves_before_conversion_and_releases_last_backing_custody() {
    let claimant = SignalAspectLoweringOwner::fresh();
    let mut sample = graph(&claimant);
    let prepared =
        SignalExecutionBasis::prepare_capture(&mut sample, &mut Work::new(100_000)).unwrap();
    let charges = prepared.charges();
    let handles = 2 * std::mem::size_of::<SignalConditionalRetentionReservation>() as u64;
    let exact = charges.retained.bytes() + charges.source_growth.bytes() + handles;
    drop(prepared);
    for maximum in [exact - 1, exact] {
        // Fresh native construction keeps the denial/admitted twin independent.
        let mut runtime = SignalRuntime::build_for::<()>(graph(&claimant));
        runtime.set_runtime_policy(
            SignalRuntimePolicy::development()
                .with_conditional_evaluation_budget(budget(maximum, 100_000)),
        );
        let basis = runtime
            .observe_signal_branch_basis(runtime.current_branch())
            .unwrap();
        let (_, mutation, _) = runtime.owner_port_slots().unwrap();
        let owner = mutation.upgrade_owner().unwrap();
        let ledger = owner.conditional_retention.clone();
        drop(owner);
        let result =
            runtime.issue_conditional_execution_service(&basis, &claimant, &source_authority());
        if maximum < exact {
            assert!(matches!(result, Err(Denial::CaptureCapacityExhausted)));
            assert_eq!(
                ledger.usage(),
                (0, 0),
                "denial constructs no charged backing"
            );
            continue;
        }
        let port = result.unwrap();
        assert_eq!(
            port._issuance_basis_custody
                .storage
                .retained_storage_charge(),
            charges.retained
        );
        assert_eq!(
            ledger.usage().0,
            0,
            "a retained seed is not an evaluation slot"
        );
        let published = ledger.usage();
        drop(port);
        assert_eq!(
            ledger.usage(),
            published,
            "the cell retains its published admission basis"
        );
        // Reissuing the same generation reuses one published retained basis.
        for _ in 0..8 {
            drop(
                runtime
                    .issue_conditional_execution_service(&basis, &claimant, &source_authority())
                    .unwrap(),
            );
            assert_eq!(ledger.usage(), published);
        }
        let port = runtime
            .issue_conditional_execution_service(&basis, &claimant, &source_authority())
            .unwrap();
        let before_close = ledger.usage();
        drop(runtime);
        assert!(port.owner.upgrade().is_none());
        assert_eq!(
            ledger.usage(),
            before_close,
            "port still retains shared backings"
        );
        drop(port);
        assert_eq!(
            ledger.usage(),
            (0, 0),
            "last backing and seed release capacity"
        );
    }
}

#[test]
fn conditional_capture_work_denial_precedes_any_resource_reservation() {
    let claimant = SignalAspectLoweringOwner::fresh();
    let mut runtime = SignalRuntime::build_for::<()>(graph(&claimant));
    runtime.set_runtime_policy(
        SignalRuntimePolicy::development()
            .with_conditional_evaluation_budget(budget(64 * 1024 * 1024, 1)),
    );
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let (_, mutation, _) = runtime.owner_port_slots().unwrap();
    let owner = mutation.upgrade_owner().unwrap();
    assert!(matches!(
        runtime.issue_conditional_execution_service(&basis, &claimant, &source_authority()),
        Err(Denial::CaptureWorkExhausted { maximum_visits: 1 })
    ));
    assert_eq!(owner.conditional_retention.usage(), (0, 0));
}

#[test]
fn conditional_capture_custody_survives_real_branch_fork_and_source_snapshot_replacement() {
    let claimant = SignalAspectLoweringOwner::fresh();
    let mut runtime = SignalRuntime::build_for::<()>(graph(&claimant));
    let source_branch = runtime.current_branch();
    let source = runtime
        .observe_signal_branch_basis(source_branch.clone())
        .unwrap();
    let (_, mutation, _) = runtime.owner_port_slots().unwrap();
    let owner = mutation.upgrade_owner().unwrap();
    let weak_owner = std::sync::Arc::downgrade(&owner);
    let ledger = owner.conditional_retention.clone();
    drop(
        runtime
            .issue_conditional_execution_service(&source, &claimant, &source_authority())
            .unwrap(),
    );
    let source_custody = ledger.usage();
    assert!(source_custody.1 > 0);
    let child = runtime
        .fork_signal_branch("retained-child", &source)
        .unwrap()
        .created_branch()
        .clone();
    let admission = owner.admit().unwrap();
    let child_cell = owner.lookup_cell(&admission, child.id).unwrap();
    drop(admission);
    let current_source = runtime.observe_signal_branch_basis(source_branch).unwrap();
    let cancellation = crate::branch::owner_services::SignalOwnerCancellationSource::new();
    // Native snapshot capture swaps an operational copy into the source cell.
    // The destination still holds the original charged persistent backing.
    drop(
        mutation
            .capture_exact(&current_source, &cancellation.token())
            .unwrap(),
    );
    let child_custody = ledger.usage();
    assert_eq!(child_custody.0, 0);
    assert!(child_custody.1 > 0, "destination retains charged backing");
    assert!(
        child_custody.1 < source_custody.1,
        "source-local pending admissions are not inherited by the fork"
    );
    drop(owner);
    drop(runtime);
    assert!(weak_owner.upgrade().is_none());
    assert_eq!(
        ledger.usage(),
        child_custody,
        "retained destination cell outlives owner"
    );
    drop(child_cell);
    assert_eq!(ledger.usage(), (0, 0));
}

#[test]
fn conditional_capture_cleanup_observer_fences_reservation_before_detaching_cells() {
    use crate::data::retained_storage::{RetainedStorageCharge, SignalConditionalRetentionDenial};

    let claimant = SignalAspectLoweringOwner::fresh();
    let mut runtime = SignalRuntime::build_for::<()>(graph(&claimant));
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let (_, mutation, _) = runtime.owner_port_slots().unwrap();
    let owner = mutation.upgrade_owner().unwrap();
    let ledger = owner.conditional_retention.clone();
    let port = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_authority())
        .unwrap();
    let retained = ledger.usage();
    let mut observed = false;
    owner
        .close_with_cleanup_observer(|_, _| {
            observed = true;
            assert!(matches!(
                ledger.reserve(0, RetainedStorageCharge::ZERO),
                Err(SignalConditionalRetentionDenial::Closed)
            ));
            assert_eq!(ledger.usage(), retained, "live seed keeps its custody");
        })
        .unwrap();
    assert!(observed);
    assert!(matches!(
        ledger.reserve(0, RetainedStorageCharge::ZERO),
        Err(SignalConditionalRetentionDenial::Closed)
    ));
    drop(runtime);
    drop(owner);
    assert_eq!(ledger.usage(), retained);
    drop(port);
    assert_eq!(ledger.usage(), (0, 0));
}
