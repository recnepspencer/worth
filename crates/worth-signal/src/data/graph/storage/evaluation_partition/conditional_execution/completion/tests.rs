use super::*;
use crate::data::aspect::{AspectMask, SignalAspectLoweringOwner};
use crate::data::comparator::DefaultComparatorPolicyResolver;
use crate::data::conditional_execution::{
    InstalledSignalConditionDecision, InstalledSignalConditionResolver,
    SignalConditionalArtifactReuse, SignalConditionalCondition,
    SignalConditionalContractDefinition, SignalConditionalExecutionRequest,
    SignalConditionalVersionComparator,
};
use crate::data::error::SignalError;
use crate::logic::transaction::SignalObservationRequest;

struct NoPredicate;
impl InstalledSignalConditionResolver for NoPredicate {
    fn resolve(
        &mut self,
        _: &crate::data::node::InstalledSignalConditionIdentity,
        _: &crate::logic::evaluation::ConditionEvaluationContext,
    ) -> Result<InstalledSignalConditionDecision, SignalError> {
        panic!("Always cannot call a predicate")
    }
}

#[test]
fn conditional_finalizer_retains_affinity_denial_when_foreign_session_cleanup_panics() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let owner = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&owner).unwrap();
    let worth_proof::TransitionOutcome::Success(capability) = graph.admit_installed_node(node)
    else {
        panic!("fresh node admits")
    };
    let contract = graph
        .install_conditional_contract(
            &owner,
            capability,
            SignalConditionalContractDefinition {
                condition: SignalConditionalCondition::Always,
                dependency_aspects: AspectMask::EMPTY,
                trigger_aspects: AspectMask::EMPTY,
                dependency_comparator: SignalConditionalVersionComparator::Exact,
                output_comparator: SignalConditionalVersionComparator::Exact,
                artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
            },
        )
        .unwrap();
    let attempt = graph.execute_installed_conditional_attempt(
        SignalConditionalExecutionRequest::new(&contract, "source", "failed", 1),
        &mut NoPredicate,
        &mut DefaultComparatorPolicyResolver::default(),
        || Err(SignalError::invalid_input("compute declined")),
        &mut crate::data::retained_storage::RetainedStoragePreparation::new(100_000),
    );
    let mut foreign = SignalGraph::new();
    let observation = foreign
        .begin_observation_session(SignalObservationRequest::operation())
        .unwrap();
    let bindings = foreign.invalidation_performed_work.shared_bindings();
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _held = bindings.lock().unwrap();
        panic!("injected foreign capture failure");
    }))
    .is_err());
    // Owner-finalizer fault injection: a genuine foreign session is rejected,
    // then its real cleanup panics. Correct partition activation never supplies
    // this foreign token; no token fields or admission authority are forged.
    let ActivatedConditionalOutcome::Unwound {
        payload,
        report: SignalPartitionConditionalUnwindReason::Cleanup { completion },
    } = finish_attempt(
        &graph,
        observation,
        attempt,
        &mut crate::data::retained_storage::RetainedStoragePreparation::new(100_000),
    )
    else {
        panic!("cleanup must be the first panic")
    };
    assert!(payload
        .downcast_ref::<String>()
        .unwrap()
        .contains("performed work observation poisoned"));
    let (decision, observation, _rejected) = completion.into_parts();
    let failure = match decision {
        Err(failure) => failure,
        Ok(_) => panic!("compute declined"),
    };
    assert_eq!(failure.counters().compute_contacts, 1);
    assert_eq!(
        failure.into_error(),
        SignalError::invalid_input("compute declined")
    );
    let Err(error) = observation else {
        panic!("foreign session must be denied")
    };
    assert_eq!(
        error,
        SignalError::invalid_input("observation session belongs to another runtime")
    );
    assert_eq!(foreign.observation_session_active_generation(), 0);
}
