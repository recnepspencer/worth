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
use crate::data::output::{ChangedRegion, NodeEvaluationResult};
use crate::logic::transaction::SignalRuntime;
use crate::runtime_policy::{SignalConditionalEvaluationBudget, SignalRuntimePolicy};
use worth_proof::{ConditionalEvaluationSource, ConditionalSourceObservationOwner};

use super::{
    SignalCommittedPatchDeliveryRequest, SignalCommittedPatchTarget,
    SignalConditionalEvaluationReadmissionDenial as ReadmissionDenial,
    SignalConditionalEvaluationReadmissionRequest as ReadmissionRequest,
    SignalConditionalExecutionPort, SignalConditionalServiceExecutionRequest as ExecutionRequest,
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

fn fixture(
    budget: Option<SignalConditionalEvaluationBudget>,
) -> (
    SignalRuntime<(), (), (), (), ()>,
    SignalConditionalExecutionPort<(), (), ()>,
    InstalledSignalConditionalContract,
    ConditionalSourceObservationOwner,
) {
    let mut graph = SignalGraph::new();
    let claimant = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&claimant).unwrap();
    let node = graph.node().build();
    let worth_proof::TransitionOutcome::Success(capability) = graph.admit_installed_node(node)
    else {
        panic!("fresh node admits")
    };
    let contract = graph
        .install_conditional_contract(&claimant, capability, definition())
        .unwrap();
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    if let Some(budget) = budget {
        runtime.set_runtime_policy(
            SignalRuntimePolicy::development().with_conditional_evaluation_budget(budget),
        );
    }
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    runtime.owner_port_slots().unwrap();
    let source_owner = ConditionalSourceObservationOwner::fresh();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    (runtime, service, contract, source_owner)
}

fn target(contract: &InstalledSignalConditionalContract) -> SignalCommittedPatchTarget {
    SignalCommittedPatchTarget::new(
        contract.graph_instance_id(),
        contract.node(),
        Aspect::new(1),
        [ChangedRegion::new("bridge-main")],
    )
}

fn output(value: u64) -> NodeEvaluationResult {
    NodeEvaluationResult::from_version(AspectVersion::from_updates([(Aspect::new(0), value)]))
}

fn execute(
    service: &SignalConditionalExecutionPort<(), (), ()>,
    admission: &super::SignalConditionalEvaluationAdmission,
    attempt: u64,
    value: u64,
) -> SignalConditionalDecisionClass {
    service
        .execute(
            admission,
            ExecutionRequest::new(attempt),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || Ok(output(value)),
        )
        .unwrap()
        .into_parts()
        .0
        .unwrap()
        .class()
}

#[test]
fn transition_clones_share_one_retained_charge_until_the_last_clone_drops() {
    let (_runtime, service, contract, _source_owner) = fixture(None);
    let ledger = service
        .owner
        .upgrade()
        .unwrap()
        .conditional_retention
        .clone();
    let before = ledger.usage();
    let completion = service
        .deliver_committed_patch(
            &contract,
            SignalCommittedPatchDeliveryRequest::new([target(&contract)]),
        )
        .unwrap();
    let retained = ledger.usage();
    assert!(retained.1 > before.1);

    let clone = completion.successor_transition().clone();
    assert_eq!(
        ledger.usage(),
        retained,
        "Arc clones do not duplicate custody"
    );
    drop(completion);
    assert_eq!(
        ledger.usage(),
        retained,
        "one clone keeps exact custody live"
    );
    drop(clone);
    assert!(
        ledger.usage().1 < retained.1,
        "the final clone releases custody"
    );
}

#[test]
fn unexecuted_predecessor_requires_fresh_admission_without_consuming_retention() {
    let (_runtime, service, contract, source_owner) = fixture(None);
    let source = || ConditionalEvaluationSource::from(source_owner.admit("unexecuted-source"));
    let predecessor = service.admit_evaluation(&contract, source()).unwrap();
    let completion = service
        .deliver_committed_patch(
            &contract,
            SignalCommittedPatchDeliveryRequest::new([target(&contract)]),
        )
        .unwrap();
    let ledger = service
        .owner
        .upgrade()
        .unwrap()
        .conditional_retention
        .clone();
    let retained = ledger.usage();
    assert!(matches!(
        service.readmit_evaluation(ReadmissionRequest {
            predecessor: &predecessor,
            transitions: &[completion.successor_transition()],
        }),
        Err(ReadmissionDenial::PredecessorNotExecuted)
    ));
    assert_eq!(ledger.usage(), retained);
    assert_eq!(
        execute(&service, &predecessor, 1, 7),
        SignalConditionalDecisionClass::ComputedChanged,
        "denial preserves the predecessor for its first execution"
    );
    let fresh = service.admit_evaluation(&contract, source()).unwrap();
    assert_eq!(
        execute(&service, &fresh, 1, 11),
        SignalConditionalDecisionClass::ComputedChanged,
        "explicit fresh admission can execute the delivered successor"
    );
}

