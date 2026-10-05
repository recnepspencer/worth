use crate::facade::{
    lineage_records_equivalent, replay_slices_equivalent, BoundedSignalInputs, DeclaredSignalInput,
    EvaluationCondition, EvaluationRequestMode, LineageRecord, NodeContract, NodeEvaluationResult,
    NodeExplanation, ReplaySlice, SignalError, SignalGraph, SignalRuntime, SignalRuntimePolicy,
};
use crate::tests::leased_execution::support::{authority, request};
use crate::tests::support::{mask_b, version_ab, DependencyBatchBuilder, ASPECT_A, ASPECT_B};

fn contract(inputs: impl IntoIterator<Item = DeclaredSignalInput>) -> NodeContract {
    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new(inputs))
}

#[test]
fn long_session_replay_and_lineage_stay_equivalent_for_worker_limits() {
    fn run(workers: usize) -> (ReplaySlice, Vec<LineageRecord>, NodeExplanation) {
        let mut runtime = SignalRuntime::builder(SignalGraph::new())
            .with_kernel_defaults()
            .build();
        runtime.set_runtime_policy(SignalRuntimePolicy::kernel().with_history_limit(8));
        let source = runtime
            .graph_mut()
            .node()
            .with_contract(contract([]))
            .output_identity()
            .build();
        let a_gate = runtime
            .graph_mut()
            .node()
            .with_contract(contract([DeclaredSignalInput::new(source, ASPECT_A)]))
            .condition(EvaluationCondition::DeltaThreshold(2.0))
            .output_identity()
            .build();
        let b_gate = runtime
            .graph_mut()
            .node()
            .with_contract(contract([DeclaredSignalInput::new(source, ASPECT_B)]))
            .aspect_filter(mask_b())
            .output_identity()
            .build();
        let sink = runtime
            .graph_mut()
            .node()
            .with_contract(contract([
                DeclaredSignalInput::new(a_gate, ASPECT_A),
                DeclaredSignalInput::new(b_gate, ASPECT_B),
            ]))
            .output_identity()
            .build();
        let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
        dependencies
            .append_dependency(a_gate, source, ASPECT_A)
            .unwrap()
            .append_dependency(b_gate, source, ASPECT_B)
            .unwrap()
            .append_dependency(sink, a_gate, ASPECT_A)
            .unwrap()
            .append_dependency(sink, b_gate, ASPECT_B)
            .unwrap();
        dependencies.commit().unwrap();

        let evaluate = |runtime: &mut SignalRuntime<(), (), (), (), ()>,
                        step: Option<u64>|
         -> Result<(), SignalError> {
            let lease = authority()
                .request_lease(request(workers, 10_000_000))
                .map_err(|_| SignalError::invalid_input("phase5 workflow lease denied"))?;
            runtime.evaluate_checked(
                &[source, a_gate, b_gate, sink],
                EvaluationRequestMode::Default,
                &(),
                &|ctx| {
                    ctx.work()
                        .checkpoint(128)
                        .map_err(|_| SignalError::invalid_input("phase5 kernel work exhausted"))?;
                    let node = ctx.node();
                    let result = if node == source {
                        let (a, b, identity) = match step {
                            Some(step) => (2 + step, 10 + step % 3, format!("source-{step}")),
                            None => (1, 10, "seed-source".to_string()),
                        };
                        NodeEvaluationResult::from_version(version_ab(a, b))
                            .with_output_identity(identity)
                    } else if node == a_gate {
                        let a = ctx.read(source, ASPECT_A)?;
                        NodeEvaluationResult::from_version(version_ab(a, 0)).with_output_identity(
                            step.map_or("seed-a".to_string(), |step| format!("a-{step}")),
                        )
                    } else if node == b_gate {
                        let b = ctx.read(source, ASPECT_B)?;
                        NodeEvaluationResult::from_version(version_ab(0, b)).with_output_identity(
                            step.map_or("seed-b".to_string(), |step| format!("b-{step}")),
                        )
                    } else if node == sink {
                        let a = ctx.read(a_gate, ASPECT_A)?;
                        let b = ctx.read(b_gate, ASPECT_B)?;
                        NodeEvaluationResult::from_version(version_ab(a, b))
                            .with_output_identity(
                                step.map_or("seed-sink".to_string(), |step| format!("sink-{step}")),
                            )
                            .with_continuity_token("surface")
                    } else {
                        return Err(SignalError::invalid_input("unknown phase5 node"));
                    };
                    Ok(ctx.finish(result))
                },
                &lease,
            )?;
            Ok(())
        };
        evaluate(&mut runtime, None).unwrap();
        let main = runtime.observe().current_branch();
        let snapshot = runtime.capture_snapshot().unwrap();
        let feature = runtime.create_branch("executor-feature").unwrap();
        runtime.switch_branch(feature.clone()).unwrap();
        for step in 0..20_u64 {
            runtime
                .transaction(&mut (), |tx| {
                    tx.mark_dirty(source, ASPECT_A)?;
                    if step % 4 == 0 {
                        tx.mark_dirty(source, ASPECT_B)?;
                    }
                    Ok(())
                })
                .unwrap();
            evaluate(&mut runtime, Some(step)).unwrap();
            if step % 5 == 4 {
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
                .lineage_chain_for_node(sink)
                .to_owned_records(),
            runtime.observe().explain(sink).unwrap(),
        )
    }

    let baseline = run(1);
    for workers in [2, 4] {
        let actual = run(workers);
        assert!(replay_slices_equivalent(&baseline.0, &actual.0));
        assert!(lineage_records_equivalent(&baseline.1, &actual.1));
        assert_eq!(baseline.2.state, actual.2.state);
        assert_eq!(baseline.2.output_change, actual.2.output_change);
        assert_eq!(baseline.2.upstream.len(), actual.2.upstream.len());
    }
}
