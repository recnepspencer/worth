use crate::facade::{NodeEvaluationResult, SignalGraph, SignalRuntime};
use crate::tests::support::{version_ab, ASPECT_A};

#[test]
fn alternating_dynamic_rewire_across_branches_preserves_subscriber_integrity() {
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
    let selector = runtime.graph_mut().node().output_identity().build();
    let left = runtime.graph_mut().node().output_identity().build();
    let right = runtime.graph_mut().node().output_identity().build();
    let target = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(selector, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("route-left"),
                ))
            })?;
            tx.read(left, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(10, 0))
                        .with_output_identity("left-v1"),
                ))
            })?;
            tx.read(right, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(20, 0))
                        .with_output_identity("right-v1"),
                ))
            })?;
            tx.read(target, &|view| {
                let route = view.read_aspect_version(selector, ASPECT_A)?;
                let upstream = if route.get(ASPECT_A) % 2 == 1 {
                    left
                } else {
                    right
                };
                let version = view.read_aspect_version(upstream, ASPECT_A)?;
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version)
                        .with_output_identity(format!("target-from-{}", upstream.index())),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let main_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let feature = runtime.create_branch("feature-rewire").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(selector, ASPECT_A)?;
            tx.read(selector, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("route-right"),
                ))
            })?;
            tx.read(target, &|view| {
                let route = view.read_aspect_version(selector, ASPECT_A)?;
                let upstream = if route.get(ASPECT_A) % 2 == 1 {
                    left
                } else {
                    right
                };
                let version = view.read_aspect_version(upstream, ASPECT_A)?;
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version)
                        .with_output_identity(format!("target-from-{}", upstream.index())),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let feature_snapshot = runtime.capture_branch_snapshot(feature.clone()).unwrap();

    assert!(runtime.graph().depends_on(target, right, ASPECT_A).unwrap());
    assert!(!runtime.graph().depends_on(target, left, ASPECT_A).unwrap());

    runtime.switch_branch(main.clone()).unwrap();
    runtime
        .restore_branch_snapshot(main.clone(), &main_snapshot)
        .unwrap();
    assert!(runtime.graph().depends_on(target, left, ASPECT_A).unwrap());
    assert!(!runtime.graph().depends_on(target, right, ASPECT_A).unwrap());
    assert!(runtime
        .graph()
        .subscribers_of(left)
        .unwrap()
        .contains(&target));
    assert!(!runtime
        .graph()
        .subscribers_of(right)
        .unwrap()
        .contains(&target));

    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .restore_branch_snapshot(feature.clone(), &feature_snapshot)
        .unwrap();
    assert!(runtime.graph().depends_on(target, right, ASPECT_A).unwrap());
    assert!(!runtime.graph().depends_on(target, left, ASPECT_A).unwrap());
    assert!(runtime
        .graph()
        .subscribers_of(right)
        .unwrap()
        .contains(&target));
    assert!(!runtime
        .graph()
        .subscribers_of(left)
        .unwrap()
        .contains(&target));
}
