mod request {
    pub(super) use worth_query::facade::consumer_kit::workflow_proof_execution_request as workflow_request;
}
use super::super::super::WorthQueryWorkflowRun;
use worth_query::facade::{domain, foundation, runtime};

#[path = "tests/phased_workflow.rs"]
mod phased_workflow;
use phased_workflow::{PhaseDomain, PhaseFamily, PhaseOperation};

#[path = "tests/differential.rs"]
mod differential;
#[path = "tests/result_retention.rs"]
mod result_retention;

#[path = "receipt_parity.rs"]
mod receipt_parity;

type Run = WorthQueryWorkflowRun<
    PhaseDomain,
    PhaseOperation,
    PhaseFamily,
    foundation::MutationPreparationLaneWitness,
>;

fn start(workspace: &mut runtime::WorthQueryWorkspace) -> Run {
    let installed = workspace.domain(PhaseDomain).unwrap();
    workspace
        .prepare_mutation_operating_world(workspace.current_world())
        .unwrap()
        .family(PhaseFamily)
        .bind(&installed, PhaseOperation)
        .unwrap()
        .admit_workflow_resources(
            phased_workflow::world::execution_resource_request(),
            workspace,
        )
        .unwrap()
        .start_workflow(workspace)
        .unwrap()
        .advance(
            "start",
            domain::WorthQueryWorkflowValue::NotRequired,
            workspace,
            worth_query::facade::runtime::ExecutionRequest::serial(&request::workflow_request()),
        )
        .unwrap()
}

fn inputs() -> Vec<(String, domain::WorthQueryWorkflowValue)> {
    phased_workflow::MEMBERS
        .into_iter()
        .enumerate()
        .map(|(index, stage)| {
            (
                stage.into(),
                domain::WorthQueryWorkflowValue::Text(format!("{}:0", 7 + index)),
            )
        })
        .collect()
}

fn effects(run: &Run) -> Vec<String> {
    run.receipts()
        .iter()
        .flat_map(|receipt| {
            receipt.effect_evidence().iter().map(|effect| {
                effect
                    .mutation_receipt()
                    .unwrap()
                    .declared_aspect_value_digest()
                    .unwrap()
                    .to_owned()
            })
        })
        .collect()
}

