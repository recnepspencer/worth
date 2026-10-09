use super::schema_registry_scenarios::conflict_isolation_merge_schema_registry;
use crate::facade::{
    ConflictIsolationGranularity, ConflictIsolationSelectionBasis, NodeEvaluationResult,
    SignalGraph, SignalRuntime,
};
use crate::tests::support::{version_ab, ASPECT_A, ASPECT_B};

#[test]
fn runtime_merge_lowers_conflict_isolation_records_for_conflicted_nodes() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph =
        SignalGraph::new().with_schema_registry(conflict_isolation_merge_schema_registry(None));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let shared = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.conflict-isolation-owned")
        .expect("known schema")
        .produces_aspects([ASPECT_A])
        .build();
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
        .create_branch("feature-conflict-isolation-schema-default")
        .unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(2, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(3, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .plan()
        .unwrap();

    assert_eq!(
        planned.plan().selected_conflict_isolation_name().as_str(),
        "signal.conflict-isolation.per-node"
    );
    assert_eq!(
        planned.plan().selected_conflict_isolation_basis(),
        ConflictIsolationSelectionBasis::BuiltInDefault
    );
    assert_eq!(
        planned.plan().conflict_isolation_plan().expansion_breadth,
        0
    );
}

#[test]
fn runtime_merge_request_named_conflict_isolation_precedes_schema_and_node_defaults() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(conflict_isolation_merge_schema_registry(
        Some("signal.conflict-isolation.per-node"),
    ));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let shared = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.conflict-isolation-owned")
        .expect("known schema")
        .conflict_isolation_policy_name("signal.conflict-isolation.per-node")
        .produces_aspects([ASPECT_A, ASPECT_B])
        .build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(401, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-request-conflict-isolation-precedence")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(402, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(403, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .conflict_isolation_policy_named("signal.conflict-isolation.per-aspect")
        .plan()
        .unwrap();

    assert_eq!(
        planned.plan().selected_conflict_isolation_name().as_str(),
        "signal.conflict-isolation.per-aspect"
    );
    assert_eq!(
        planned.plan().selected_conflict_isolation_basis(),
        ConflictIsolationSelectionBasis::RequestNamed
    );
    assert_eq!(planned.plan().conflict_isolation_plan().records.len(), 1);
    assert_eq!(
        planned.plan().conflict_isolation_plan().records[0].granularity,
        ConflictIsolationGranularity::PerAspect
    );
    assert_eq!(
        planned.plan().conflict_isolation_plan().records[0].isolated_aspects,
        vec![ASPECT_A, ASPECT_B]
    );
}

#[test]
fn runtime_merge_node_conflict_isolation_override_precedes_schema_default() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(conflict_isolation_merge_schema_registry(
        Some("signal.conflict-isolation.per-node"),
    ));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let shared = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.conflict-isolation-owned")
        .expect("known schema")
        .conflict_isolation_policy_name("signal.conflict-isolation.per-aspect")
        .produces_aspects([ASPECT_A, ASPECT_B])
        .build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(411, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-node-conflict-isolation-precedence")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(412, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(413, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .plan()
        .unwrap();

    assert_eq!(
        planned.plan().selected_conflict_isolation_name().as_str(),
        "signal.conflict-isolation.per-aspect"
    );
    assert_eq!(
        planned.plan().selected_conflict_isolation_basis(),
        ConflictIsolationSelectionBasis::NodeOverride
    );
    assert_eq!(
        planned.plan().conflict_isolation_plan().records[0].granularity,
        ConflictIsolationGranularity::PerAspect
    );
}

#[test]
fn runtime_merge_schema_default_conflict_isolation_applies_to_structural_conflicts() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(conflict_isolation_merge_schema_registry(
        Some("signal.conflict-isolation.per-aspect"),
    ));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let shared = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.conflict-isolation-owned")
        .expect("known schema")
        .produces_aspects([ASPECT_A, ASPECT_B])
        .build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(421, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-schema-conflict-isolation-precedence")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(422, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(423, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let planned = runtime
        .merge_raw()
        .from(feature.clone())
        .into(main.clone())
        .plan()
        .unwrap();

    assert_eq!(
        planned.plan().selected_conflict_isolation_name().as_str(),
        "signal.conflict-isolation.per-aspect"
    );
    assert_eq!(
        planned.plan().selected_conflict_isolation_basis(),
        ConflictIsolationSelectionBasis::SchemaDefault
    );
    assert_eq!(planned.plan().conflict_isolation_plan().records.len(), 1);
    assert_eq!(
        planned.plan().conflict_isolation_plan().records[0].isolated_aspects,
        vec![ASPECT_A, ASPECT_B]
    );
}
