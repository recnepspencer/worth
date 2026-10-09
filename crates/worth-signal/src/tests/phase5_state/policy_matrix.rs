use crate::facade::{
    LineageRecord, NodeEvaluationResult, ReplaySlice, SignalGraph, SignalRuntime,
    SignalRuntimePolicy, SnapshotRestoreLineageMode,
};
use crate::tests::support::{version_ab, ASPECT_A};

#[test]
fn replay_and_lineage_overlap_stay_equivalent_across_runtime_policy_matrix() {
    fn run_workload(policy: SignalRuntimePolicy) -> (ReplaySlice, ReplaySlice, Vec<LineageRecord>) {
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
        let mut runtime_ctx = ();

        runtime
            .transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.read(source, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(1, 0))
                            .with_output_identity("artifact-main"),
                    ))
                })?;
                Ok(())
            })
            .unwrap();

        let main = runtime.observe().current_branch();
        let feature = runtime.create_branch("feature-policy").unwrap();
        let main_snapshot = runtime
            .capture_snapshot()
            .expect("snapshot capture should succeed without managed queue bindings");

        runtime.switch_branch(feature.clone()).unwrap();
        runtime
            .transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.mark_dirty(source, ASPECT_A)?;
                tx.read(source, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(2, 0))
                            .with_output_identity("artifact-feature"),
                    ))
                })?;
                Ok(())
            })
            .unwrap();
        let feature_snapshot = runtime.capture_branch_snapshot(feature.clone()).unwrap();
        runtime
            .restore_branch_snapshot(feature.clone(), &feature_snapshot)
            .unwrap();
        runtime.switch_branch(main).unwrap();
        runtime.restore_snapshot(&main_snapshot).unwrap();

        (
            runtime
                .observe()
                .replay_for_branch(runtime.observe().current_branch().id),
            runtime.observe().replay_for_branch(feature.id),
            runtime
                .graph()
                .observe()
                .lineage_for_node(source)
                .to_owned_records(),
        )
    }

    let operational = run_workload(
        SignalRuntimePolicy::operational()
            .with_snapshot_restore_lineage_mode(SnapshotRestoreLineageMode::CompactGlobal),
    );
    let development = run_workload(
        SignalRuntimePolicy::development()
            .with_snapshot_restore_lineage_mode(SnapshotRestoreLineageMode::CompactGlobal),
    );
    let forensic = run_workload(
        SignalRuntimePolicy::forensic()
            .with_snapshot_restore_lineage_mode(SnapshotRestoreLineageMode::CompactGlobal),
    );

    for (left_main, left_feature, left_lineage, right_main, right_feature, right_lineage) in [
        (
            &operational.0,
            &operational.1,
            &operational.2,
            &development.0,
            &development.1,
            &development.2,
        ),
        (
            &development.0,
            &development.1,
            &development.2,
            &forensic.0,
            &forensic.1,
            &forensic.2,
        ),
        (
            &operational.0,
            &operational.1,
            &operational.2,
            &forensic.0,
            &forensic.1,
            &forensic.2,
        ),
    ] {
        assert!(
            left_main == right_main,
            "main-branch replay should remain equivalent across runtime-policy richness changes"
        );
        assert!(
            left_feature == right_feature,
            "feature-branch replay should remain equivalent across runtime-policy richness changes"
        );
        assert!(
            left_lineage == right_lineage,
            "lineage on the overlapping guaranteed surface should remain equivalent across runtime-policy richness changes"
        );
    }
}
