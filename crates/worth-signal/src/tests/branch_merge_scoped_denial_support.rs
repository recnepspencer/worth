use crate::facade::*;
use crate::schema::data::{
    SignalSchemaDescriptor, SignalSchemaId, SignalSchemaName, SignalSchemaRegistration,
    SignalSchemaRegistry, SignalSchemaVersion,
};
use crate::tests::support::{version_ab, ASPECT_A};

pub(crate) fn build_scoped_denial_runtime() -> (
    SignalRuntime<(), (), (), (), ()>,
    SignalBranchHandle,
    SignalBranchHandle,
    NodeId,
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

    let graph = SignalGraph::new();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let primary = runtime
        .graph_mut()
        .node()
        .reads_aspects([ASPECT_A])
        .produces_aspects([ASPECT_A])
        .build();
    let mut runtime_ctx = ();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(primary, &|view| {
                let _ = view;
                Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
                    .with_output_identity("phase6-primary"))
            })?;
            Ok(())
        })
        .unwrap();
    let main = runtime.current_branch();
    let feature = runtime.create_branch("feature-scoped-denial").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(primary, ASPECT_A)?;
            tx.read(primary, &|view| {
                let _ = view;
                Ok(NodeEvaluationResult::from_version(version_ab(101, 0))
                    .with_output_identity("phase6-primary-feature"))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();
    (runtime, feature, main, primary)
}

pub(crate) fn build_ambiguous_scoped_denial_runtime() -> (
    SignalRuntime<(), (), (), (), ()>,
    SignalBranchHandle,
    SignalBranchHandle,
    NodeId,
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

    let graph = SignalGraph::new().with_schema_registry(cross_identity_merge_schema_registry());
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let mut runtime_ctx = ();
    let main = runtime.current_branch();
    let feature = runtime.create_branch("feature-scoped-ambiguous").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let source = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.cross-identity-merge-owned")
        .expect("known schema")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            tx.read(source, &|view| {
                let _ = view;
                Ok(NodeEvaluationResult::from_version(version_ab(3, 0))
                    .with_output_identity("ambiguous"))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();
    let target_left = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.cross-identity-merge-owned")
        .expect("known schema")
        .output_identity()
        .build();
    let target_right = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.cross-identity-merge-owned")
        .expect("known schema")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(target_left, &|view| {
                let _ = view;
                Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
                    .with_output_identity("ambiguous"))
            })?;
            tx.read(target_right, &|view| {
                let _ = view;
                Ok(NodeEvaluationResult::from_version(version_ab(2, 0))
                    .with_output_identity("ambiguous"))
            })?;
            Ok(())
        })
        .unwrap();
    (runtime, feature, main, source)
}

pub(crate) fn selected_node_scope_digest(
    feature: &SignalBranchHandle,
    main: &SignalBranchHandle,
    node: NodeId,
) -> String {
    BranchMergeRequest::selected_nodes(feature.clone(), main.clone(), [node])
        .normalize()
        .expect("selected-node request should normalize")
        .normalized_scope()
        .scope_digest()
        .to_owned()
}

pub(crate) fn selected_aspect_scope_digest(
    feature: &SignalBranchHandle,
    main: &SignalBranchHandle,
    entry: SignalSelectedAspectRequestEntry,
) -> String {
    BranchMergeRequest::selected_aspects(feature.clone(), main.clone(), [entry])
        .normalize()
        .expect("selected-aspect request should normalize")
        .normalized_scope()
        .scope_digest()
        .to_owned()
}

pub(crate) fn assert_scoped_denial_is_side_effect_free(
    runtime: &mut SignalRuntime<(), (), (), (), ()>,
    branch: &SignalBranchHandle,
    expected_branch_digest: &str,
) {
    assert_eq!(
        runtime
            .current_branch_basis_artifact()
            .payload()
            .basis_digest(),
        expected_branch_digest,
        "typed scoped denial must not mutate current branch state"
    );
    assert_eq!(
        runtime.current_branch(),
        branch.clone(),
        "typed scoped denial must not switch the active branch"
    );
    assert_eq!(
        runtime.telemetry().transaction.delivered_observation_count,
        0,
        "typed scoped denial must not deliver observations"
    );
}

fn cross_identity_merge_schema_registry() -> SignalSchemaRegistry {
    SignalSchemaRegistry::from_registrations(vec![SignalSchemaRegistration::new(
        SignalSchemaDescriptor::new_with_merge_semantics(
            SignalSchemaId(42),
            SignalSchemaName::new("signal.demo.cross-identity-merge-owned"),
            SignalSchemaVersion::new(1, 0),
            NodeContract::wildcard().with_cross_identity_persistent_matching(),
            Some(MergeStrategyName::new(
                "signal.merge.rebase-source-onto-target",
            )),
            None,
            Some(IdentityMatcherName::new(
                "signal.identity.output-identity-in-target-journal",
            )),
            None,
            None,
        ),
    )
    .expect("valid cross-identity registration")])
    .expect("valid cross-identity schema registry")
}
