use super::schema_registry_scenarios::cross_identity_merge_schema_registry;
use crate::data::error::SignalError;
use crate::facade::{BranchMergeFailureKind, NodeEvaluationResult, SignalGraph, SignalRuntime};
use crate::tests::support::version_ab;

#[test]
fn runtime_merge_output_identity_matcher_rejects_ambiguous_target_journal_candidates() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(cross_identity_merge_schema_registry(None));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let mut runtime_ctx = ();
    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-identity-ambiguous-candidates")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.cross-identity-merge-owned")
        .expect("known schema")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(37, 0))
                        .with_output_identity("identity-ambiguous"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let target_a = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.cross-identity-merge-owned")
        .expect("known schema")
        .output_identity()
        .build();
    let target_b = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.cross-identity-merge-owned")
        .expect("known schema")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(target_a, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(38, 0))
                        .with_output_identity("identity-ambiguous"),
                ))
            })?;
            tx.read(target_b, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(39, 0))
                        .with_output_identity("identity-ambiguous"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let err = runtime
        .merge_raw()
        .from(feature)
        .into(main)
        .identity_matcher_named("signal.identity.output-identity-in-target-journal")
        .run()
        .expect_err("ambiguous output-identity candidates must fail explicitly");

    match err {
        SignalError::BranchMergeFailed { kind, message, .. } => {
            assert_eq!(kind, BranchMergeFailureKind::UnsupportedMergeStrategy);
            assert!(message.contains("ambiguous target journal correspondence"));
            assert!(message.contains(&feature_only.to_string()));
        }
        other => panic!("unexpected error kind: {other:?}"),
    }
}
