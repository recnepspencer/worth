//! Equal generation divergent fork installations cannot cross.

use super::*;

#[test]
fn equal_generation_divergent_fork_installations_cannot_cross() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut root = SignalGraph::new();
    let claimant = SignalAspectLoweringOwner::fresh();
    root.claim_aspect_lowering_owner(&claimant).unwrap();
    let node = root.node().build();
    let worth_proof::TransitionOutcome::Success(capability) = root.admit_installed_node(node)
    else {
        panic!("node must admit")
    };
    root.install_conditional_contract(&claimant, capability, definition())
        .unwrap();
    let (mut left, _) = root.fork_persistent();
    let (mut right, _) = root.fork_persistent();
    left.claim_aspect_lowering_owner(&claimant).unwrap();
    right.claim_aspect_lowering_owner(&claimant).unwrap();
    let worth_proof::TransitionOutcome::Success(left_capability) = left.admit_installed_node(node)
    else {
        panic!("left node must admit")
    };
    let worth_proof::TransitionOutcome::Success(right_capability) =
        right.admit_installed_node(node)
    else {
        panic!("right node must admit")
    };
    let left_contract = left
        .install_conditional_contract(&claimant, left_capability, definition())
        .unwrap();
    let right_contract = right
        .install_conditional_contract(&claimant, right_capability, definition())
        .unwrap();
    assert_eq!(left_contract.generation(), right_contract.generation());
    assert_ne!(left_contract.occurrence(), right_contract.occurrence());

    let mut runtime = SignalRuntime::build_for::<()>(left);
    let source_owner = ConditionalSourceObservationOwner::fresh();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    runtime.owner_port_slots().unwrap();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    assert!(matches!(
        service.admit_evaluation(&right_contract, source(&source_owner, "right", &mut 0)),
        Err(Denial::DefinitionMismatch)
    ));
    let left_evaluation = service
        .admit_evaluation(&left_contract, source(&source_owner, "left", &mut 0))
        .unwrap();
    let (decision, _) = service
        .execute(
            request_execution,
            &left_evaluation,
            Request::new(1),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || Ok(output(11)),
        )
        .unwrap()
        .into_parts();
    assert_eq!(decision.unwrap().output_version(), 11);
}
