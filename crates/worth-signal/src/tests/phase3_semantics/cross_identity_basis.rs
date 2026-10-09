use crate::facade::{
    ChangedRegion, EvaluationContext, NodeContract, NodeEvaluationResult, OutputChange,
    PartitionSubscription, Recipe, SignalGraph, SignalRuntime, SignalRuntimePolicy,
    VersionComparatorPolicy,
};
use crate::tests::support::{version_ab, ASPECT_A, ASPECT_B};

#[test]
fn cross_identity_contract_declared_basis_is_retained_in_runtime_truth() {
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
                        .with_output_identity("contract-basis-artifact")
                        .with_output_change(OutputChange::Refreshed),
                ))
            },
        })
        .unwrap();
    let source = projection.keyed("source");
    let alias = projection.keyed("alias-contract");
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

    let trace_summary = runtime
        .graph()
        .observe()
        .materialize()
        .materialize_trace_summary(alias_node)
        .unwrap()
        .expect("trace summary");
    assert_eq!(
        trace_summary.reuse_origin,
        crate::data::reuse::ReuseOrigin::CrossIdentityPersistentReuse
    );
    assert_eq!(
        trace_summary
            .reuse_boundary_context
            .as_ref()
            .and_then(|ctx| ctx.persistent_correspondence()),
        Some(
            &crate::data::reuse::PersistentCorrespondenceEvidence::ContractDeclaredBasis(
                "contract:mesh-family:v2".to_string()
            )
        )
    );
}

#[test]
fn cross_identity_lineage_mapping_is_retained_in_runtime_truth() {
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
                        .with_output_identity("lineage-map-artifact")
                        .with_output_change(OutputChange::Refreshed),
                ))
            },
        })
        .unwrap();
    let source = projection.keyed("source");
    let alias = projection.keyed("alias-lineage");
    let alias_node = alias.node(&mut runtime);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            source.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            alias.evaluate_cross_identity_with_lineage_mapping(
                tx,
                "source",
                "shape-v1",
                "lineage-map:mesh-42->mesh-77",
            )
        })
        .unwrap();

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
            &crate::data::reuse::PersistentCorrespondenceEvidence::LineageBackedMapping(
                "lineage-map:mesh-42->mesh-77".to_string()
            )
        )
    );
}

#[test]
fn cross_identity_region_identity_basis_is_retained_in_runtime_truth() {
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
                .with_cross_identity_persistent_matching()
                .with_partition_scope(PartitionSubscription::whole_partition("wing")),
            tier: (),
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |view: &mut EvaluationContext<'_, ()>| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("region-basis-artifact")
                        .with_output_change(OutputChange::Refreshed)
                        .with_changed_region(ChangedRegion::new("wing")),
                ))
            },
        })
        .unwrap();
    let source = projection.keyed("source");
    let alias = projection.keyed("alias-region");
    let alias_node = alias.node(&mut runtime);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            source.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            alias.evaluate_cross_identity_with_region_identity(
                tx,
                "source",
                "shape-v1",
                "region:wing",
            )
        })
        .unwrap();

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
            &crate::data::reuse::PersistentCorrespondenceEvidence::RegionIdentityBasis(
                "region:wing".to_string()
            )
        )
    );
}
