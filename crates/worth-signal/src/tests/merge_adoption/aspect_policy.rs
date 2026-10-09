use super::schema_registry_scenarios::aspect_policy_merge_schema_registry;
use crate::facade::{
    AspectMergeDecisionOutcome, AspectMergePolicySelectionBasis, NodeEvaluationResult, SignalGraph,
    SignalRuntime,
};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn runtime_merge_lowers_schema_default_aspect_policy_for_affected_aspect() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(aspect_policy_merge_schema_registry(Some(
        "signal.aspect.prefer-source",
    )));
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
        .create_branch("feature-aspect-schema-default")
        .unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.aspect-merge-owned")
        .expect("known schema")
        .produces_aspects([ASPECT_A])
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

    let aspect_plan = planned.plan().aspect_policy_plan();
    assert_eq!(aspect_plan.records.len(), 1);
    assert_eq!(aspect_plan.records[0].aspect, ASPECT_A);
    assert_eq!(
        aspect_plan.records[0].selected_policy_name.as_str(),
        "signal.aspect.prefer-source"
    );
    assert_eq!(
        aspect_plan.records[0].selected_policy_basis,
        AspectMergePolicySelectionBasis::SchemaDefault
    );
}

#[test]
fn runtime_merge_node_aspect_policy_override_precedes_schema_default() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(aspect_policy_merge_schema_registry(Some(
        "signal.aspect.prefer-target",
    )));
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
        .create_branch("feature-aspect-node-override")
        .unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.aspect-merge-owned")
        .expect("known schema")
        .produces_aspects([ASPECT_A])
        .aspect_merge_policy_name(ASPECT_A, "signal.aspect.prefer-source")
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

    let aspect_plan = planned.plan().aspect_policy_plan();
    assert_eq!(aspect_plan.records.len(), 1);
    assert_eq!(
        aspect_plan.records[0].selected_policy_name.as_str(),
        "signal.aspect.prefer-source"
    );
    assert_eq!(
        aspect_plan.records[0].selected_policy_basis,
        AspectMergePolicySelectionBasis::NodeOverride
    );
}

#[test]
fn runtime_merge_request_named_aspect_policy_precedes_schema_and_node_defaults() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(aspect_policy_merge_schema_registry(Some(
        "signal.aspect.prefer-target",
    )));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let shared = runtime.graph_mut().node().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(4, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-aspect-request-override")
        .unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.aspect-merge-owned")
        .expect("known schema")
        .produces_aspects([ASPECT_A])
        .aspect_merge_policy_name(ASPECT_A, "signal.aspect.prefer-target")
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
        .aspect_policy_named(ASPECT_A, "signal.aspect.prefer-source")
        .plan()
        .unwrap();

    let aspect_plan = planned.plan().aspect_policy_plan();
    assert_eq!(aspect_plan.records.len(), 1);
    assert_eq!(
        aspect_plan.records[0].selected_policy_name.as_str(),
        "signal.aspect.prefer-source"
    );
    assert_eq!(
        aspect_plan.records[0].selected_policy_basis,
        AspectMergePolicySelectionBasis::RequestNamed
    );
}

#[test]
fn runtime_merge_lowers_aspect_decision_records_for_affected_nodes() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(aspect_policy_merge_schema_registry(Some(
        "signal.aspect.prefer-source",
    )));
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
        .create_branch("feature-aspect-decision-lowering")
        .unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.aspect-merge-owned")
        .expect("known schema")
        .produces_aspects([ASPECT_A])
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

    let aspect_decisions = &planned.plan().aspect_decision_plan().records;
    assert_eq!(aspect_decisions.len(), 1);
    assert_eq!(aspect_decisions[0].aspect, ASPECT_A);
    assert_eq!(aspect_decisions[0].source_node, feature_only);
    assert_eq!(
        aspect_decisions[0].selected_policy_name.as_str(),
        "signal.aspect.prefer-source"
    );
    assert_eq!(
        aspect_decisions[0].outcome,
        AspectMergeDecisionOutcome::SourceIntroducedIntoTarget
    );
}
