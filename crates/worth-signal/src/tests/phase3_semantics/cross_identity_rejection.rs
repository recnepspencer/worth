use crate::facade::{
    mark_dirty, EvaluationContext, NodeContract, NodeEvaluationResult, OutputChange, Recipe,
    SignalGraph, SignalRuntime, SignalRuntimePolicy, VersionComparatorPolicy,
};
use crate::tests::support::{version_ab, ASPECT_A, ASPECT_B};
use std::sync::atomic::{AtomicU32, Ordering};

#[test]
fn cross_identity_changed_contract_basis_is_rejected_and_preserves_previous_correspondence() {
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
    runtime
        .graph_mut()
        .set_runtime_policy(SignalRuntimePolicy::development());
    let compute_calls = AtomicU32::new(0);
    let projection = runtime
        .define(Recipe {
            family: "projection".into(),
            contract: NodeContract::reads([ASPECT_A])
                .with_produces([ASPECT_B])
                .with_cross_identity_persistent_matching(),
            tier: (),
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |view: &mut EvaluationContext<'_, ()>| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("contract-mismatch-artifact")
                        .with_output_change(OutputChange::Refreshed),
                ))
            },
        })
        .unwrap();
    let source = projection.keyed("source");
    let alias = projection.keyed("alias-contract-mismatch");
    let alias_node = alias.node(&mut runtime);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            source.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            alias.evaluate_cross_identity_with_contract_basis(
                tx,
                "source",
                "shape-v1",
                "contract:mesh-family:v2",
            )
        })
        .unwrap();

    mark_dirty(runtime.graph_mut(), alias_node, ASPECT_A).unwrap();

    let err = runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            alias.evaluate_cross_identity_with_contract_basis(
                tx,
                "source",
                "shape-v1",
                "contract:mesh-family:v3",
            )
        })
        .expect_err("changed correspondence basis should be rejected");
    assert!(err.to_string().contains("reuse certification failed"));
    assert_eq!(
        compute_calls.load(Ordering::Relaxed),
        1,
        "rejected cross-identity mismatch should fail before fallback recompute"
    );

    let trace_summary = runtime
        .graph()
        .observe()
        .materialize()
        .materialize_trace_summary(alias_node)
        .unwrap()
        .expect("trace summary");
    assert_eq!(
        trace_summary
            .reuse_boundary_context
            .as_ref()
            .and_then(|ctx| ctx.persistent_correspondence()),
        Some(
            &crate::data::reuse::PersistentCorrespondenceEvidence::ContractDeclaredBasis(
                "contract:mesh-family:v2".to_string()
            )
        ),
        "failed reuse admission must not overwrite prior certified correspondence"
    );
    assert_eq!(
        runtime
            .observe()
            .metrics()
            .evaluation
            .reuse_rejected_persistent_correspondence_invalid_count,
        1
    );
}

#[test]
fn cross_identity_evidence_family_change_is_rejected_and_not_treated_as_equivalent() {
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
    runtime
        .graph_mut()
        .set_runtime_policy(SignalRuntimePolicy::development());
    let compute_calls = AtomicU32::new(0);
    let projection = runtime
        .define(Recipe {
            family: "projection".into(),
            contract: NodeContract::reads([ASPECT_A])
                .with_produces([ASPECT_B])
                .with_cross_identity_persistent_matching(),
            tier: (),
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |view: &mut EvaluationContext<'_, ()>| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("family-mismatch-artifact")
                        .with_output_change(OutputChange::Refreshed),
                ))
            },
        })
        .unwrap();
    let source = projection.keyed("source");
    let alias = projection.keyed("alias-family-mismatch");
    let alias_node = alias.node(&mut runtime);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            source.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            alias.evaluate_cross_identity(tx, "source", "shape-v1", "mesh-001")
        })
        .unwrap();

    mark_dirty(runtime.graph_mut(), alias_node, ASPECT_A).unwrap();

    let err = runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            alias.evaluate_cross_identity_with_lineage_mapping(tx, "source", "shape-v1", "mesh-001")
        })
        .expect_err("changing correspondence evidence family should be rejected");
    assert!(err.to_string().contains("reuse certification failed"));
    assert_eq!(
        compute_calls.load(Ordering::Relaxed),
        1,
        "family mismatch should not silently degrade into recompute"
    );

    let trace_summary = runtime
        .graph()
        .observe()
        .materialize()
        .materialize_trace_summary(alias_node)
        .unwrap()
        .expect("trace summary");
    assert_eq!(
        trace_summary
            .reuse_boundary_context
            .as_ref()
            .and_then(|ctx| ctx.persistent_correspondence()),
        Some(
            &crate::data::reuse::PersistentCorrespondenceEvidence::HostSuppliedKey(
                "mesh-001".to_string()
            )
        ),
        "distinct correspondence families must remain semantically distinct after rejection"
    );
    assert_eq!(
        runtime
            .observe()
            .metrics()
            .evaluation
            .reuse_rejected_persistent_correspondence_invalid_count,
        1
    );
}

#[test]
fn ambiguous_lineage_mapping_is_rejected_before_cross_identity_reuse_commits() {
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
    runtime
        .graph_mut()
        .set_runtime_policy(SignalRuntimePolicy::development());
    let projection = runtime
        .define(Recipe {
            family: "projection".into(),
            contract: NodeContract::reads([ASPECT_A])
                .with_produces([ASPECT_B])
                .with_cross_identity_persistent_matching(),
            tier: (),
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |view: &mut EvaluationContext<'_, ()>| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("ambiguous-lineage-artifact")
                        .with_output_change(OutputChange::Refreshed),
                ))
            },
        })
        .unwrap();
    let source = projection.keyed("source");
    let alias = projection.keyed("alias-ambiguous");
    let alias_node = alias.node(&mut runtime);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            source.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();

    let err = runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            alias.evaluate_cross_identity_with_lineage_mapping(
                tx,
                "source",
                "shape-v1",
                "lineage-map:mesh-42->mesh-77|mesh-99",
            )
        })
        .expect_err("ambiguous lineage mapping should be rejected");
    assert!(err.to_string().contains("reuse certification failed"));
    assert_eq!(
        runtime
            .observe()
            .metrics()
            .evaluation
            .reuse_rejected_persistent_correspondence_invalid_count,
        1
    );
    assert!(
        runtime
            .graph()
            .observe()
            .runtime_artifact_state(alias_node)
            .unwrap()
            .is_none(),
        "failed first admission must not commit runtime artifact state"
    );
}
