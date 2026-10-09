use crate::data::aspect::{Aspect, AspectMask, AspectVersion, SignalAspectLoweringOwner};
use crate::data::comparator::DefaultComparatorPolicyResolver;
use crate::data::conditional_execution::{
    InstalledSignalConditionDecision, InstalledSignalConditionResolver,
    InstalledSignalConditionalContract, SignalConditionalArtifactReuse, SignalConditionalCondition,
    SignalConditionalContractDefinition, SignalConditionalDecisionClass,
    SignalConditionalVersionComparator,
};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::output::NodeEvaluationResult;
use crate::logic::transaction::SignalRuntime;
use crate::runtime_policy::{SignalConditionalEvaluationBudget, SignalRuntimePolicy};
use worth_proof::{ConditionalEvaluationSource, ConditionalSourceObservationOwner};

use super::{
    SignalConditionalServiceExecutionDenial as Denial,
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

fn runtime_with_contract() -> (
    SignalRuntime<(), (), (), (), ()>,
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
        SignalRuntime::build_for::<()>(graph),
        claimant,
        contracts,
        ConditionalSourceObservationOwner::fresh(),
    )
}

fn source(
    owner: &ConditionalSourceObservationOwner,
    label: &str,
    loads: &mut usize,
) -> ConditionalEvaluationSource {
    *loads += 1;
    ConditionalEvaluationSource::AdmittedRelationalSource(owner.admit(label))
}

fn output(value: u64) -> NodeEvaluationResult {
    NodeEvaluationResult::from_version(AspectVersion::from_updates([(Aspect::new(0), value)]))
}

#[test]
fn admitted_sources_preserve_independent_b_a_b_slots_without_reload_or_recompute() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let (mut runtime, claimant, [contract, _], source_owner) = runtime_with_contract();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    runtime.owner_port_slots().unwrap();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    let mut source_loads = 0;
    let b = service
        .admit_evaluation(
            &contract,
            source(&source_owner, "source-b", &mut source_loads),
        )
        .unwrap();
    let a = service
        .admit_evaluation(
            &contract,
            source(&source_owner, "source-a", &mut source_loads),
        )
        .unwrap();
    let mut computes = 0;

    for (admission, value, source_label) in [(&b, 4, "source-b"), (&a, 6, "source-a")] {
        let completion = service
            .execute(
                request_execution,
                admission,
                Request::new(1),
                &mut NoPredicate,
                &mut DefaultComparatorPolicyResolver::default(),
                || {
                    computes += 1;
                    Ok(output(value))
                },
            )
            .unwrap();
        assert_eq!(completion.binding().source().projection(), source_label);
        let (decision, observation) = completion.into_parts();
        let decision = decision.unwrap();
        assert_eq!(
            decision.class(),
            SignalConditionalDecisionClass::ComputedChanged
        );
        assert_eq!(decision.output_version(), value);
        assert!(observation.unwrap().is_some());
    }

    let (decision, observation) = service
        .execute(
            request_execution,
            &b,
            Request::new(2),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || {
                computes += 1;
                Ok(output(99))
            },
        )
        .unwrap()
        .into_parts();
    let decision = decision.unwrap();
    assert_eq!(
        decision.class(),
        SignalConditionalDecisionClass::DependencyUnchanged
    );
    assert_eq!(decision.output_version(), 4);
    assert_eq!(decision.counters().compute_contacts, 0);
    assert!(observation.unwrap().is_none());
    assert_eq!((source_loads, computes), (2, 2));
}

#[test]
fn provider_unwind_releases_the_admitted_slot_for_retry() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let (mut runtime, claimant, [contract, _], source_owner) = runtime_with_contract();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    runtime.owner_port_slots().unwrap();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    let evaluation = service
        .admit_evaluation(&contract, source(&source_owner, "source", &mut 0))
        .unwrap();

    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = service.execute(
            request_execution,
            &evaluation,
            Request::new(1),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || -> Result<NodeEvaluationResult, SignalError> { panic!("provider unwind") },
        );
    }));
    assert_eq!(
        unwind.unwrap_err().downcast_ref::<&str>(),
        Some(&"provider unwind")
    );

    let (decision, observation) = service
        .execute(
            request_execution,
            &evaluation,
            Request::new(2),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || Ok(output(8)),
        )
        .unwrap()
        .into_parts();
    assert_eq!(decision.unwrap().output_version(), 8);
    assert!(observation.unwrap().is_some());
}

#[test]
fn an_evaluation_admission_cannot_cross_its_issuing_service() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let (mut runtime, claimant, [contract, _], first_source_owner) = runtime_with_contract();
    let second_source_owner = ConditionalSourceObservationOwner::fresh();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    runtime.owner_port_slots().unwrap();
    let first = runtime
        .issue_conditional_execution_service(&basis, &claimant, &first_source_owner.authority())
        .unwrap();
    let second = runtime
        .issue_conditional_execution_service(&basis, &claimant, &second_source_owner.authority())
        .unwrap();
    let foreign_source = source(&first_source_owner, "source", &mut 0);
    assert!(matches!(
        second.admit_evaluation(&contract, foreign_source),
        Err(Denial::SourceAuthorityMismatch)
    ));
    let evaluation = first
        .admit_evaluation(&contract, source(&first_source_owner, "source", &mut 0))
        .unwrap();
    let mut computes = 0;
    let denial = match second.execute(
        request_execution,
        &evaluation,
        Request::new(1),
        &mut NoPredicate,
        &mut DefaultComparatorPolicyResolver::default(),
        || {
            computes += 1;
            Ok(output(9))
        },
    ) {
        Err(denial) => denial,
        Ok(_) => panic!("an evaluation admission must retain its issuing service"),
    };
    assert!(matches!(denial, Denial::DefinitionMismatch));
    assert_eq!(computes, 0);
}

#[test]
fn reinstalled_same_node_rejects_the_old_contract_before_provider_work() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = SignalGraph::new();
    let claimant = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&claimant).unwrap();
    let node = graph.node().build();
    let worth_proof::TransitionOutcome::Success(first_capability) =
        graph.admit_installed_node(node)
    else {
        panic!("node must admit")
    };
    let old = graph
        .install_conditional_contract(&claimant, first_capability, definition())
        .unwrap();
    let worth_proof::TransitionOutcome::Success(current_capability) =
        graph.admit_installed_node(node)
    else {
        panic!("installed node must readmit")
    };
    let current = graph
        .install_conditional_contract(&claimant, current_capability, definition())
        .unwrap();
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    let source_owner = ConditionalSourceObservationOwner::fresh();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    runtime.owner_port_slots().unwrap();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    let mut source_loads = 0;

    assert!(matches!(
        service.admit_evaluation(&old, source(&source_owner, "old", &mut source_loads)),
        Err(Denial::DefinitionMismatch)
    ));
    let admitted = service
        .admit_evaluation(
            &current,
            source(&source_owner, "current", &mut source_loads),
        )
        .unwrap();
    let mut computes = 0;
    let (decision, _) = service
        .execute(
            request_execution,
            &admitted,
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
    assert_eq!(decision.unwrap().output_version(), 7);
    assert_eq!((source_loads, computes), (2, 1));
}

#[path = "execution_tests/unexecuted_admissions_are_capacity_bounded_and_drop_to_a_stable_baseline.rs"]
mod unexecuted_admissions_are_capacity_bounded_and_drop_to_a_stable_baseline;

#[path = "execution_tests/equal_generation_divergent_fork_installations_cannot_cross.rs"]
mod equal_generation_divergent_fork_installations_cannot_cross;
