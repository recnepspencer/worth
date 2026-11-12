use worth_execution::{CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{
    AspectValue, DurableSignalBranchIdentity, DurableSignalOwnerIdentity, HomeIdentity,
    SignalOwnerSlotIdentity,
};
use worth_signal::facade::*;

fn branch() -> DurableSignalBranchIdentity {
    DurableSignalBranchIdentity::from_recorded(
        DurableSignalOwnerIdentity::from_recorded(
            HomeIdentity::from_observed_bytes([7; 16]).unwrap(),
            SignalOwnerSlotIdentity::from_recorded(1),
        ),
        0,
    )
}

#[test]
fn conditional_source_reinstalls_with_fresh_bindings_after_graph_reconstitution() {
    let funding = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(16 << 20)),
        CancellationToken::new(),
        None,
    );
    let execution = ExecutionRequest::serial(&funding);
    let mut graph = SignalGraph::new();
    let owner = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&owner).unwrap();
    let source = graph.create_node();
    let node = graph.create_node();
    graph
        .set_dependencies(node, [DependencyEdge::new(source, Aspect::new(1))])
        .unwrap();
    let definition = SignalConditionalContractDefinition {
        condition: SignalConditionalCondition::DeltaThreshold(SignalDeltaThresholdContract::new(
            AspectValue::UInt64(37),
            "authored-units",
            SignalThresholdValueFamily::Integer,
            SignalThresholdComparisonDomain::AbsoluteDifference,
            SignalThresholdBoundary::Inclusive,
        )),
        dependency_aspects: AspectMask::from_bits(7),
        trigger_aspects: AspectMask::from_bits(3),
        dependency_comparator: SignalConditionalVersionComparator::Tolerance(5),
        output_comparator: SignalConditionalVersionComparator::Exact,
        artifact_reuse: SignalConditionalArtifactReuse::RuntimeResolved,
    };
    let worth_proof::TransitionOutcome::Success(capability) = graph.admit_installed_node(node)
    else {
        panic!("fresh authored node must admit");
    };
    let original = graph
        .install_conditional_contract(&owner, capability, definition.clone())
        .unwrap();
    let image = graph.capture_authored_image(&execution, branch()).unwrap();
    let rebound = graph
        .reconstitute_for_runtime_rebind()
        .unwrap()
        .into_parts()
        .0;
    for projected in [graph, rebound] {
        let mut fresh = Box::new(SignalGraph::new());
        let fresh_owner = SignalAspectLoweringOwner::fresh();
        fresh.claim_aspect_lowering_owner(&fresh_owner).unwrap();
        let source_image = projected
            .capture_authored_image(&execution, branch())
            .unwrap();
        assert_eq!(source_image.as_bytes(), image.as_bytes());
        let (recovered, contracts) = fresh
            .readmit_authored_image(&execution, &source_image)
            .unwrap()
            .into_parts();
        assert_eq!(contracts.len(), 1);
        assert_eq!(contracts[0].semantic_condition(), &definition.condition);
        assert_eq!(contracts[0].authored_definition(), &definition);
        assert_ne!(
            contracts[0].graph_instance_id(),
            original.graph_instance_id()
        );
        assert_eq!(contracts[0].generation(), 1);
        assert_eq!(
            recovered.declared_dependencies_of(node).unwrap()[0].source(),
            source
        );
        assert_eq!(
            recovered
                .capture_authored_image(&execution, branch())
                .unwrap()
                .as_bytes(),
            image.as_bytes()
        );
    }
}

