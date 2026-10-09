use crate::facade::{
    AspectMergePolicyBinding, AspectMergePolicyName, ConflictPolicyName, MergeStrategyName,
    NodeContract, NodeEvaluationResult, SignalBranchHandle, SignalGraph, SignalRuntime,
};
use crate::schema::data::{
    SignalSchemaDescriptor, SignalSchemaId, SignalSchemaName, SignalSchemaRegistration,
    SignalSchemaRegistry, SignalSchemaVersion,
};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A, ASPECT_B};

pub(super) fn certification_schema_registry() -> SignalSchemaRegistry {
    SignalSchemaRegistry::from_registrations(vec![SignalSchemaRegistration::new(
        SignalSchemaDescriptor::new_with_merge_semantics_and_isolation(
            SignalSchemaId(91),
            SignalSchemaName::new("signal.demo.merge-certification-owned"),
            SignalSchemaVersion::new(1, 0),
            NodeContract::wildcard(),
            Some(MergeStrategyName::new(
                "signal.merge.rebase-source-onto-target",
            )),
            Some(ConflictPolicyName::new(
                "signal.conflict.resolve-source-when-structure-matches",
            )),
            None,
            None,
            None,
            None,
        ),
    )
    .expect("valid schema registration")])
    .expect("valid schema registry")
}

pub(super) fn certification_aspect_schema_registry() -> SignalSchemaRegistry {
    SignalSchemaRegistry::from_registrations(vec![SignalSchemaRegistration::new(
        SignalSchemaDescriptor::new_with_merge_semantics_and_aspects(
            SignalSchemaId(92),
            SignalSchemaName::new("signal.demo.merge-certification-aspect-owned"),
            SignalSchemaVersion::new(1, 0),
            NodeContract::wildcard(),
            Some(MergeStrategyName::new(
                "signal.merge.rebase-source-onto-target",
            )),
            None,
            None,
            None,
            None,
            vec![AspectMergePolicyBinding::new(
                ASPECT_A,
                AspectMergePolicyName::new("signal.aspect.prefer-source"),
            )],
        ),
    )
    .expect("valid schema registration")])
    .expect("valid schema registry")
}

pub(super) fn build_shared_state_conflict_runtime() -> (
    SignalRuntime<(), (), (), (), ()>,
    SignalBranchHandle,
    SignalBranchHandle,
) {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(certification_schema_registry());
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let shared = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.merge-certification-owned")
        .expect("known schema")
        .produces_aspects([ASPECT_A, ASPECT_B])
        .build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(501, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-merge-certification")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(502, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(503, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    (runtime, feature, main)
}

pub(super) fn build_aspect_policy_runtime() -> (
    SignalRuntime<(), (), (), (), ()>,
    SignalBranchHandle,
    SignalBranchHandle,
) {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(certification_aspect_schema_registry());
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let shared = runtime.graph_mut().node().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(511, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-merge-certification-aspect")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.merge-certification-aspect-owned")
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

    (runtime, feature, main)
}
