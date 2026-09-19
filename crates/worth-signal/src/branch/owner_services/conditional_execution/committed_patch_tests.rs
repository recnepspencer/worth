use crate::data::aspect::{Aspect, AspectMask, AspectVersion, SignalAspectLoweringOwner};
use crate::data::comparator::DefaultComparatorPolicyResolver;
use crate::data::conditional_execution::{
    InstalledSignalConditionDecision, InstalledSignalConditionResolver,
    InstalledSignalConditionalContract, SignalConditionalArtifactReuse, SignalConditionalCondition,
    SignalConditionalContractDefinition, SignalConditionalDecisionClass,
    SignalConditionalVersionComparator,
};
use crate::data::error::SignalError;
use crate::data::graph::{storage::execution_basis::SignalExecutionBasis, SignalGraph};
use crate::data::node::NodeState;
use crate::data::output::{ChangedRegion, NodeEvaluationResult};
use crate::data::retained_storage::{
    RetainedStoragePreparation, SignalConditionalRetentionReservation,
};
use crate::logic::transaction::SignalRuntime;
use crate::runtime_policy::{SignalConditionalEvaluationBudget, SignalRuntimePolicy};
use worth_proof::{ConditionalEvaluationSource, ConditionalSourceObservationOwner};

use super::{
    SignalCommittedPatchDeliveryDenial as DeliveryDenial,
    SignalCommittedPatchDeliveryRequest as DeliveryRequest, SignalCommittedPatchTarget as Target,
    SignalConditionalServiceExecutionRequest as Request,
};

struct NoPredicate;

impl InstalledSignalConditionResolver for NoPredicate {
    fn resolve(
        &mut self,
        _: &crate::data::node::InstalledSignalConditionIdentity,
        _: &crate::logic::evaluation::ConditionEvaluationContext,
    ) -> Result<InstalledSignalConditionDecision, SignalError> {
        panic!("Always does not contact a predicate provider")
    }
}

fn definition() -> SignalConditionalContractDefinition {
    SignalConditionalContractDefinition {
        condition: SignalConditionalCondition::Always,
        dependency_aspects: AspectMask::from_aspect(Aspect::new(1)),
        trigger_aspects: AspectMask::from_aspect(Aspect::new(1)),
        dependency_comparator: SignalConditionalVersionComparator::Exact,
        output_comparator: SignalConditionalVersionComparator::Exact,
        artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
    }
}

fn installed_graph() -> (
    SignalGraph,
    SignalAspectLoweringOwner,
    [InstalledSignalConditionalContract; 2],
    ConditionalSourceObservationOwner,
) {
    let mut graph = SignalGraph::new();
    let claimant = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&claimant).unwrap();
    let contracts = [(); 2].map(|()| {
        let node = graph.node().build();
        let worth_proof::TransitionOutcome::Success(capability) = graph.admit_installed_node(node)
        else {
            panic!("fresh node must admit")
        };
        graph
            .install_conditional_contract(&claimant, capability, definition())
            .unwrap()
    });
    (
        graph,
        claimant,
        contracts,
        ConditionalSourceObservationOwner::fresh(),
    )
}

fn installed_runtime() -> (
    SignalRuntime<(), (), (), (), ()>,
    SignalAspectLoweringOwner,
    [InstalledSignalConditionalContract; 2],
    ConditionalSourceObservationOwner,
) {
    let (graph, claimant, contracts, source_owner) = installed_graph();
    (
        SignalRuntime::build_for::<()>(graph),
        claimant,
        contracts,
        source_owner,
    )
}

fn target(graph_instance_id: u64, contract: &InstalledSignalConditionalContract) -> Target {
    Target::new(
        graph_instance_id,
        contract.node(),
        Aspect::new(1),
        [ChangedRegion::new("bridge-main")],
    )
}

fn output(value: u64) -> NodeEvaluationResult {
    NodeEvaluationResult::from_version(AspectVersion::from_updates([(Aspect::new(0), value)]))
}

