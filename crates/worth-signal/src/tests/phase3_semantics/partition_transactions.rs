use crate::facade::{
    ChangedRegion, NodeEvaluationResult, NodeState, PartitionSubscription, SignalError,
    SignalGraph, SignalRuntime,
};
use crate::tests::support::{
    version_ab, DependencyBatchBuilder, GraphDependencyBatchExt, ASPECT_A,
};

#[test]
fn transaction_partition_seed_resolves_to_exact_matching_cause_after_commit() {
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
    let source = runtime.graph_mut().node().partitioned_output().build();
    let matching = runtime.graph_mut().node().build();
    let non_matching = runtime.graph_mut().node().build();
    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
    dependencies
        .append_partition_dependency(matching, source, ASPECT_A, "wing")
        .unwrap()
        .append_partition_dependency(non_matching, source, ASPECT_A, "tail")
        .unwrap();
    dependencies.commit().unwrap();

    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(matching, &|view| {
                let _ = view.read_partitioned_aspect_version(
                    source,
                    ASPECT_A,
                    PartitionSubscription::whole_partition("wing"),
                )?;
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(10, 0))))
            })?;
            tx.read(non_matching, &|view| {
                let _ = view.read_partitioned_aspect_version(
                    source,
                    ASPECT_A,
                    PartitionSubscription::whole_partition("tail"),
                )?;
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(20, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.mark_dirty_with_regions(source, ASPECT_A, &[ChangedRegion::new("wing")])?;
            Ok(())
        })
        .unwrap();

    assert_eq!(runtime.graph().get_state(source).unwrap(), NodeState::Dirty);
    assert_eq!(runtime.graph().dependencies_of(matching).unwrap().len(), 1);
    assert_eq!(
        runtime.graph().dependencies_of(non_matching).unwrap().len(),
        1
    );
    assert_eq!(
        runtime.graph().get_state(matching).unwrap(),
        NodeState::Clean
    );
    assert_eq!(
        runtime.graph().get_state(non_matching).unwrap(),
        NodeState::Clean
    );

    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_changed_region(ChangedRegion::new("wing")),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    assert_eq!(
        runtime.graph().get_state(matching).unwrap(),
        NodeState::Dirty
    );
    assert_eq!(
        runtime.graph().get_state(non_matching).unwrap(),
        NodeState::Clean
    );
    let causes = runtime.graph().pending_causes(matching).unwrap();
    assert_eq!(causes.len(), 1);
    assert_eq!(causes[0].key.aspect, ASPECT_A);
    assert!(causes[0].changed_scopes.iter().any(|scope| scope
        .path()
        .segments()
        .first()
        .is_some_and(|part| part == "wing")));
}

