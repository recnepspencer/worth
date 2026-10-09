use super::schema_registry_scenarios::deletion_merge_schema_registry;
use crate::facade::{
    DeletionPolicySelectionBasis, NodeEvaluationResult, SignalGraph, SignalRuntime,
};
use crate::tests::support::{version_ab, ASPECT_A};

#[test]
fn runtime_merge_uses_schema_default_deletion_policy_when_request_is_silent() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(deletion_merge_schema_registry(Some(
        "signal.deletion.reject-target-only-conflict",
    )));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let shared = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(34, 0))
                        .with_output_identity("deletion-shared-base"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-schema-default-deletion-policy")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.deletion-merge-owned")
        .expect("known schema")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(34, 0))
                        .with_output_identity("deletion-shared-base"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let main_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(main_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(35, 0))
                        .with_output_identity("schema-default-deletion-policy"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let err = runtime
        .merge_raw()
        .from(feature)
        .into(main)
        .run()
        .expect_err("schema default reject-target-only-conflict should fail closed");

    assert!(err
        .to_string()
        .contains("signal.deletion.reject-target-only-conflict"));
}

#[test]
fn runtime_merge_node_deletion_policy_override_precedes_schema_default() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(deletion_merge_schema_registry(Some(
        "signal.deletion.reject-target-only-conflict",
    )));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let shared = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(36, 0))
                        .with_output_identity("deletion-override-shared"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-node-deletion-policy-override")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.deletion-merge-owned")
        .expect("known schema")
        .deletion_policy_name("signal.deletion.preserve-target-only")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(36, 0))
                        .with_output_identity("deletion-override-shared"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let main_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(main_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(37, 0))
                        .with_output_identity("node-deletion-policy-override"),
                ))
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
        planned.plan().selected_deletion_policy_name().as_str(),
        "signal.deletion.preserve-target-only"
    );
    assert_eq!(
        planned.plan().selected_deletion_policy_basis(),
        DeletionPolicySelectionBasis::NodeOverride
    );

    let result = planned.execute().unwrap();
    assert_eq!(
        result.selected_deletion_policy_name.as_str(),
        "signal.deletion.preserve-target-only"
    );
    assert_eq!(
        result.selected_deletion_policy_basis,
        DeletionPolicySelectionBasis::NodeOverride
    );
    assert_eq!(result.deletion_plan.target_only_count, 1);
}

#[test]
fn runtime_merge_request_named_deletion_policy_precedes_schema_and_node_defaults() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(deletion_merge_schema_registry(Some(
        "signal.deletion.reject-target-only-conflict",
    )));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let shared = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(136, 0))
                        .with_output_identity("deletion-request-shared"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-request-deletion-policy")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.deletion-merge-owned")
        .expect("known schema")
        .deletion_policy_name("signal.deletion.reject-target-only-conflict")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(136, 0))
                        .with_output_identity("deletion-request-shared"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let main_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(main_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(137, 0))
                        .with_output_identity("request-deletion-policy"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let result = runtime
        .merge_raw()
        .from(feature)
        .into(main)
        .deletion_policy_named("signal.deletion.preserve-target-only")
        .run()
        .expect("request deletion policy should override schema and node defaults");

    assert_eq!(
        result.selected_deletion_policy_name.as_str(),
        "signal.deletion.preserve-target-only"
    );
    assert_eq!(
        result.selected_deletion_policy_basis,
        DeletionPolicySelectionBasis::RequestNamed
    );
}
