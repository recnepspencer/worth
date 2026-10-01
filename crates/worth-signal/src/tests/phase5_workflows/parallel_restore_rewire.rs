use crate::facade::{
    BoundedSignalInputs, DeclaredSignalInput, EvaluationCondition, EvaluationRequestMode,
    NodeContract, NodeEvaluationResult, SignalError, SignalGraph, SignalRuntime,
    SignalRuntimePolicy,
};
use crate::tests::leased_execution::support::{authority, request};
use crate::tests::support::{mask_b, version_ab, DependencyBatchBuilder, ASPECT_A, ASPECT_B};

fn bounded(inputs: impl IntoIterator<Item = DeclaredSignalInput>) -> NodeContract {
    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new(inputs))
}

#[test]
fn dynamic_rewire_threshold_session_with_leased_restore_preserves_subscriber_sets() {
    fn run(workers: usize) -> bool {
        let mut runtime = SignalRuntime::builder(SignalGraph::new())
            .with_kernel_defaults()
            .build();
        runtime.set_runtime_policy(SignalRuntimePolicy::development().with_history_limit(8));
        let selector = runtime
            .graph_mut()
            .node()
            .with_contract(bounded([]))
            .output_identity()
            .build();
        let left = runtime
            .graph_mut()
            .node()
            .with_contract(bounded([]))
            .output_identity()
            .build();
        let right = runtime
            .graph_mut()
            .node()
            .with_contract(bounded([]))
            .output_identity()
            .build();
        let left_gate = runtime
            .graph_mut()
            .node()
            .with_contract(bounded([DeclaredSignalInput::new(left, ASPECT_A)]))
            .condition(EvaluationCondition::DeltaThreshold(2.0))
            .output_identity()
            .build();
        let right_gate = runtime
            .graph_mut()
            .node()
            .with_contract(bounded([DeclaredSignalInput::new(right, ASPECT_B)]))
            .aspect_filter(mask_b())
            .output_identity()
            .build();
        let target = runtime
            .graph_mut()
            .node()
            .with_contract(bounded([
                DeclaredSignalInput::new(selector, ASPECT_A),
                DeclaredSignalInput::new(left_gate, ASPECT_A),
                DeclaredSignalInput::new(right_gate, ASPECT_B),
            ]))
            .output_identity()
            .build();
        let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
        dependencies
            .append_dependency(left_gate, left, ASPECT_A)
            .unwrap()
            .append_dependency(right_gate, right, ASPECT_B)
            .unwrap();
        dependencies.commit().unwrap();

        let evaluate = |runtime: &mut SignalRuntime<(), (), (), (), ()>,
                        step: Option<u64>|
         -> Result<(), SignalError> {
            let lease = authority()
                .request_lease(request(workers, 10_000_000))
                .map_err(|_| SignalError::invalid_input("rewire host lease denied"))?;
            runtime.evaluate_checked(
                &[left_gate, right_gate, target],
                EvaluationRequestMode::Default,
                &(),
                &|ctx| {
                    ctx.work()
                        .checkpoint(128)
                        .map_err(|_| SignalError::invalid_input("rewire kernel work exhausted"))?;
                    let node = ctx.node();
                    let result = if node == left_gate {
                        let a = ctx.read(left, ASPECT_A)?;
                        NodeEvaluationResult::from_version(version_ab(a, 0)).with_output_identity(
                            step.map_or("left-gate".to_string(), |step| {
                                format!("left-gate-{step}")
                            }),
                        )
                    } else if node == right_gate {
                        let b = ctx.read(right, ASPECT_B)?;
                        NodeEvaluationResult::from_version(version_ab(0, b)).with_output_identity(
                            step.map_or("right-gate".to_string(), |step| {
                                format!("right-gate-{step}")
                            }),
                        )
                    } else if node == target {
                        let route = ctx.read(selector, ASPECT_A)?;
                        if route % 2 == 1 {
                            let a = ctx.read(left_gate, ASPECT_A)?;
                            NodeEvaluationResult::from_version(version_ab(a, 0))
                                .with_output_identity(
                                    step.map_or("target-left".to_string(), |step| {
                                        format!("target-left-{step}")
                                    }),
                                )
                        } else {
                            let b = ctx.read(right_gate, ASPECT_B)?;
                            NodeEvaluationResult::from_version(version_ab(0, b))
                                .with_output_identity(
                                    step.map_or("target-right".to_string(), |step| {
                                        format!("target-right-{step}")
                                    }),
                                )
                        }
                    } else {
                        return Err(SignalError::invalid_input("unknown rewire node"));
                    };
                    Ok(ctx.finish(result))
                },
                &lease,
            )?;
            Ok(())
        };

        runtime
            .transaction(&mut (), |tx| {
                tx.read(selector, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(1, 0))
                            .with_output_identity("route-left"),
                    ))
                })?;
                tx.read(left, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(10, 0))
                            .with_output_identity("left-seed"),
                    ))
                })?;
                tx.read(right, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(0, 20))
                            .with_output_identity("right-seed"),
                    ))
                })?;
                Ok(())
            })
            .unwrap();
        evaluate(&mut runtime, None).unwrap();
        let main = runtime.observe().current_branch();
        let main_snapshot = runtime.capture_snapshot().unwrap();
        let feature = runtime.create_branch("feature-rewire-parallel").unwrap();
        runtime.switch_branch(feature.clone()).unwrap();

        for step in 0..10_u64 {
            runtime
                .transaction(&mut (), |tx| {
                    tx.mark_dirty(selector, ASPECT_A)?;
                    if step % 2 == 0 {
                        tx.mark_dirty(right, ASPECT_B)?;
                    } else {
                        tx.mark_dirty(left, ASPECT_A)?;
                    }
                    tx.read(selector, &|view| {
                        Ok(view.finish(
                            NodeEvaluationResult::from_version(version_ab(2 + step, 0))
                                .with_output_identity(format!("route-{step}")),
                        ))
                    })?;
                    tx.read(left, &|view| {
                        Ok(view.finish(
                            NodeEvaluationResult::from_version(version_ab(10 + step, 0))
                                .with_output_identity(format!("left-{step}")),
                        ))
                    })?;
                    tx.read(right, &|view| {
                        Ok(view.finish(
                            NodeEvaluationResult::from_version(version_ab(0, 20 + step))
                                .with_output_identity(format!("right-{step}")),
                        ))
                    })?;
                    Ok(())
                })
                .unwrap();
            evaluate(&mut runtime, Some(step)).unwrap();
        }
        let feature_snapshot = runtime.capture_branch_snapshot(feature.clone()).unwrap();
        runtime.switch_branch(main.clone()).unwrap();
        runtime
            .restore_branch_snapshot(main, &main_snapshot)
            .unwrap();
        assert!(runtime
            .graph()
            .depends_on(target, left_gate, ASPECT_A)
            .unwrap());
        assert!(!runtime
            .graph()
            .depends_on(target, right_gate, ASPECT_B)
            .unwrap());
        assert!(runtime
            .graph()
            .subscribers_of(left_gate)
            .unwrap()
            .contains(&target));

        runtime.switch_branch(feature.clone()).unwrap();
        runtime
            .restore_branch_snapshot(feature, &feature_snapshot)
            .unwrap();
        let feature_left = runtime
            .graph()
            .depends_on(target, left_gate, ASPECT_A)
            .unwrap();
        let feature_right = runtime
            .graph()
            .depends_on(target, right_gate, ASPECT_B)
            .unwrap();
        assert_ne!(feature_left, feature_right);
        let active = if feature_left { left_gate } else { right_gate };
        let inactive = if feature_left { right_gate } else { left_gate };
        assert!(runtime
            .graph()
            .subscribers_of(active)
            .unwrap()
            .contains(&target));
        assert!(!runtime
            .graph()
            .subscribers_of(inactive)
            .unwrap()
            .contains(&target));
        feature_left
    }
    let baseline = run(1);
    for workers in [2, 4] {
        assert_eq!(run(workers), baseline);
    }
}
