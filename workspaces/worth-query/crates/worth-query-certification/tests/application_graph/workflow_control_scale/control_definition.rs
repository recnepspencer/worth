//! control definition.
use super::*;

pub(super) fn control_definition(
    occurrences: u16,
) -> AuthoredWorkflowDefinition<ReviewedDocumentWorkflow> {
    let nodes = occurrences * FRAGMENT_NODES + 1;
    let connections = occurrences * (FRAGMENT_CONNECTIONS + 1);
    let limits = ApplicationWorkflowDefinitionLimits::new(
        nodes,
        connections,
        connections,
        component_limits(occurrences),
        16 * 1024 * 1024,
    )
    .expect("finite control definition limits");
    let mut fragment =
        ApplicationWorkflowComponentBuilder::<ReviewedDocumentWorkflow>::new("control-fragment")
            .expect("component identity");
    let proposal = fragment
        .operation::<WorkflowDefinitionAuthoringOperation>("proposal", false)
        .expect("proposal operation");
    let condition = fragment
        .condition(
            "condition",
            "retained",
            ApplicationWorkflowConditionOperands::new()
                .query::<DocumentRetentionConditionQuery>("retained"),
        )
        .expect("typed condition");
    let retention = fragment
        .assessment::<DocumentRetentionQuery>("retention")
        .expect("resource assessment");
    let independent = fragment
        .assessment_for::<DocumentRetentionQuery>(
            "independent",
            ApplicationWorkflowSubjectSelector::related(),
        )
        .expect("related assessment");
    let join = fragment
        .evidence_join(
            "evidence",
            ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        )
        .expect("evidence join");
    let revise = fragment
        .operation::<WorkflowDefinitionAuthoringOperation>("revise", false)
        .expect("revision operation");
    let exit = fragment
        .operation::<WorkflowDefinitionAuthoringOperation>("exit", false)
        .expect("fragment exit");
    let checkpoint = fragment
        .operation::<WorkflowDefinitionAuthoringOperation>("checkpoint", false)
        .expect("fragment checkpoint");
    let last = fragment
        .operation::<WorkflowDefinitionAuthoringOperation>("last", false)
        .expect("fragment last operation");
    let retry = ApplicationWorkflowRetry::new(
        ApplicationWorkflowControlOutcome::Completed,
        "control-revision",
        2,
    )
    .expect("finite retry");
    fragment
        .control(
            &proposal,
            ApplicationWorkflowControlOutcome::Completed,
            &condition,
        )
        .unwrap()
        .control(
            &condition,
            ApplicationWorkflowControlOutcome::ConditionSatisfied,
            &retention,
        )
        .unwrap()
        .control(
            &condition,
            ApplicationWorkflowControlOutcome::ConditionUnsatisfied,
            &exit,
        )
        .unwrap()
        .control(
            &retention,
            ApplicationWorkflowControlOutcome::Completed,
            &independent,
        )
        .unwrap()
        .control(
            &independent,
            ApplicationWorkflowControlOutcome::Completed,
            &join,
        )
        .unwrap()
        .control(
            &join,
            ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            &exit,
        )
        .unwrap()
        .control(
            &join,
            ApplicationWorkflowControlOutcome::EvidenceFailed,
            &revise,
        )
        .unwrap()
        .retry(&revise, retry, &proposal)
        .unwrap()
        .control(
            &revise,
            ApplicationWorkflowControlOutcome::RetryExhausted,
            &exit,
        )
        .unwrap()
        .control(
            &exit,
            ApplicationWorkflowControlOutcome::Completed,
            &checkpoint,
        )
        .unwrap()
        .control(
            &checkpoint,
            ApplicationWorkflowControlOutcome::Completed,
            &last,
        )
        .unwrap()
        .condition_subject(&proposal, &condition)
        .unwrap()
        .proposal_for_assessment(&proposal, &retention)
        .unwrap()
        .proposal_for_assessment(&proposal, &independent)
        .unwrap()
        .assessment_evidence(&retention, &join)
        .unwrap()
        .assessment_evidence(&independent, &join)
        .unwrap();
    let input = fragment.input_port("input", &proposal).unwrap();
    let output = fragment.output_port("output", &last).unwrap();
    let fragment = fragment.finish().expect("control fragment closes");
    let mut definition = ApplicationWorkflowDefinitionBuilder::<ReviewedDocumentWorkflow>::new(
        "repeated-control-fragment",
        limits,
    )
    .expect("definition identity");
    let mut previous = None;
    for index in 0..occurrences {
        let expanded = definition
            .expand_component(&format!("control-{index:04}"), &fragment)
            .expect("bounded control expansion");
        let entry = expanded.input(&input).expect("exported input binds");
        let exit = expanded.output(&output).expect("exported output binds");
        if let Some(previous) = previous {
            definition.control(
                &previous,
                ApplicationWorkflowControlOutcome::Completed,
                &entry,
            );
        } else {
            definition.start(&entry);
        }
        previous = Some(exit);
    }
    let terminal = definition.terminal("done").expect("terminal node");
    definition.control(
        &previous.expect("nonempty control sequence"),
        ApplicationWorkflowControlOutcome::Completed,
        &terminal,
    );
    definition.finish().expect("control definition closes")
}
