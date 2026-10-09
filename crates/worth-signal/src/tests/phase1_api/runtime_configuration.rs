use crate::facade::*;
use crate::tests::support::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Domain {
    Cache,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Impact {
    One,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ev {
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tier {
    Feature,
}

#[test]
fn runtime_builder_supports_typed_runtime_configuration() {
    let graph = SignalGraph::new();
    let _ = Impact::One;
    let _ = Ev::Tick;
    let _ = Tier::Feature;
    let runtime = SignalRuntime::builder(graph)
        .with_kernel_defaults()
        .with_domains::<Domain>()
        .with_impacts::<Impact>()
        .with_events::<Ev>()
        .with_tiers::<Tier>()
        .checkpoint_barrier(CheckpointBarrier::PerOperation)
        .fallback_comparator(VersionComparatorPolicy::Exact)
        .build();

    assert_eq!(
        runtime.checkpoint().policy().barrier_for(Domain::Cache),
        CheckpointBarrier::PerOperation
    );
}

#[test]
fn transaction_helper_commits_on_success() {
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
    let source = graph.node().build();
    let dependent = graph.node().build();
    graph
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();

    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let outcome = runtime
        .transaction(request_execution, &mut (), |transaction| {
            transaction.mark_dirty(source, ASPECT_A)?;
            Ok(())
        })
        .unwrap();

    assert_eq!(outcome.outcome, TransactionOutcome::Committed);
    assert_eq!(
        outcome.reconstructability.authority_branch_id,
        runtime.observe().current_branch().id
    );
    assert_eq!(
        outcome.reconstructability.authority_snapshot_id,
        runtime.observe().current_branch().head_snapshot_id
    );
    assert!(outcome.reconstructability.journal.replay_event_count >= 1);
    assert_eq!(
        outcome.reconstructability.checkpoint.journal_replay_span,
        outcome.reconstructability.journal.replay_event_count as u64
    );
    let metrics = runtime.observe().metrics();
    let graph_metrics = runtime.observe().graph().metrics();
    assert!(metrics.transaction.decision_log_event_count >= 1);
    assert!(graph_metrics.invalidation.batch_width >= 1);
    assert!(
        outcome
            .performance_accounting
            .transaction
            .decision_log_event_count
            >= 1
    );
    assert!(
        metrics.checkpoint.journal_replay_span
            >= outcome.reconstructability.journal.replay_event_count as u64
    );
    assert!(
        outcome
            .performance_accounting
            .checkpoint
            .journal_replay_span
            >= outcome.reconstructability.journal.replay_event_count as u64
    );
    assert_eq!(
        outcome.reconstructability.checkpoint.checkpoint_size,
        outcome.performance_accounting.checkpoint.checkpoint_size
    );
    let proof = outcome.reconstructability.proof();
    assert_eq!(
        proof.checkpoint.authority_branch_id,
        outcome.reconstructability.authority_branch_id
    );
    assert!(
        proof.required_rebuild.len() >= 2,
        "transaction proof should classify semantically required derived rebuild surfaces"
    );
    assert_eq!(
        runtime.graph().get_state(dependent).unwrap(),
        NodeState::Dirty
    );
}

#[test]
fn transaction_helper_rolls_back_on_error() {
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
    let source = graph.node().build();
    let dependent = graph.node().build();
    graph
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();
    let before = graph.get_state(dependent).unwrap();

    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let err = runtime
        .transaction(request_execution, &mut (), |transaction| {
            transaction.mark_dirty(source, ASPECT_A)?;
            Err(SignalError::internal("fail the transaction"))
        })
        .unwrap_err();

    assert!(format!("{err}").contains("fail the transaction"));
    assert_eq!(runtime.graph().get_state(dependent).unwrap(), before);
}

#[test]
fn graph_node_builder_sets_accessible_configuration() {
    let mut graph = SignalGraph::new();
    let node = graph
        .node()
        .reads_aspects([ASPECT_A, ASPECT_B])
        .produces_aspects([ASPECT_B])
        .requires_context(ContextRequirement::DomainContext)
        .path_class(PathClass::Rich)
        .maintenance_mode(MaintenanceMode::RebuildAllowed)
        .artifact_policy(ArtifactPolicyClass::DevelopmentRetained)
        .on_demand()
        .tolerance(2)
        .build();

    let config = graph.get_entry(node).unwrap().get_eval_config().clone();
    assert_eq!(
        config.contract.semantics.reads,
        AspectMask::from([ASPECT_A, ASPECT_B])
    );
    assert_eq!(
        config.contract.semantics.produces,
        AspectMask::from([ASPECT_B])
    );
    assert_eq!(
        config.contract.semantics.required_context,
        ContextRequirement::DomainContext
    );
    assert_eq!(
        config.contract.projection.consumes,
        AspectMask::from([ASPECT_A, ASPECT_B])
    );
    assert_eq!(config.contract.execution.path_class, PathClass::Rich);
    assert_eq!(
        config.contract.execution.maintenance_mode,
        MaintenanceMode::RebuildAllowed
    );
    assert_eq!(
        config.contract.execution.artifact_policy,
        ArtifactPolicyClass::DevelopmentRetained
    );
    assert_eq!(
        config.contract.execution.equivalence,
        EquivalenceContract::default()
    );
    assert_eq!(
        config.contract.authority.policy,
        AuthorityPolicy::SpeculativeThenReconcile
    );
    assert_eq!(config.condition, EvaluationCondition::OnDemand);
    assert_eq!(
        config.comparator,
        Some(VersionComparatorPolicy::Tolerance { epsilon: 2 })
    );
    assert_eq!(
        config.output_equivalence,
        crate::data::output_equivalence::OutputEquivalencePolicy::ExactAspectVersion
    );
}

#[test]
fn dependency_comparison_and_output_equivalence_have_separate_builder_authority() {
    let mut graph = SignalGraph::new();
    let dependency_only = graph
        .node()
        .dependency_comparator(VersionComparatorPolicy::Tolerance { epsilon: 3 })
        .build();
    let output_only = graph.node().output_identity().build();

    let dependency = graph.node_eval_config(dependency_only).unwrap();
    assert_eq!(
        dependency.comparator,
        Some(VersionComparatorPolicy::Tolerance { epsilon: 3 })
    );
    assert_eq!(
        dependency.output_equivalence,
        crate::data::output_equivalence::OutputEquivalencePolicy::ExactAspectVersion
    );

    let output = graph.node_eval_config(output_only).unwrap();
    assert_eq!(output.comparator, None);
    assert_eq!(
        output.output_equivalence,
        crate::data::output_equivalence::OutputEquivalencePolicy::OutputIdentity
    );
}

#[test]
fn legacy_output_identity_configuration_upgrades_to_the_canonical_split() {
    let mut graph = SignalGraph::new();
    let mut legacy = NodeEvaluationConfig::default();
    legacy.comparator = Some(VersionComparatorPolicy::OutputIdentity);
    legacy.contract.execution.equivalence =
        EquivalenceContract::for_comparator_override(&VersionComparatorPolicy::OutputIdentity);
    let mut encoded = serde_json::to_value(legacy).unwrap();
    encoded
        .as_object_mut()
        .unwrap()
        .remove("output_equivalence");
    let decoded: NodeEvaluationConfig = serde_json::from_value(encoded).unwrap();
    let node = graph.create_node_with_config(decoded);
    let restored = graph.node_eval_config(node).unwrap();

    assert_eq!(
        restored.comparator,
        Some(VersionComparatorPolicy::OutputIdentity)
    );
    assert_eq!(
        restored.output_equivalence,
        crate::data::output_equivalence::OutputEquivalencePolicy::OutputIdentity
    );
}

#[test]
fn node_contract_uses_explicit_performance_defaults() {
    let contract = NodeContract::default();

    assert_eq!(
        contract.execution.equivalence,
        EquivalenceContract::default()
    );
    assert_eq!(contract.execution.path_class, PathClass::Operational);
    assert_eq!(
        contract.execution.maintenance_mode,
        MaintenanceMode::DensityAdaptive
    );
    assert_eq!(
        contract.execution.artifact_policy,
        ArtifactPolicyClass::OperationalMinimal
    );
    assert_eq!(
        contract.authority.policy,
        AuthorityPolicy::SpeculativeThenReconcile
    );
    assert_eq!(
        contract.reuse,
        NodeReuseContract {
            equivalence: ArtifactEquivalenceContract::strict(),
            retain_certification: true,
        }
    );
    assert_eq!(contract.projection.consumes, AspectMask::ALL);
}

#[test]
fn graph_node_builder_sets_reuse_contract_accessibly() {
    let mut graph = SignalGraph::new();
    let node = graph
        .node()
        .artifact_equivalence_contract(ArtifactEquivalenceContract {
            required_boundaries: vec![
                ArtifactSemanticBoundary::TopologyRegime,
                ArtifactSemanticBoundary::AuthorityLane,
            ],
            supported_strategies: vec![
                crate::data::reuse::ReuseStrategy::OutputSuppression,
                crate::data::reuse::ReuseStrategy::MemoizedArtifactReuse,
                crate::data::reuse::ReuseStrategy::SnapshotRestoreReuse,
                crate::data::reuse::ReuseStrategy::ReconciliationAdoption,
                crate::data::reuse::ReuseStrategy::CrossIdentityPersistentMatch,
                crate::data::reuse::ReuseStrategy::PartialArtifactSplicing,
            ],
            allows_snapshot_restore_reuse: true,
            allows_authority_reconciliation_reuse: false,
        })
        .retain_reuse_certification(false)
        .build();

    let config = graph.get_entry(node).unwrap().get_eval_config().clone();
    assert_eq!(
        config.contract.reuse.equivalence.required_boundaries,
        vec![
            ArtifactSemanticBoundary::TopologyRegime,
            ArtifactSemanticBoundary::AuthorityLane,
        ]
    );
    assert!(
        config
            .contract
            .reuse
            .equivalence
            .allows_snapshot_restore_reuse
    );
    assert!(
        !config
            .contract
            .reuse
            .equivalence
            .allows_authority_reconciliation_reuse
    );
    assert!(!config.contract.reuse.retain_certification);
}

#[test]
fn reuse_domain_types_are_publicly_reachable() {
    let basis = ReuseBasis::strategy(
        crate::data::reuse::ReuseStrategy::MemoizedArtifactReuse,
        ReuseSource::MemoizedArtifact,
        ReuseCrossing::None,
    );
    let record = ReuseCertificationRecord {
        strategy: crate::data::reuse::ReuseStrategy::SnapshotRestoreReuse,
        origin: crate::data::reuse::ReuseOrigin::SnapshotRestore,
        source: ReuseSource::SnapshotArtifact,
        crossing: ReuseCrossing::SnapshotRestore,
        proofs: vec![ReuseBoundaryProof {
            boundary: ArtifactSemanticBoundary::SnapshotLineage,
            satisfied: true,
        }],
    };

    assert_eq!(
        basis,
        ReuseBasis::strategy(
            crate::data::reuse::ReuseStrategy::MemoizedArtifactReuse,
            ReuseSource::MemoizedArtifact,
            ReuseCrossing::None,
        )
    );
    assert_eq!(record.proofs.len(), 1);
    assert_eq!(
        record.proofs[0].boundary,
        ArtifactSemanticBoundary::SnapshotLineage
    );
}