#[test]
fn partition_scoped_runtime_reads_do_not_widen_captured_dependencies() {
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
    let source = runtime.graph_mut().node().partitioned_output().build();
    let matching = runtime.graph_mut().node().build();
    let non_matching = runtime.graph_mut().node().build();
    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_changed_region(ChangedRegion::new("wing").with_detail("rib-12")),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
    dependencies
        .append_partition_dependency(matching, source, ASPECT_A, "wing")
        .unwrap()
        .append_partition_dependency(non_matching, source, ASPECT_A, "tail")
        .unwrap();
    dependencies.commit().unwrap();

    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(matching, &|view| {
                let _ = view.read_partitioned_aspect_version(
                    source,
                    ASPECT_A,
                    PartitionSubscription::whole_partition("wing"),
                )?;
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(10, 0))))
            })?;
            tx.read(non_matching, &|view| {
                let _ = view.read_partitioned_aspect_version(
                    source,
                    ASPECT_A,
                    PartitionSubscription::whole_partition("tail"),
                )?;
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(20, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.mark_dirty_with_regions(
                source,
                ASPECT_A,
                &[ChangedRegion::new("wing").with_detail("rib-12")],
            )?;
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_changed_region(ChangedRegion::new("wing").with_detail("rib-12")),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    assert_eq!(
        runtime.graph().get_state(matching).unwrap(),
        NodeState::Dirty
    );
    assert_eq!(
        runtime.graph().get_state(non_matching).unwrap(),
        NodeState::Clean
    );
}

#[test]
fn transaction_rollback_after_partition_local_evaluation_restores_clean_states() {
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
    let source = runtime.graph_mut().node().partitioned_output().build();
    let matching = runtime.graph_mut().node().build();
    let non_matching = runtime.graph_mut().node().build();
    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_changed_region(ChangedRegion::new("wing").with_detail("rib-12")),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime
        .graph_mut()
        .append_partition_detail_dependency(matching, source, ASPECT_A, "wing", "rib-12")
        .unwrap();
    runtime
        .graph_mut()
        .append_partition_dependency(non_matching, source, ASPECT_A, "tail")
        .unwrap();

    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(matching, &|view| {
                let _ = view.read_partitioned_aspect_version(
                    source,
                    ASPECT_A,
                    PartitionSubscription::partition_and_detail("wing", "rib-12"),
                )?;
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(10, 0))))
            })?;
            tx.read(non_matching, &|view| {
                let _ = view.read_partitioned_aspect_version(
                    source,
                    ASPECT_A,
                    PartitionSubscription::whole_partition("tail"),
                )?;
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(20, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let err = runtime.transaction(request_execution, &mut (), |tx| {
        tx.mark_dirty_with_regions(
            source,
            ASPECT_A,
            &[ChangedRegion::new("wing").with_detail("rib-12")],
        )?;
        tx.read(source, &|view| {
            Ok(view.finish(
                NodeEvaluationResult::from_version(version_ab(2, 0))
                    .with_changed_region(ChangedRegion::new("wing").with_detail("rib-12")),
            ))
        })?;
        tx.read(matching, &|view| {
            let _ = view.read_partitioned_aspect_version(
                source,
                ASPECT_A,
                PartitionSubscription::partition_and_detail("wing", "rib-12"),
            )?;
            Ok(view.finish(NodeEvaluationResult::from_version(version_ab(11, 0))))
        })?;
        Err(SignalError::invalid_input("rollback localized wave"))
    });
    assert!(err.is_err());

    assert_eq!(runtime.graph().get_state(source).unwrap(), NodeState::Clean);
    assert_eq!(
        runtime.graph().get_state(matching).unwrap(),
        NodeState::Clean
    );
    assert_eq!(
        runtime.graph().get_state(non_matching).unwrap(),
        NodeState::Clean
    );
}

#[test]
fn committed_partition_local_evaluation_preserves_changed_region_explanation_and_metrics() {
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
    let source = runtime.graph_mut().node().partitioned_output().build();
    let matching = runtime.graph_mut().node().build();
    let non_matching = runtime.graph_mut().node().build();
    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_changed_region(ChangedRegion::new("wing").with_detail("rib-12")),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime
        .graph_mut()
        .append_partition_detail_dependency(matching, source, ASPECT_A, "wing", "rib-12")
        .unwrap();
    runtime
        .graph_mut()
        .append_partition_dependency(non_matching, source, ASPECT_A, "tail")
        .unwrap();

    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(matching, &|view| {
                let _ = view.read_partitioned_aspect_version(
                    source,
                    ASPECT_A,
                    PartitionSubscription::partition_and_detail("wing", "rib-12"),
                )?;
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(10, 0))))
            })?;
            tx.read(non_matching, &|view| {
                let _ = view.read_partitioned_aspect_version(
                    source,
                    ASPECT_A,
                    PartitionSubscription::whole_partition("tail"),
                )?;
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(20, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.mark_dirty_with_regions(
                source,
                ASPECT_A,
                &[ChangedRegion::new("wing").with_detail("rib-12")],
            )?;
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_changed_region(ChangedRegion::new("wing").with_detail("rib-12")),
                ))
            })?;
            tx.read(matching, &|view| {
                let _ = view.read_partitioned_aspect_version(
                    source,
                    ASPECT_A,
                    PartitionSubscription::partition_and_detail("wing", "rib-12"),
                )?;
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(11, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    assert_eq!(
        runtime.graph().get_state(matching).unwrap(),
        NodeState::Clean
    );
    assert_eq!(
        runtime.graph().get_state(non_matching).unwrap(),
        NodeState::Clean
    );
    let explanation = runtime.observe().explain(source).unwrap();
    assert!(explanation
        .changed_regions
        .iter()
        .any(|region| { region.path().segments() == ["wing", "rib-12"] }));
}
