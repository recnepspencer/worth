use crate::facade::*;
use crate::tests::support::{version_ab, ASPECT_A, ASPECT_B};

fn build_scoped_candidate_runtime() -> (
    SignalRuntime<(), (), (), (), ()>,
    SignalBranchHandle,
    SignalBranchHandle,
    NodeId,
    NodeId,
    NodeId,
) {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let support = runtime
        .graph_mut()
        .node()
        .reads_aspects([ASPECT_A])
        .produces_aspects([ASPECT_A])
        .build();
    let primary = runtime
        .graph_mut()
        .node()
        .reads_aspects([ASPECT_A])
        .produces_aspects([ASPECT_A])
        .build();
    let companion = runtime
        .graph_mut()
        .node()
        .reads_aspects([ASPECT_A])
        .produces_aspects([ASPECT_A])
        .build();
    runtime
        .graph_mut()
        .append_dependency(primary, support, ASPECT_A)
        .unwrap();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(support, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            tx.read(primary, &|view| {
                let upstream = view.read_aspect_version(support, ASPECT_A)?;
                Ok(view.finish(NodeEvaluationResult::from_version(upstream)))
            })?;
            tx.read(companion, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(2, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.current_branch();
    let feature = runtime.create_branch("feature-candidate-scope").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(primary, ASPECT_A)?;
            tx.read(primary, &|view| {
                let upstream = view.read_aspect_version(support, ASPECT_A)?;
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(
                    upstream.get(ASPECT_A) + 100,
                    0,
                ))))
            })?;
            tx.mark_dirty(companion, ASPECT_A)?;
            tx.read(companion, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(202, 0))))
            })?;
            Ok(())
        })
        .unwrap();
    runtime.switch_branch(main.clone()).unwrap();

    (runtime, feature, main, support, primary, companion)
}

#[test]
fn selected_node_scope_changes_candidate_truth_before_identity_and_support_planning() {
    let (mut runtime, feature, main, support, primary, companion) =
        build_scoped_candidate_runtime();

    let (full_candidate_breadth, full_selected_semantics, full_boundary_candidate_nodes) = {
        let full_plan = runtime
            .merge_raw()
            .from(feature.clone())
            .into(main.clone())
            .plan()
            .expect("full-branch merge should plan");
        (
            full_plan.plan().planned_candidates().breadth(),
            full_plan.plan().selected_semantics().clone(),
            full_plan
                .plan()
                .scoped_candidates()
                .boundary_candidate_nodes()
                .to_vec(),
        )
    };
    let selected_plan = runtime
        .merge_raw()
        .from(feature)
        .into(main)
        .selected_nodes([primary])
        .plan()
        .expect("selected-node merge should plan");

    assert_eq!(full_candidate_breadth, 2);
    assert_eq!(
        selected_plan.plan().planned_candidates().nodes,
        vec![primary],
        "selected scope must narrow the admitted candidate set before downstream planning"
    );
    assert!(
        !selected_plan
            .plan()
            .planned_candidates()
            .nodes
            .contains(&companion),
        "non-requested source journal nodes must not survive scoped candidate lowering"
    );
    assert_eq!(
        selected_plan.plan().selected_semantics(),
        &full_selected_semantics,
        "scoped candidate lowering should not invent different merge semantics when only the requested scope changed"
    );
    let scoped = selected_plan.plan().scoped_candidates();
    assert_eq!(
        scoped.boundary_candidate_nodes(),
        full_boundary_candidate_nodes.as_slice(),
        "scoped candidate lowering should preserve the same branch-local boundary witness and only narrow admitted scope"
    );
    assert_eq!(scoped.requested_nodes(), &[primary]);
    assert!(scoped.skipped_nodes().is_empty());
    assert_eq!(scoped.support_closure_nodes(), &[support]);
    assert_eq!(scoped.breadth_summary().requested_scope_width, 1);
    assert_eq!(scoped.breadth_summary().admitted_candidate_width, 1);
    assert_eq!(scoped.breadth_summary().skipped_scope_width, 0);
    assert_eq!(scoped.breadth_summary().no_op_scope_width, 0);
    assert_eq!(scoped.breadth_summary().support_closure_width, 1);
    assert_eq!(
        selected_plan.plan().identity_correspondence().records.len(),
        1
    );
    assert_eq!(
        selected_plan.plan().identity_correspondence().records[0].source_node,
        primary
    );
    assert_eq!(selected_plan.plan().node_plan().len(), 1);
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .scoped_candidate_lowering_count,
        2
    );
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .scoped_candidate_full_branch_count,
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .scoped_candidate_selected_node_count,
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .scoped_candidate_broad_scan_denial_count,
        0,
        "bounded scoped candidate lowering must not touch the broad-scan denial path"
    );
}

