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

fn definition(dependency_aspects: AspectMask) -> SignalConditionalContractDefinition {
    SignalConditionalContractDefinition {
        condition: SignalConditionalCondition::Always,
        dependency_aspects,
        trigger_aspects: AspectMask::from_aspect(Aspect::new(1)),
        dependency_comparator: SignalConditionalVersionComparator::Exact,
        output_comparator: SignalConditionalVersionComparator::Exact,
        artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
    }
}

fn runtime_with_source_variants() -> (
    SignalRuntime<(), (), (), (), ()>,
    SignalAspectLoweringOwner,
    InstalledSignalConditionalContract,
    InstalledSignalConditionalContract,
    ConditionalSourceObservationOwner,
) {
    let mut graph = SignalGraph::new();
    let claimant = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&claimant).unwrap();
    let mut install = |dependency_aspects| {
        let node = graph.node().build();
        let worth_proof::TransitionOutcome::Success(capability) = graph.admit_installed_node(node)
        else {
            panic!("fresh node must admit")
        };
        graph
            .install_conditional_contract(&claimant, capability, definition(dependency_aspects))
            .unwrap()
    };
    let source_free = install(AspectMask::EMPTY);
    let source_present = install(AspectMask::from_aspect(Aspect::new(1)));
    drop(install);
    (
        SignalRuntime::build_for::<()>(graph),
        claimant,
        source_free,
        source_present,
        ConditionalSourceObservationOwner::fresh(),
    )
}

fn output(value: u64) -> NodeEvaluationResult {
    NodeEvaluationResult::from_version(AspectVersion::from_updates([(Aspect::new(0), value)]))
}

#[test]
fn typed_source_postures_deny_mismatch_and_keep_source_free_reuse_isolated() {
    let (mut runtime, claimant, source_free, source_present, source_owner) =
        runtime_with_source_variants();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    runtime.owner_port_slots().unwrap();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    let baseline = service._issuance_basis_custody.retention_usage();

    assert!(matches!(
        service.admit_evaluation(
            &source_present,
            ConditionalEvaluationSource::NoRelationalSource,
        ),
        Err(Denial::MissingSourceEvidence)
    ));
    assert_eq!(service._issuance_basis_custody.retention_usage(), baseline);

    assert!(matches!(
        service.admit_evaluation(
            &source_free,
            ConditionalEvaluationSource::AdmittedRelationalSource(
                source_owner.admit("unexpected-source"),
            ),
        ),
        Err(Denial::UnexpectedSourceEvidence)
    ));
    assert_eq!(service._issuance_basis_custody.retention_usage(), baseline);

    let first = service
        .admit_evaluation(
            &source_free,
            ConditionalEvaluationSource::NoRelationalSource,
        )
        .unwrap();
    let second = service
        .admit_evaluation(
            &source_free,
            ConditionalEvaluationSource::NoRelationalSource,
        )
        .unwrap();
    assert_ne!(first.source.projection(), second.source.projection());
    assert!(first.source.admitted_relational_source().is_none());
    drop(second);

    let mut computes = 0;
    for (attempt, expected_class) in [
        (1, SignalConditionalDecisionClass::ComputedChanged),
        (2, SignalConditionalDecisionClass::SuppressedBeforeCompute),
    ] {
        let completion = service
            .execute(
                &first,
                Request::new(attempt),
                &mut NoPredicate,
                &mut DefaultComparatorPolicyResolver::default(),
                || {
                    computes += 1;
                    Ok(output(5))
                },
            )
            .unwrap();
        assert!(completion
            .binding()
            .source()
            .admitted_relational_source()
            .is_none());
        let (decision, _) = completion.into_parts();
        assert_eq!(decision.unwrap().class(), expected_class);
    }
    assert_eq!(computes, 1);
}
