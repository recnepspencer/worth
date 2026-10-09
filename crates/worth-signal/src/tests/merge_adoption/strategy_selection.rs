use super::schema_registry_scenarios::merge_schema_registry;
use crate::facade::{
    merge_plan_proof_report, runtime_proof_report, BranchMergeStrategy,
    MergeStrategySelectionBasis, NodeEvaluationResult, SignalGraph, SignalRuntime,
};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn runtime_merge_strategy_hint_selects_registered_descriptor() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let shared = runtime.graph_mut().node().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-strategy-hint").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime.graph_mut().node().build();
    runtime
        .graph_mut()
        .append_dependency(feature_only, shared, ASPECT_A)
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                let upstream = view.read_aspect_version(shared, ASPECT_A)?;
                Ok(view.finish(NodeEvaluationResult::from_version(upstream)))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();

    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .strategy_hint(BranchMergeStrategy::ReplaySourceDeltaOntoTarget)
        .plan()
        .unwrap();

    assert_eq!(
        planned.plan().merge_strategy(),
        BranchMergeStrategy::ReplaySourceDeltaOntoTarget
    );
    assert_eq!(
        planned.plan().selected_strategy_name().as_str(),
        "signal.merge.replay-source-delta"
    );
}

#[test]
fn runtime_merge_strategy_named_selects_registered_descriptor() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let shared = runtime.graph_mut().node().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-strategy-name").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime.graph_mut().node().build();
    runtime
        .graph_mut()
        .append_dependency(feature_only, shared, ASPECT_A)
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                let upstream = view.read_aspect_version(shared, ASPECT_A)?;
                Ok(view.finish(NodeEvaluationResult::from_version(upstream)))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();

    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .strategy_named("signal.merge.rebase-source-onto-target")
        .plan()
        .unwrap();

    assert_eq!(
        planned.plan().merge_strategy(),
        BranchMergeStrategy::RebaseSourceOntoTarget
    );
    assert_eq!(
        planned.plan().selected_strategy_name().as_str(),
        "signal.merge.rebase-source-onto-target"
    );
}

#[test]
fn runtime_merge_base_named_selects_registered_descriptor_and_lowers_plan() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let shared = runtime.graph_mut().node().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-merge-base-name").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime.graph_mut().node().build();
    runtime
        .graph_mut()
        .append_dependency(feature_only, shared, ASPECT_A)
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                let upstream = view.read_aspect_version(shared, ASPECT_A)?;
                Ok(view.finish(NodeEvaluationResult::from_version(upstream)))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();

    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .merge_base_named("signal.merge-base.fork-point")
        .plan()
        .unwrap();

    let lowered = planned
        .plan()
        .lowered_merge_base()
        .expect("lowered merge-base plan");
    assert_eq!(
        lowered.selected_merge_base_name.as_str(),
        "signal.merge-base.fork-point"
    );
    assert_eq!(
        lowered.resolved_base.forked_from_snapshot_id,
        planned
            .plan()
            .merge_base()
            .and_then(|base| base.forked_from_snapshot_id)
    );
    let proof = merge_plan_proof_report(planned.plan(), planned.plan().registry_bundle_digest());
    assert_eq!(
        proof.selected_merge_base_digest,
        lowered.selected_merge_base_digest
    );
    assert_eq!(
        planned.plan().selected_semantics().merge_base_name.as_str(),
        "signal.merge-base.fork-point"
    );
}

#[test]
fn runtime_proof_report_carries_merge_base_registry_digest() {
    let runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();

    let report = runtime_proof_report(
        runtime.schema_registry().registry_digest(),
        runtime.merge_strategy_registry().registry_digest(),
        runtime.merge_base_strategy_registry().registry_digest(),
        runtime.aspect_merge_policy_registry().registry_digest(),
        runtime.conflict_isolation_registry().registry_digest(),
        runtime.conflict_policy_registry().registry_digest(),
        runtime.identity_matcher_registry().registry_digest(),
        runtime.source_only_policy_registry().registry_digest(),
        runtime.deletion_policy_registry().registry_digest(),
    );

    assert_eq!(
        report.merge_base_strategy_registry_digest,
        runtime.merge_base_strategy_registry().registry_digest()
    );
}

#[test]
fn runtime_merge_uses_schema_default_strategy_when_request_is_silent() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(merge_schema_registry(
        "signal.merge.rebase-source-onto-target",
        None,
        None,
    ));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let shared = runtime.graph_mut().node().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-schema-default-strategy")
        .unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.merge-owned")
        .expect("known schema")
        .build();
    runtime
        .graph_mut()
        .append_dependency(feature_only, shared, ASPECT_A)
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                let upstream = view.read_aspect_version(shared, ASPECT_A)?;
                Ok(view.finish(NodeEvaluationResult::from_version(upstream)))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();

    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .plan()
        .unwrap();

    assert_eq!(
        planned.plan().merge_strategy(),
        BranchMergeStrategy::RebaseSourceOntoTarget
    );
    assert_eq!(
        planned.plan().selected_strategy_basis(),
        MergeStrategySelectionBasis::SchemaDefault
    );
}

#[test]
fn runtime_merge_node_override_precedes_schema_default_strategy() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(merge_schema_registry(
        "signal.merge.rebase-source-onto-target",
        None,
        None,
    ));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let shared = runtime.graph_mut().node().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-node-override-strategy")
        .unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.merge-owned")
        .expect("known schema")
        .merge_strategy_name("signal.merge.replay-source-delta")
        .build();
    runtime
        .graph_mut()
        .append_dependency(feature_only, shared, ASPECT_A)
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                let upstream = view.read_aspect_version(shared, ASPECT_A)?;
                Ok(view.finish(NodeEvaluationResult::from_version(upstream)))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();

    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .plan()
        .unwrap();

    assert_eq!(
        planned.plan().merge_strategy(),
        BranchMergeStrategy::ReplaySourceDeltaOntoTarget
    );
    assert_eq!(
        planned.plan().selected_strategy_basis(),
        MergeStrategySelectionBasis::NodeOverride
    );
}