#[test]
fn selected_aspect_scope_preserves_requested_and_skipped_loci_without_widening_candidates() {
    let (mut runtime, feature, main, _support, primary, companion) =
        build_scoped_candidate_runtime();

    let (
        full_selected_strategy_digest,
        full_selected_conflict_policy_digest,
        full_selected_conflict_isolation_digest,
        full_selected_identity_matcher_digest,
        full_selected_source_only_policy_digest,
        full_selected_deletion_policy_digest,
        full_boundary_candidate_nodes,
    ) = {
        let full_plan = runtime
            .merge_raw()
            .from(feature.clone())
            .into(main.clone())
            .plan()
            .expect("full-branch merge should plan");
        (
            full_plan.plan().selected_strategy_digest().to_owned(),
            full_plan
                .plan()
                .selected_conflict_policy_digest()
                .to_owned(),
            full_plan
                .plan()
                .selected_conflict_isolation_digest()
                .to_owned(),
            full_plan
                .plan()
                .selected_identity_matcher_digest()
                .to_owned(),
            full_plan
                .plan()
                .selected_source_only_policy_digest()
                .to_owned(),
            full_plan
                .plan()
                .selected_deletion_policy_digest()
                .to_owned(),
            full_plan
                .plan()
                .scoped_candidates()
                .boundary_candidate_nodes()
                .to_vec(),
        )
    };
    let plan = runtime
        .merge_raw()
        .from(feature)
        .into(main)
        .selected_aspects([
            SignalSelectedAspectRequestEntry::new(primary, ASPECT_A),
            SignalSelectedAspectRequestEntry::new(companion, ASPECT_B),
        ])
        .plan()
        .expect("selected-aspect merge should plan");

    let scoped = plan.plan().scoped_candidates();
    assert_eq!(
        scoped.planned_candidates().nodes,
        vec![primary],
        "selected-aspect lowering must only admit nodes whose requested aspect is structurally relevant"
    );
    assert_eq!(
        plan.plan().selected_strategy_digest(),
        full_selected_strategy_digest,
        "selected-aspect candidate lowering should not change the global merge strategy selection"
    );
    assert_eq!(
        plan.plan().selected_conflict_policy_digest(),
        full_selected_conflict_policy_digest,
        "selected-aspect candidate lowering should not change the global conflict policy selection"
    );
    assert_eq!(
        plan.plan().selected_conflict_isolation_digest(),
        full_selected_conflict_isolation_digest,
        "selected-aspect candidate lowering should not change the global conflict-isolation selection"
    );
    assert_eq!(
        plan.plan().selected_identity_matcher_digest(),
        full_selected_identity_matcher_digest,
        "selected-aspect candidate lowering should not change the global identity matcher selection"
    );
    assert_eq!(
        plan.plan().selected_source_only_policy_digest(),
        full_selected_source_only_policy_digest,
        "selected-aspect candidate lowering should not change the global source-only policy selection"
    );
    assert_eq!(
        plan.plan().selected_deletion_policy_digest(),
        full_selected_deletion_policy_digest,
        "selected-aspect candidate lowering should not change the global deletion policy selection"
    );
    assert_eq!(
        scoped.boundary_candidate_nodes(),
        full_boundary_candidate_nodes.as_slice(),
        "selected-aspect lowering should stay anchored to the same branch-local boundary candidate witness"
    );
    assert_eq!(
        scoped.requested_aspects(),
        &[
            SignalSelectedAspectRequestEntry::new(primary, ASPECT_A),
            SignalSelectedAspectRequestEntry::new(companion, ASPECT_B),
        ]
    );
    assert!(scoped.skipped_aspects().is_empty());
    assert_eq!(
        scoped.no_op_aspects(),
        &[SignalSelectedAspectRequestEntry::new(companion, ASPECT_B)],
        "structurally irrelevant node-local aspect requests must be preserved as no-op loci, not silently widened into admitted candidates"
    );
    assert_eq!(scoped.breadth_summary().requested_scope_width, 2);
    assert_eq!(scoped.breadth_summary().admitted_candidate_width, 1);
    assert_eq!(scoped.breadth_summary().skipped_scope_width, 0);
    assert_eq!(scoped.breadth_summary().no_op_scope_width, 1);
    assert_eq!(plan.plan().identity_correspondence().records.len(), 1);
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .scoped_candidate_selected_aspect_count,
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .scoped_candidate_admitted_breadth,
        3,
        "telemetry should reflect the full-branch parity baseline plus the one admitted selected-aspect candidate"
    );
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .scoped_candidate_no_op_breadth,
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .scoped_candidate_broad_scan_denial_count,
        0,
        "selected-aspect candidate lowering must stay within branch-local mutation scope"
    );
}
