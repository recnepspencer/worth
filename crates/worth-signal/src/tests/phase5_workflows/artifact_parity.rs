use crate::facade::{
    lineage_records_equivalent, replay_slices_equivalent, LineageRecord, NodeEvaluationResult,
    NodeExplanation, ReplaySlice, SignalGraph, SignalRuntime, SignalRuntimePolicy,
};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn retained_vs_reconstructed_artifacts_match_after_long_churn() {
    fn run(policy: SignalRuntimePolicy) -> (ReplaySlice, Vec<LineageRecord>, NodeExplanation) {
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
        runtime.set_runtime_policy(policy.with_observation_activation(
            worth_foundational::ObservationActivationProfile::Continuous,
        ));
        let source = runtime.graph_mut().node().output_identity().build();
        let dependent = runtime.graph_mut().node().output_identity().build();
        runtime
            .graph_mut()
            .append_dependency(dependent, source, ASPECT_A)
            .unwrap();
        let mut runtime_ctx = ();

        runtime
            .transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.read(source, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(1, 0))
                            .with_output_identity("seed"),
                    ))
                })?;
                tx.read(dependent, &|view| {
                    let version = view.read_aspect_version(source, ASPECT_A)?;
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version)
                            .with_output_identity("dependent-seed"),
                    ))
                })?;
                Ok(())
            })
            .unwrap();

        let main = runtime.observe().current_branch();
        let snapshot = runtime
            .capture_snapshot()
            .expect("snapshot capture should succeed without managed queue bindings");
        let feature = runtime.create_branch("feature-retention").unwrap();
        runtime.switch_branch(feature.clone()).unwrap();

        for step in 0..12 {
            runtime
                .transaction(request_execution, &mut runtime_ctx, |tx| {
                    tx.mark_dirty(source, ASPECT_A)?;
                    tx.read(source, &|view| {
                        Ok(view.finish(
                            NodeEvaluationResult::from_version(version_ab(2 + step, 0))
                                .with_output_identity(format!("source-{step}")),
                        ))
                    })?;
                    tx.read(dependent, &|view| {
                        let version = view.read_aspect_version(source, ASPECT_A)?;
                        Ok(view.finish(
                            NodeEvaluationResult::from_version(version)
                                .with_output_identity(format!("dependent-{step}")),
                        ))
                    })?;
                    Ok(())
                })
                .unwrap();
            if step % 3 == 2 {
                runtime.switch_branch(main.clone()).unwrap();
                runtime
                    .restore_branch_snapshot(main.clone(), &snapshot)
                    .unwrap();
                runtime.switch_branch(feature.clone()).unwrap();
            }
        }

        (
            runtime.observe().replay_for_branch(feature.id),
            runtime
                .observe()
                .lineage_chain_for_node(dependent)
                .to_owned_records(),
            runtime.observe().explain(dependent).unwrap(),
        )
    }

    let development = run(SignalRuntimePolicy::development());
    let operational = run(SignalRuntimePolicy::operational());

    assert!(replay_slices_equivalent(&development.0, &operational.0));
    assert!(lineage_records_equivalent(&development.1, &operational.1));
    assert_eq!(development.2.state, operational.2.state);
    assert_eq!(development.2.output_change, operational.2.output_change);
    assert_eq!(development.2.upstream.len(), operational.2.upstream.len());
}
