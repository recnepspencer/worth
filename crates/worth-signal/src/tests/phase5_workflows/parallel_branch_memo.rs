use std::sync::atomic::{AtomicU32, Ordering};

use crate::facade::{
    mark_dirty, ArtifactTransitionKind, BoundedSignalInputs, DeclaredSignalInput,
    EvaluationCondition, EvaluationRequestMode, LineageRecordKind, NodeContract,
    NodeEvaluationResult, OutputChange, SignalError, SignalGraph, SignalRuntime,
    SignalRuntimePolicy,
};
use crate::tests::leased_execution::support::{authority, request};
use crate::tests::support::{
    define_keyed_computation, mask_b, version_ab, DependencyBatchBuilder, ASPECT_A, ASPECT_B,
};

fn bounded(inputs: impl IntoIterator<Item = DeclaredSignalInput>) -> NodeContract {
    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new(inputs))
}

#[test]
fn leased_branch_memo_rollback_preserves_branch_local_replay_and_cache_truth() {
    fn run(workers: usize) {
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
        runtime.set_runtime_policy(
            SignalRuntimePolicy::development()
                .with_history_limit(8)
                .with_detail_limit(4),
        );
        let source = runtime
            .graph_mut()
            .node()
            .with_contract(bounded([]))
            .output_identity()
            .build();
        let gated = runtime
            .graph_mut()
            .node()
            .with_contract(bounded([DeclaredSignalInput::new(source, ASPECT_A)]))
            .condition(EvaluationCondition::DeltaThreshold(2.0))
            .output_identity()
            .build();
        let filtered = runtime
            .graph_mut()
            .node()
            .with_contract(bounded([DeclaredSignalInput::new(source, ASPECT_B)]))
            .aspect_filter(mask_b())
            .output_identity()
            .build();
        let fused = runtime
            .graph_mut()
            .node()
            .with_contract(bounded([
                DeclaredSignalInput::new(gated, ASPECT_A),
                DeclaredSignalInput::new(filtered, ASPECT_B),
            ]))
            .output_identity()
            .build();
        let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
        dependencies
            .append_dependency(gated, source, ASPECT_A)
            .unwrap()
            .append_dependency(filtered, source, ASPECT_B)
            .unwrap()
            .append_dependency(fused, gated, ASPECT_A)
            .unwrap()
            .append_dependency(fused, filtered, ASPECT_B)
            .unwrap();
        dependencies.commit().unwrap();
        let family = define_keyed_computation(&mut runtime, "parallel-branch-memo", ());
        let keyed_def = family.keyed("mesh-cache");
        let keyed = keyed_def.node(&mut runtime);
        let memo = keyed_def.memoized("lod-0");
        let compute_calls = AtomicU32::new(0);

        let evaluate = |runtime: &mut SignalRuntime<(), (), (), (), ()>,
                        step: Option<u64>|
         -> Result<(), SignalError> {
            let lease = authority()
                .request_lease(request(workers, 10_000_000))
                .map_err(|_| SignalError::invalid_input("branch memo host lease denied"))?;
            runtime.evaluate_checked(
                &[gated, filtered, fused],
                EvaluationRequestMode::Default,
                &(),
                &|ctx| {
                    ctx.work()
                        .checkpoint(128)
                        .map_err(|_| SignalError::invalid_input("memo kernel work exhausted"))?;
                    let result = if ctx.node() == gated {
                        let a = ctx.read(source, ASPECT_A)?;
                        NodeEvaluationResult::from_version(version_ab(a, 0)).with_output_identity(
                            step.map_or("gated-seed".to_string(), |step| format!("gated-{step}")),
                        )
                    } else if ctx.node() == filtered {
                        let b = ctx.read(source, ASPECT_B)?;
                        NodeEvaluationResult::from_version(version_ab(0, b)).with_output_identity(
                            step.map_or("filtered-seed".to_string(), |step| {
                                format!("filtered-{step}")
                            }),
                        )
                    } else if ctx.node() == fused {
                        let a = ctx.read(gated, ASPECT_A)?;
                        let b = ctx.read(filtered, ASPECT_B)?;
                        NodeEvaluationResult::from_version(version_ab(a, b))
                            .with_output_identity(
                                step.map_or("fused-seed".to_string(), |step| {
                                    format!("fused-{step}")
                                }),
                            )
                            .with_continuity_token("mesh-continuity")
                    } else {
                        return Err(SignalError::invalid_input("unknown memo node"));
                    };
                    Ok(ctx.finish(result))
                },
                &lease,
            )?;
            Ok(())
        };

        runtime
            .transaction(request_execution, &mut (), |tx| {
                tx.read(source, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(10, 100))
                            .with_output_identity("seed-ab"),
                    ))
                })?;
                Ok(())
            })
            .unwrap();
        evaluate(&mut runtime, None).unwrap();
        runtime
            .transaction(request_execution, &mut (), |tx| {
                tx.evaluate_keyed(keyed, &memo, &|view| {
                    compute_calls.fetch_add(1, Ordering::Relaxed);
                    let version = view.read_aspect_version(fused, ASPECT_A)?;
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version)
                            .with_output_identity("memo-seed")
                            .with_output_change(OutputChange::Refreshed),
                    ))
                })?;
                Ok(())
            })
            .unwrap();
        let main = runtime.observe().current_branch();
        let main_snapshot = runtime.capture_snapshot().unwrap();
        let feature = runtime.create_branch("parallel-feature").unwrap();
        runtime.switch_branch(feature.clone()).unwrap();

        for step in 0..12_u64 {
            runtime
                .transaction(request_execution, &mut (), |tx| {
                    tx.mark_dirty(source, ASPECT_A)?;
                    if step % 3 == 0 {
                        tx.mark_dirty(source, ASPECT_B)?;
                    }
                    tx.read(source, &|view| {
                        Ok(view.finish(
                            NodeEvaluationResult::from_version(version_ab(
                                20 + step,
                                100 + step % 2,
                            ))
                            .with_output_identity(format!("source-{step}")),
                        ))
                    })?;
                    Ok(())
                })
                .unwrap();
            evaluate(&mut runtime, Some(step)).unwrap();
            mark_dirty(runtime.graph_mut(), keyed, ASPECT_A).unwrap();
            runtime
                .transaction(request_execution, &mut (), |tx| {
                    tx.evaluate_keyed(keyed, &memo, &|view| {
                        compute_calls.fetch_add(1, Ordering::Relaxed);
                        let version = view.read_aspect_version(fused, ASPECT_A)?;
                        Ok(view.finish(NodeEvaluationResult::from_version(version)))
                    })?;
                    Ok(())
                })
                .unwrap();
            if step % 2 == 1 {
                let analysis = runtime.create_branch(format!("analysis-{step}")).unwrap();
                runtime.switch_branch(analysis).unwrap();
                let err = runtime.transaction(request_execution, &mut (), |tx| {
                    tx.mark_dirty(source, ASPECT_A)?;
                    tx.read(source, &|view| {
                        Ok(view.finish(
                            NodeEvaluationResult::from_version(version_ab(500 + step, 900))
                                .with_output_identity(format!("bad-{step}")),
                        ))
                    })?;
                    tx.read(fused, &|view| {
                        let a = view.read_aspect_version(gated, ASPECT_A)?;
                        let b = view.read_aspect_version(filtered, ASPECT_B)?;
                        Ok(view.finish(
                            NodeEvaluationResult::from_version(version_ab(
                                a.get(ASPECT_A),
                                b.get(ASPECT_B),
                            ))
                            .with_output_identity(format!("bad-fused-{step}")),
                        ))
                    })?;
                    Err(SignalError::invalid_input(
                        "synthetic parallel analysis rollback",
                    ))
                });
                assert!(err.is_err());
                runtime.switch_branch(feature.clone()).unwrap();
            }
        }
        assert_eq!(
            compute_calls.load(Ordering::Relaxed),
            1,
            "memoized keyed artifact should stay hot through branch churn"
        );
        let replay = runtime.observe().replay_for_branch(feature.id);
        assert!(replay
            .frames
            .iter()
            .all(|frame| frame.branch_id == feature.id));
        let lineage = runtime.observe().lineage_chain_for_node(fused);
        assert!(
            lineage.len() >= 2
                && lineage.iter().any(|record| matches!(
                    record.kind,
                    LineageRecordKind::SnapshotRestore { .. }
                        | LineageRecordKind::ArtifactTransition {
                            transition: ArtifactTransitionKind::Replaced
                                | ArtifactTransitionKind::Refreshed { .. },
                            ..
                        }
                ))
        );
        runtime.switch_branch(main.clone()).unwrap();
        runtime
            .restore_branch_snapshot(main, &main_snapshot)
            .unwrap();
        let restored = runtime
            .graph()
            .get_entry(source)
            .unwrap()
            .get_aspect_version();
        assert_eq!(restored.get(ASPECT_A), 10);
        assert_eq!(restored.get(ASPECT_B), 100);
    }
    for workers in [1, 2, 4] {
        run(workers);
    }
}