#[test]
fn readmission_slot_shortage_denies_before_disturbing_the_predecessor() {
    let budget = SignalConditionalEvaluationBudget {
        maximum_retained_slots: 1,
        maximum_retained_bytes: 512 * 1024 * 1024,
        maximum_attempt_visits: 8_000_000,
    };
    let (_runtime, service, contract, source_owner) = fixture(Some(budget));
    let predecessor = service
        .admit_evaluation(
            &contract,
            ConditionalEvaluationSource::AdmittedRelationalSource(source_owner.admit("source")),
        )
        .unwrap();
    assert_eq!(
        execute(&service, &predecessor, 1, 7),
        SignalConditionalDecisionClass::ComputedChanged
    );
    let completion = service
        .deliver_committed_patch(
            &contract,
            SignalCommittedPatchDeliveryRequest::new([target(&contract)]),
        )
        .unwrap();
    let denial = match service.readmit_evaluation(ReadmissionRequest {
        predecessor: &predecessor,
        transitions: &[completion.successor_transition()],
    }) {
        Ok(_) => panic!("a second retained slot exceeds the installed ceiling"),
        Err(denial) => denial,
    };
    assert!(matches!(
        denial,
        ReadmissionDenial::AdmissionCapacityExhausted
    ));
    assert_eq!(
        execute(&service, &predecessor, 2, 99),
        SignalConditionalDecisionClass::DependencyUnchanged
    );
}

#[test]
fn panic_after_transition_activation_restores_custody_and_resumes_the_original_unwind() {
    let (_runtime, service, contract, source_owner) = fixture(None);
    let predecessor = service
        .admit_evaluation(
            &contract,
            ConditionalEvaluationSource::AdmittedRelationalSource(source_owner.admit("source")),
        )
        .unwrap();
    execute(&service, &predecessor, 1, 7);
    let completion = service
        .deliver_committed_patch(
            &contract,
            SignalCommittedPatchDeliveryRequest::new([target(&contract)]),
        )
        .unwrap();
    let ledger = service
        .owner
        .upgrade()
        .unwrap()
        .conditional_retention
        .clone();
    let before = ledger.usage();
    super::evaluation_reuse::arm_panic_after_transition_apply();

    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = service.readmit_evaluation(ReadmissionRequest {
            predecessor: &predecessor,
            transitions: &[completion.successor_transition()],
        });
    }));
    assert!(unwind.is_err(), "the injected panic must remain an unwind");
    assert_eq!(
        ledger.usage(),
        before,
        "failed readmission returns all custody"
    );
    assert_eq!(
        execute(&service, &predecessor, 2, 99),
        SignalConditionalDecisionClass::DependencyUnchanged,
        "the predecessor remains reusable and its mutex is not poisoned"
    );

    let successor = service
        .readmit_evaluation(ReadmissionRequest {
            predecessor: &predecessor,
            transitions: &[completion.successor_transition()],
        })
        .unwrap()
        .into_parts()
        .0;
    assert_eq!(
        execute(&service, &successor, 3, 8),
        SignalConditionalDecisionClass::ComputedChanged,
        "the canonical cell is restored and accepts the same transition"
    );
}

#[test]
fn concurrent_port_deliveries_publish_one_exact_serial_transition_chain() {
    let (_runtime, service, contract, source_owner) = fixture(None);
    let predecessor = service
        .admit_evaluation(
            &contract,
            ConditionalEvaluationSource::AdmittedRelationalSource(source_owner.admit("source")),
        )
        .unwrap();
    execute(&service, &predecessor, 1, 7);
    let service = std::sync::Arc::new(service);
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let mut completions = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..2)
            .map(|_| {
                let service = std::sync::Arc::clone(&service);
                let barrier = std::sync::Arc::clone(&barrier);
                let contract = contract.clone();
                scope.spawn(move || {
                    barrier.wait();
                    service
                        .deliver_committed_patch(
                            &contract,
                            SignalCommittedPatchDeliveryRequest::new([target(&contract)]),
                        )
                        .unwrap()
                })
            })
            .collect();
        barrier.wait();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    let first_order = service.readmit_evaluation(ReadmissionRequest {
        predecessor: &predecessor,
        transitions: &[
            completions[0].successor_transition(),
            completions[1].successor_transition(),
        ],
    });
    let successor = match first_order {
        Ok(successor) => successor,
        Err(ReadmissionDenial::TransitionChainMismatch) => {
            completions.swap(0, 1);
            service
                .readmit_evaluation(ReadmissionRequest {
                    predecessor: &predecessor,
                    transitions: &[
                        completions[0].successor_transition(),
                        completions[1].successor_transition(),
                    ],
                })
                .unwrap()
        }
        Err(denial) => panic!("unexpected concurrent delivery denial: {denial:?}"),
    };
    let (successor, counters) = successor.into_parts();
    assert_eq!(counters.transitions_checked(), 2);
    assert_eq!(
        execute(&service, &successor, 2, 8),
        SignalConditionalDecisionClass::ComputedChanged
    );
}
