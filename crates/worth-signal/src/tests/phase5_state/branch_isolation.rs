use crate::facade::{
    LineageRecordKind, NodeEvaluationResult, ReplayEventKind, SignalGraph, SignalRuntime,
};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn runtime_branches_keep_evaluation_state_isolated_across_switches() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = SignalGraph::new();
    let source = graph.node().output_identity().build();
    let dependent = graph.node().build();
    graph
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();

    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            tx.read(dependent, &|view| {
                let result = if view.node() == source {
                    view.finish(
                        NodeEvaluationResult::from_version(version_ab(1, 0))
                            .with_output_identity("main-artifact"),
                    )
                } else {
                    let version = view.read_aspect_version(source, ASPECT_A)?;
                    view.finish(NodeEvaluationResult::from_version(version))
                };
                Ok(result)
            })?;
            Ok(())
        })
        .unwrap();

    let main_branch = runtime.observe().current_branch();
    let feature_branch = runtime.create_branch("feature-a").unwrap();

    runtime.switch_branch(feature_branch.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            tx.read(dependent, &|view| {
                let result = if view.node() == source {
                    view.finish(
                        NodeEvaluationResult::from_version(version_ab(2, 0))
                            .with_output_identity("feature-artifact"),
                    )
                } else {
                    let version = view.read_aspect_version(source, ASPECT_A)?;
                    view.finish(NodeEvaluationResult::from_version(version))
                };
                Ok(result)
            })?;
            Ok(())
        })
        .unwrap();

    assert_eq!(runtime.observe().current_branch().id, feature_branch.id);
    assert_eq!(
        runtime
            .graph()
            .get_entry(source)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        2
    );

    runtime.switch_branch(main_branch.clone()).unwrap();

    assert_eq!(runtime.observe().current_branch().id, main_branch.id);
    assert_eq!(
        runtime
            .graph()
            .get_entry(source)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        1
    );
    assert!(
        runtime
            .graph()
            .replay_events()
            .iter()
            .any(|event| event.kind == ReplayEventKind::BranchSwitched),
        "branch switching should emit replay events"
    );
    let ancestry = runtime.observe().branch_ancestry(feature_branch.id);
    assert_eq!(ancestry.first().unwrap().id, main_branch.id);
    assert_eq!(ancestry.last().unwrap().id, feature_branch.id);
}

#[test]
fn switching_existing_branch_does_not_emit_branched_from_lineage() {
    let graph = SignalGraph::new();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();

    let main_branch = runtime.observe().current_branch();
    let feature_branch = runtime.create_branch("feature").unwrap();
    runtime.switch_branch(feature_branch.clone()).unwrap();
    let lineage_after_create = runtime.graph().observe().lineage_records().len();

    runtime.switch_branch(main_branch.clone()).unwrap();

    let switch_records = runtime
        .graph()
        .observe()
        .lineage_records()
        .iter()
        .skip(lineage_after_create)
        .collect::<Vec<_>>();
    assert!(
        switch_records.iter().any(|record| {
            matches!(
                &record.kind,
                LineageRecordKind::BranchSwitch {
                    from_branch_id,
                    to_branch_id,
                    from_branch_display_name,
                    to_branch_display_name,
                } if *from_branch_id == feature_branch.id
                    && *to_branch_id == main_branch.id
                    && from_branch_display_name == "feature"
                    && to_branch_display_name == "main"
            )
        }),
        "branch switch should remain lineage-visible"
    );
    assert!(
        switch_records
            .iter()
            .all(|record| !matches!(record.kind, LineageRecordKind::BranchFork { .. })),
        "switching existing branches must not masquerade as branch creation"
    );
    assert_eq!(runtime.observe().current_branch().id, main_branch.id);
    assert_eq!(feature_branch.parent_branch_id, Some(main_branch.id));
}