#[test]
fn earlier_apply_failure_wins_over_later_preparation_or_compute_failure() {
    let _probe = phased_workflow::fold_executor::ComputeProbe::new();
    for later_mode in [1, 2] {
        let mut reference = phased_workflow::reference_workspace("phase-first-failure");
        let mut workspace = phased_workflow::phased_workspace("phase-first-failure");
        let mut members = inputs();
        members[0].1 = domain::WorthQueryWorkflowValue::Text("7:4".into());
        members[2].1 = domain::WorthQueryWorkflowValue::Text(format!("9:{later_mode}"));
        let mut reference_members = inputs();
        reference_members[0].1 = domain::WorthQueryWorkflowValue::Text("7:4".into());
        reference_members[2].1 = domain::WorthQueryWorkflowValue::Text(format!("9:{later_mode}"));
        let reference_outcome = match member_by_member_frontier(
            start(&mut reference),
            reference_members,
            &mut reference,
        ) {
            worth_proof::TransitionOutcome::Failed(denial) => denial,
            _ => panic!("expected member-by-member reference failure"),
        };
        let actual = match start(&mut workspace).advance_admitted_frontier(
            members,
            &mut workspace,
            worth_query::facade::runtime::ExecutionRequest::serial(&request::workflow_request()),
        ) {
            worth_proof::TransitionOutcome::Failed(denial) => denial,
            _ => panic!("expected phased failure"),
        };
        assert_eq!(actual.kind(), reference_outcome.kind());
        assert_eq!(
            owner_counters(actual.counters()),
            owner_counters(reference_outcome.counters())
        );
        assert_eq!(
            actual.completed_stage_receipts().len(),
            reference_outcome.completed_stage_receipts().len()
        );
        assert_eq!(actual.executed_effects().len(), 1);
        assert!(actual.executed_effects()[0]
            .semantic_replay_eq(&reference_outcome.executed_effects()[0]));
    }
}
// Independent orchestration reference: each canonical member completes its
// one-member batch before the next is prepared. The subject prepares the whole
// frontier first. This is a member-by-member reference, not HEAD code.
fn member_by_member_frontier(
    mut run: Run,
    members: Vec<(String, domain::WorthQueryWorkflowValue)>,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> super::super::super::workflow_progression::WorthQueryWorkflowAdvanceOutcome<
    PhaseDomain,
    PhaseOperation,
    PhaseFamily,
    foundation::MutationPreparationLaneWitness,
> {
    let members = run.canonical_parallel_stages(members).unwrap();
    run.validate_parallel_runtime_authority(workspace).unwrap();
    let frontier = run.prepare_parallel_frontier(&members).unwrap();
    run.admit_parallel_frontier(frontier).unwrap();
    for (stage, input) in members {
        match run.advance_once(&stage, input, workspace, worth_query::facade::runtime::ExecutionRequest::serial(&request::workflow_request())) {
            Ok(super::super::super::workflow_progression_state::WorthQueryWorkflowAdvanceStep::Advanced) => (),
            Ok(_) => panic!("reference fixture never defers"),
            Err(denial) => return run.outcome_from_denial(denial),
        }
    }
    run.active_parallel_admission = None;
    worth_proof::TransitionOutcome::Success(run)
}

#[test]
fn a_frontier_member_cannot_supply_another_members_predecessor_receipt() {
    let _probe = phased_workflow::fold_executor::ComputeProbe::new();
    let mut workspace = phased_workflow::phased_workspace("phase-fixed-predecessors");
    let run = start(&mut workspace);
    let outcome = run.advance_admitted_frontier(
        [
            (
                "left".into(),
                domain::WorthQueryWorkflowValue::Text("7:0".into()),
            ),
            (
                "publish".into(),
                domain::WorthQueryWorkflowValue::Text("join".into()),
            ),
        ],
        &mut workspace,
        worth_query::facade::runtime::ExecutionRequest::serial(&request::workflow_request()),
    );
    let denial = match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        _ => panic!("same-frontier dependencies must be refused before preparation"),
    };
    assert!(
        matches!(denial.kind(), domain::WorthQueryWorkflowAdvanceDenialKind::PredecessorIncomplete(stage) if stage == "left")
    );
    assert_eq!(denial.counters().stage_executor_contacts, 1);
    assert_eq!(denial.completed_stage_receipts().len(), 1);
    assert!(denial.executed_effects().is_empty());
}

#[test]
fn computed_results_cannot_cross_stage_or_frontier_identities() {
    let _probe = phased_workflow::fold_executor::ComputeProbe::new();
    for foreign_frontier in [false, true] {
        let mut workspace = phased_workflow::phased_workspace("phase-binding");
        let mut run = start(&mut workspace);
        let mut computed = run.prepare_frontier_computation(inputs()).compute(
            worth_execution::ExecutionRequest::serial(&request::workflow_request()),
        );
        if foreign_frontier {
            let mut other_workspace = phased_workflow::phased_workspace("phase-foreign-binding");
            let other = start(&mut other_workspace);
            let foreign = other.prepare_frontier_computation(inputs()).compute(
                worth_execution::ExecutionRequest::serial(&request::workflow_request()),
            );
            for (member, foreign_member) in computed.prefix.iter_mut().zip(foreign.prefix) {
                member.computed = foreign_member.computed;
            }
        } else {
            let middle = computed.prefix.remove(1).computed;
            let left = std::mem::replace(&mut computed.prefix[0].computed, middle);
            computed.prefix.insert(
                1,
                super::ComputedWorkflowStage {
                    rank: 1,
                    input: domain::WorthQueryWorkflowValue::Text("8:0".into()),
                    computed: left,
                },
            );
        }
        let denial = computed
            .apply(&mut run, &mut workspace, None)
            .err()
            .unwrap();
        assert!(matches!(
            denial.kind(),
            domain::WorthQueryWorkflowAdvanceDenialKind::UndeclaredFailureClass(
                domain::WorthQueryOperationFailureClass::Indeterminate
            )
        ));
        assert_eq!(run.receipts().len(), 1);
        assert!(denial.executed_effects().is_empty());
    }
}

fn owner_counters(
    mut counters: domain::WorthQueryWorkflowRunCounters,
) -> domain::WorthQueryWorkflowRunCounters {
    // HEAD had no metered compute. Only the new billing field is compared by
    // the dispatch differential; every prior owner counter remains exact.
    counters.computation_charged_work = 0;
    counters
}
#[path = "tests/dispatch.rs"]
mod dispatch;