#[test]
fn committed_patch_preserves_exact_basis_and_reaches_conditional_execution() {
    let (mut runtime, claimant, [contract, _], source_owner) = installed_runtime();
    let graph_instance_id = contract.graph_instance_id();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let (basis_port, _, _) = runtime.owner_port_slots().unwrap();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    let evaluation = service
        .admit_evaluation(
            &contract,
            ConditionalEvaluationSource::AdmittedRelationalSource(source_owner.admit("source")),
        )
        .unwrap();
    let retained_before = retained_node_observation(&evaluation, contract.node(), Aspect::new(1))
        .expect("admission retains the installed target");
    assert_eq!(retained_before.1, 0);
    assert!(retained_before.2);
    let mut computes = 0;

    for attempt in 1..=2 {
        let (decision, _) = service
            .execute(
                &evaluation,
                Request::new(attempt),
                &mut NoPredicate,
                &mut DefaultComparatorPolicyResolver::default(),
                || {
                    computes += 1;
                    Ok(output(7))
                },
            )
            .unwrap()
            .into_parts();
        let expected = if attempt == 1 {
            SignalConditionalDecisionClass::ComputedChanged
        } else {
            SignalConditionalDecisionClass::DependencyUnchanged
        };
        assert_eq!(decision.unwrap().class(), expected);
    }

    let completion = service
        .deliver_committed_patch(
            &contract,
            DeliveryRequest::new([target(graph_instance_id, &contract)]),
        )
        .unwrap();
    assert_eq!(completion.graph_instance_id(), graph_instance_id);
    assert_eq!(completion.target_count(), 1);
    basis_port.compare_current_exact(&basis).unwrap();

    let (retained_decision, _) = service
        .execute(
            &evaluation,
            Request::new(3),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || {
                computes += 1;
                Ok(output(7))
            },
        )
        .unwrap()
        .into_parts();
    assert_eq!(
        retained_decision.unwrap().class(),
        SignalConditionalDecisionClass::DependencyUnchanged
    );
    assert_eq!(computes, 1);
    assert_eq!(
        retained_node_observation(&evaluation, contract.node(), Aspect::new(1)),
        Some(retained_before),
        "the admitted slot keeps its pre-delivery basis"
    );

    let refreshed = service
        .admit_evaluation(
            &contract,
            ConditionalEvaluationSource::AdmittedRelationalSource(
                source_owner.admit("source-after-patch"),
            ),
        )
        .unwrap();
    assert_eq!(
        retained_node_observation(&refreshed, contract.node(), Aspect::new(1)),
        Some((NodeState::Dirty, 1, true)),
        "fresh admission must retain the delivered version and dirty aspect"
    );
    let (refreshed_decision, _) = service
        .execute(
            &refreshed,
            Request::new(1),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || {
                computes += 1;
                Ok(output(7))
            },
        )
        .unwrap()
        .into_parts();
    let refreshed_decision = refreshed_decision.unwrap();
    assert_ne!(
        refreshed_decision.class(),
        SignalConditionalDecisionClass::DependencyUnchanged
    );
    assert_eq!(refreshed_decision.counters().compute_contacts, 1);
    assert_eq!(computes, 2);
    basis_port.compare_current_exact(&basis).unwrap();
}

