use crate::data::aspect::SignalAspectLoweringOwner;
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;

use super::SignalConditionalServiceIssuanceDenial as Denial;
use crate::branch::owner_services::SignalOwnerCancellationSource;

fn source_authority() -> worth_proof::ConditionalSourceObservationAuthority {
    worth_proof::ConditionalSourceObservationOwner::fresh().authority()
}

#[test]
fn issuance_requires_a_sealed_exact_claim_and_remains_weak_to_owner_lifecycle() {
    let claimant = SignalAspectLoweringOwner::fresh();
    let mut graph = SignalGraph::new();
    graph.claim_aspect_lowering_owner(&claimant).unwrap();
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    assert!(matches!(
        runtime.issue_conditional_execution_service(&basis, &claimant, &source_authority()),
        Err(Denial::OwnerUnavailable(_)),
    ));
    // The denied request did not implicitly seal or consume construction state.
    runtime
        .graph_mut()
        .claim_aspect_lowering_owner(&claimant)
        .unwrap();
    let (_, mutation, _) = runtime.owner_port_slots().unwrap();
    assert!(matches!(
        runtime.issue_conditional_execution_service(
            &basis,
            &SignalAspectLoweringOwner::fresh(),
            &source_authority(),
        ),
        Err(Denial::ClaimantMismatch),
    ));
    let port = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_authority())
        .unwrap();
    assert_eq!(port.basis.admission_identity(), basis.admission_identity());
    assert!(port.definition.is_claimed_by(&claimant));

    let foreign_runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
    let foreign = foreign_runtime
        .observe_signal_branch_basis(foreign_runtime.current_branch())
        .unwrap();
    assert!(matches!(
        runtime.issue_conditional_execution_service(&foreign, &claimant, &source_authority()),
        Err(Denial::ForeignBasis),
    ));

    let cancellation = SignalOwnerCancellationSource::new();
    let (_, current) = mutation
        .capture_exact(&basis, &cancellation.token())
        .unwrap()
        .into_parts();
    assert!(matches!(
        runtime.issue_conditional_execution_service(&basis, &claimant, &source_authority()),
        Err(Denial::BasisMismatch { .. }),
    ));
    let current_port = runtime
        .issue_conditional_execution_service(&current, &claimant, &source_authority())
        .unwrap();
    assert_eq!(port.incarnation, current_port.incarnation);
    assert!(current_port.definition.is_claimed_by(&claimant));
    drop(runtime);
    assert!(port.owner.upgrade().is_none());
    assert!(current_port.owner.upgrade().is_none());
}

#[test]
fn reconstitution_readmits_same_owner_and_exact_definition_without_advancing_basis() {
    let claimant = SignalAspectLoweringOwner::fresh();
    let mut graph = SignalGraph::new();
    graph.claim_aspect_lowering_owner(&claimant).unwrap();
    let graph_instance = graph.runtime_instance_id();
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let (_, mutation, _) = runtime.owner_port_slots().unwrap();
    let port = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_authority())
        .unwrap();
    let (readmitted, report) = port.reconstitute_at_basis(&basis).unwrap();
    assert_eq!(report.owner_runtime_instance_id(), graph_instance);
    assert_eq!(
        report.definition_basis(),
        port.definition.definition_basis()
    );
    assert_eq!(report.service_readmission_count(), 1);
    assert_eq!(
        readmitted.basis.admission_identity(),
        basis.admission_identity()
    );
    assert!(!std::sync::Arc::ptr_eq(
        &port.authority,
        &readmitted.authority
    ));
    assert_eq!(
        port.active_node_count().unwrap(),
        readmitted.active_node_count().unwrap()
    );
    let cancellation = SignalOwnerCancellationSource::new();
    let (_, current) = mutation
        .capture_exact(&basis, &cancellation.token())
        .unwrap()
        .into_parts();
    assert!(port.reconstitute_at_basis(&basis).is_err());
    assert!(port.reconstitute_at_basis(&current).is_ok());
    drop(runtime);
    assert!(readmitted.active_node_count().is_err());
}