#[test]
fn authored_readmission_keeps_values_cold_until_first_demand() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let funding = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        CancellationToken::new(),
        None,
    );
    let execution = ExecutionRequest::serial(&funding);
    let mut graph = SignalGraph::new();
    let node = graph.node().on_demand().build();
    let calls = AtomicUsize::new(0);
    let evaluate = |_: &mut EvaluationContext<'_, ()>| -> Result<AspectVersion, SignalError> {
        calls.fetch_add(1, Ordering::Relaxed);
        Ok(AspectVersion::from_updates([(Aspect::new(0), 71)]))
    };
    let mut memory = SignalRuntime::build_for::<()>(graph);
    let expected = memory.read(node, &(), &evaluate).unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    let image = memory
        .graph()
        .capture_authored_image(&execution, branch())
        .unwrap();
    let (graph, contracts) = Box::new(SignalGraph::new())
        .readmit_authored_image(&execution, &image)
        .unwrap()
        .into_parts();
    assert!(contracts.is_empty());
    let mut recovered = SignalRuntime::build_for::<()>(*graph);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert_eq!(recovered.read(node, &(), &evaluate).unwrap(), expected);
    assert_eq!(calls.load(Ordering::Relaxed), 2);
    assert_eq!(recovered.read(node, &(), &evaluate).unwrap(), expected);
    assert_eq!(calls.load(Ordering::Relaxed), 2);
}

fn graph_with_runtime_comparator() -> (Box<SignalGraph>, InstalledSignalConditionalContract) {
    let mut graph = Box::new(SignalGraph::new());
    let owner = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&owner).unwrap();
    let node = graph.create_node();
    let worth_proof::TransitionOutcome::Success(capability) = graph.admit_installed_node(node)
    else {
        panic!("fresh node must admit");
    };
    let installed = graph
        .install_conditional_contract(
            &owner,
            capability,
            SignalConditionalContractDefinition {
                condition: SignalConditionalCondition::OnDemand,
                dependency_aspects: AspectMask::ALL,
                trigger_aspects: AspectMask::ALL,
                dependency_comparator: SignalConditionalVersionComparator::RuntimeResolved,
                output_comparator: SignalConditionalVersionComparator::Exact,
                artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
            },
        )
        .unwrap();
    (graph, installed)
}

#[test]
fn runtime_comparator_readmission_requires_its_full_definition_and_rebinds_current_installation() {
    use worth_signal::facade::durability::{
        SignalAuthoredImageDenial, SignalDurableTierVocabulary,
    };
    use worth_signal::facade::runtime::RuntimeConfig;
    let funding = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        CancellationToken::new(),
        None,
    );
    let execution = ExecutionRequest::serial(&funding);
    let vocabulary = SignalDurableTierVocabulary::declare([((), "base")]).unwrap();
    let (graph, installed) = graph_with_runtime_comparator();
    let (_, foreign) = graph_with_runtime_comparator();
    let mut config = RuntimeConfig::new();
    config.set_fallback_comparator(foreign.dependency_comparator().clone());
    assert!(matches!(
        config.capture_authored_image(&graph, &vocabulary, branch(), &execution),
        Err(SignalAuthoredImageDenial::ForeignConditionalBinding)
    ));
    config.set_fallback_comparator(installed.dependency_comparator().clone());
    let runtime_image = config
        .capture_authored_image(&graph, &vocabulary, branch(), &execution)
        .unwrap();
    let graph_image = graph.capture_authored_image(&execution, branch()).unwrap();
    let (fresh, _) = graph_with_runtime_comparator();
    let (recovered, contracts) = fresh
        .readmit_authored_image(&execution, &graph_image)
        .unwrap()
        .into_parts();
    let recovered_config =
        RuntimeConfig::readmit_authored_image(&execution, &runtime_image, &recovered, &vocabulary)
            .unwrap();
    assert_eq!(
        recovered_config.fallback_comparator(),
        contracts[0].dependency_comparator()
    );
    assert_ne!(
        recovered_config.fallback_comparator(),
        config.fallback_comparator()
    );
    let mut missing = SignalGraph::new();
    assert_eq!(missing.create_node(), installed.node());
    assert!(matches!(
        RuntimeConfig::readmit_authored_image(&execution, &runtime_image, &missing, &vocabulary),
        Err(SignalAuthoredImageDenial::ConditionalDefinitionMissing)
    ));
}
