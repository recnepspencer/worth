//! Real computed results retain only the applied canonical prefix.
use super::{domain, effects, inputs, phased_workflow, start};

#[test]
fn failure_retains_only_the_canonical_prefix() {
    let _probe = phased_workflow::fold_executor::ComputeProbe::new();
    let mut workspace = phased_workflow::phased_workspace("phase-result-retention");
    let mut run = start(&mut workspace);
    let owner = workspace.advancement_owner();
    owner.with_advancement(|phase| {
    let mut members = inputs();
    members[1].1 = domain::WorthQueryWorkflowValue::Text("8:3".into());
    let stages = run.canonical_parallel_stages(members).unwrap();
    run.validate_parallel_runtime_authority(&workspace).unwrap();
    let frontier = run.prepare_parallel_frontier(&stages).unwrap();
    run.admit_parallel_frontier(frontier).unwrap();
    let computed = run.prepare_frontier_computation(stages).compute(
        phase.execution_request_for(&owner).unwrap(),
    );
    assert_eq!(
        _probe.take_computes(),
        phased_workflow::MEMBERS
            .iter()
            .map(|stage| (stage.to_string(), 17))
            .collect::<Vec<_>>()
    );
    let denial = computed
        .apply(&phase, &mut run, &mut workspace, None)
        .err()
        .unwrap();
    assert!(
        matches!(denial.kind(), domain::WorthQueryWorkflowAdvanceDenialKind::StageExecutor { detail, .. } if detail == "apply failure")
    );
    assert_eq!(
        run.receipts()
            .iter()
            .map(|receipt| receipt.stage_identity())
            .collect::<Vec<_>>(),
        ["start", "left"]
    );
    assert_eq!(effects(&run).len(), 1);
    assert!(denial.executed_effects().is_empty());
    assert_eq!(run.counters().stage_executor_contacts, 3);
    }).unwrap();
}

#[test]
fn missing_predecessor_authority_remains_a_typed_denial() {
    let _probe = phased_workflow::fold_executor::ComputeProbe::new();
    for invalid_index in [false, true] {
        let mut workspace = phased_workflow::phased_workspace("phase-missing-predecessor");
        let mut run = start(&mut workspace);
        if invalid_index {
            run.receipt_index.insert("start".into(), usize::MAX);
        } else {
            run.receipt_index.remove("start");
        }
        let mut ordinary_workspace = phased_workflow::phased_workspace("phase-missing-predecessor");
        let mut ordinary = start(&mut ordinary_workspace);
        if invalid_index {
            ordinary.receipt_index.insert("start".into(), usize::MAX);
        } else {
            ordinary.receipt_index.remove("start");
        }
        let ordinary_owner = ordinary_workspace.advancement_owner();
        let ordinary_denial = ordinary_owner
            .with_advancement(|phase| {
                ordinary
                    .advance_once(
                        &phase,
                        "left",
                        domain::WorthQueryWorkflowValue::Text("7:0".into()),
                        &mut ordinary_workspace,
                    )
                    .err()
                    .unwrap()
            })
            .unwrap();
        let owner = workspace.advancement_owner();
        let denial = owner
            .with_advancement(|phase| {
                run.prepare_frontier_computation(inputs())
                    .compute(phase.execution_request_for(&owner).unwrap())
                    .apply(&phase, &mut run, &mut workspace, None)
                    .err()
                    .unwrap()
            })
            .unwrap();
        assert!(
            matches!(denial.kind(), domain::WorthQueryWorkflowAdvanceDenialKind::PredecessorAuthorityMissing(stage) if stage == "start")
        );
        assert_eq!(denial.counters(), ordinary_denial.counters());
        assert_eq!(run.receipts().len(), 1);
        assert!(denial.executed_effects().is_empty());
    }
}