#[test]
fn invalid_patch_descriptions_deny_without_disturbing_the_exact_service() {
    let (mut runtime, claimant, [contract, foreign], source_owner) = installed_runtime();
    let graph_instance_id = contract.graph_instance_id();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let (basis_port, _, _) = runtime.owner_port_slots().unwrap();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();

    assert!(matches!(
        service.deliver_committed_patch(
            &contract,
            DeliveryRequest::new([target(graph_instance_id + 1, &contract)]),
        ),
        Err(DeliveryDenial::ForeignGraph)
    ));
    assert!(matches!(
        service.deliver_committed_patch(
            &contract,
            DeliveryRequest::new([target(graph_instance_id, &foreign)]),
        ),
        Err(DeliveryDenial::ForeignContractTarget)
    ));
    let duplicate = target(graph_instance_id, &contract);
    assert!(matches!(
        service.deliver_committed_patch(
            &contract,
            DeliveryRequest::new([duplicate.clone(), duplicate]),
        ),
        Err(DeliveryDenial::DuplicateTarget)
    ));

    basis_port.compare_current_exact(&basis).unwrap();
    assert_eq!(
        service
            .deliver_committed_patch(
                &contract,
                DeliveryRequest::new([target(graph_instance_id, &contract)]),
            )
            .unwrap()
            .target_count(),
        1
    );
    basis_port.compare_current_exact(&basis).unwrap();
}

#[test]
fn successor_retention_denial_publishes_neither_patch_nor_basis() {
    let (mut measured_graph, _, _, _) = installed_graph();
    let prepared = SignalExecutionBasis::prepare_capture(
        &mut measured_graph,
        &mut RetainedStoragePreparation::new(100_000),
    )
    .unwrap();
    let charges = prepared.charges();
    let reservation_handles =
        2 * std::mem::size_of::<SignalConditionalRetentionReservation>() as u64;
    let exact_initial_bytes =
        charges.retained.bytes() + charges.source_growth.bytes() + reservation_handles;
    drop(prepared);

    let (graph, claimant, [contract, _], source_owner) = installed_graph();
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    runtime.set_runtime_policy(
        SignalRuntimePolicy::development().with_conditional_evaluation_budget(
            SignalConditionalEvaluationBudget {
                maximum_retained_slots: 1,
                maximum_retained_bytes: exact_initial_bytes,
                maximum_attempt_visits: 100_000,
            },
        ),
    );
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let _ports = runtime.owner_port_slots().unwrap();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    let graph_instance_id = contract.graph_instance_id();
    let before = current_basis_and_graph_observation(&service, &contract);
    let owner = service.owner.upgrade().unwrap();
    let retention_before = owner.conditional_retention.usage();
    drop(owner);

    assert!(matches!(
        service.deliver_committed_patch(
            &contract,
            DeliveryRequest::new([target(graph_instance_id, &contract)]),
        ),
        Err(DeliveryDenial::SuccessorCaptureCapacityExhausted)
    ));
    assert_eq!(
        current_basis_and_graph_observation(&service, &contract),
        before
    );
    assert_eq!(
        service
            .owner
            .upgrade()
            .unwrap()
            .conditional_retention
            .usage(),
        retention_before
    );
}

fn current_basis_and_graph_observation(
    service: &super::SignalConditionalExecutionPort<(), (), ()>,
    contract: &InstalledSignalConditionalContract,
) -> ((NodeState, u64, bool), (NodeState, u64, bool)) {
    let owner = service.owner.upgrade().expect("live service owner");
    let admission = owner.admit().unwrap();
    let cell = owner
        .lookup_cell(&admission, service.basis.owner_branch_id())
        .unwrap();
    cell.with_state(&admission, |state, _| {
        let aspect = Aspect::new(1);
        let retained = state
            .current_conditional_basis()
            .unwrap()
            .retained_node_observation(contract.node(), aspect)
            .unwrap();
        let graph = state.state().graph();
        let canonical = (
            graph.get_state(contract.node()).unwrap(),
            graph
                .node_aspect_version(contract.node())
                .unwrap()
                .get(aspect),
            graph
                .node_dirty_aspects(contract.node())
                .unwrap()
                .contains(AspectMask::from_aspect(aspect)),
        );
        (retained, canonical)
    })
    .unwrap()
}

fn retained_node_observation(
    admission: &super::SignalConditionalEvaluationAdmission,
    node: crate::data::handle::NodeId,
    aspect: Aspect,
) -> Option<(NodeState, u64, bool)> {
    admission
        .retained_basis
        .retained_node_observation(node, aspect)
}
