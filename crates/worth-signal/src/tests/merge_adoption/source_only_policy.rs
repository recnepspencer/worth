use super::schema_registry_scenarios::source_only_merge_schema_registry;
use crate::facade::{
    ArtifactMergeAction, NodeEvaluationResult, SignalGraph, SignalRuntime,
    SourceOnlyPolicySelectionBasis,
};
use crate::tests::support::version_ab;

#[test]
fn runtime_merge_request_named_source_only_policy_selects_registered_descriptor() {
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
    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-request-named-source-only-policy")
        .unwrap();
    let mut runtime_ctx = ();

    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(30, 0))
                        .with_output_identity("request-named-source-only-policy"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();

    let planned = match runtime
        .merge_raw()
        .from(feature)
        .into(main)
        .source_only_policy_named("signal.source-only.reject-introduction")
        .plan()
    {
        Ok(_) => panic!(
            "reject-introduction policy must fail closed when source-only adoption is required"
        ),
        Err(err) => err,
    };

    assert!(planned
        .to_string()
        .contains("signal.source-only.reject-introduction"));
}

#[test]
fn runtime_merge_uses_schema_default_source_only_policy_when_request_is_silent() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(source_only_merge_schema_registry(Some(
        "signal.source-only.reject-introduction",
    )));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-schema-default-source-only-policy")
        .unwrap();
    let mut runtime_ctx = ();

    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.source-only-merge-owned")
        .expect("known schema")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(31, 0))
                        .with_output_identity("schema-default-source-only-policy"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();

    let err = match runtime.merge_raw().from(feature).into(main).plan() {
        Ok(_) => panic!("schema default reject-introduction should fail closed"),
        Err(err) => err,
    };

    assert!(err
        .to_string()
        .contains("signal.source-only.reject-introduction"));
}

#[test]
fn runtime_merge_node_source_only_policy_override_precedes_schema_default() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(source_only_merge_schema_registry(Some(
        "signal.source-only.reject-introduction",
    )));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-node-source-only-policy-override")
        .unwrap();
    let mut runtime_ctx = ();

    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.source-only-merge-owned")
        .expect("known schema")
        .source_only_policy_name("signal.source-only.introduce-adoptable-skip-non-adoptable")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(32, 0))
                        .with_output_identity("node-source-only-policy-override"),
                ))
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
        planned.plan().selected_source_only_policy_name().as_str(),
        "signal.source-only.introduce-adoptable-skip-non-adoptable"
    );
    assert_eq!(
        planned.plan().selected_source_only_policy_basis(),
        SourceOnlyPolicySelectionBasis::NodeOverride
    );

    let result = planned.execute().unwrap();
    assert_eq!(
        result.selected_source_only_policy_name.as_str(),
        "signal.source-only.introduce-adoptable-skip-non-adoptable"
    );
    assert_eq!(
        result.selected_source_only_policy_basis,
        SourceOnlyPolicySelectionBasis::NodeOverride
    );
    assert!(result.records.iter().any(|record| {
        record.source_node == feature_only
            && matches!(record.action, ArtifactMergeAction::IntroducedIntoTarget)
    }));
}

#[test]
fn runtime_merge_reject_source_only_policy_blocks_introduction_and_preserves_target_breadth() {
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
    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-reject-source-only-policy")
        .unwrap();
    let mut runtime_ctx = ();

    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(33, 0))
                        .with_output_identity("reject-source-only-policy"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let target_node_count_before = runtime.graph().active_node_count();
    let err = runtime
        .merge_raw()
        .from(feature)
        .into(main)
        .source_only_policy_named("signal.source-only.reject-introduction")
        .run()
        .expect_err("reject-introduction policy must block source-only adoption");

    assert!(err
        .to_string()
        .contains("rejects introducing source-only node"));
    assert_eq!(
        runtime.graph().active_node_count(),
        target_node_count_before
    );
}
