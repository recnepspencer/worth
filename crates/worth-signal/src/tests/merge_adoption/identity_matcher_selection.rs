use super::schema_registry_scenarios::merge_schema_registry;
use crate::facade::{
    IdentityMatcherSelectionBasis, NodeEvaluationResult, SignalGraph, SignalRuntime,
};
use crate::tests::support::version_ab;

#[test]
fn runtime_merge_request_named_identity_matcher_selects_registered_descriptor() {
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
    let feature = runtime
        .create_branch("feature-request-named-identity-matcher")
        .unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("identity-matcher-request"),
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
        .identity_matcher_named("signal.identity.output-identity-in-target-journal")
        .plan()
        .unwrap();

    assert_eq!(
        planned.plan().selected_identity_matcher_name().as_str(),
        "signal.identity.output-identity-in-target-journal"
    );
    assert_eq!(
        planned.plan().selected_identity_matcher_basis(),
        IdentityMatcherSelectionBasis::RequestNamed
    );
    assert_eq!(
        planned
            .plan()
            .selected_semantics()
            .identity_matcher_name
            .as_str(),
        "signal.identity.output-identity-in-target-journal"
    );
}

#[test]
fn runtime_merge_uses_schema_default_identity_matcher_when_request_is_silent() {
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
        Some("signal.identity.output-identity-in-target-journal"),
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
        .create_branch("feature-schema-default-identity-matcher")
        .unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.merge-owned")
        .expect("known schema")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("identity-matcher-schema"),
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
        planned.plan().selected_identity_matcher_name().as_str(),
        "signal.identity.output-identity-in-target-journal"
    );
    assert_eq!(
        planned.plan().selected_identity_matcher_basis(),
        IdentityMatcherSelectionBasis::SchemaDefault
    );
}

#[test]
fn runtime_merge_node_identity_matcher_override_precedes_schema_default() {
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
        Some("signal.identity.exact-node-id"),
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
        .create_branch("feature-node-override-identity-matcher")
        .unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.merge-owned")
        .expect("known schema")
        .identity_matcher_name("signal.identity.output-identity-in-target-journal")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("identity-matcher-node"),
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
        planned.plan().selected_identity_matcher_name().as_str(),
        "signal.identity.output-identity-in-target-journal"
    );
    assert_eq!(
        planned.plan().selected_identity_matcher_basis(),
        IdentityMatcherSelectionBasis::NodeOverride
    );
}
