//! Evidence cardinality is nonempty, not an implicit two-check quorum.
use super::*;

fn singleton(
    policy: ApplicationWorkflowEvidenceJoinPolicy,
    edges: usize,
) -> Result<ValidatedWorkflowDefinition<ReviewedChange>, ApplicationWorkflowValidationDenial> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new(
        "single-check-approval",
        limits(),
    )
    .unwrap();
    let propose = builder
        .operation::<ProposeChange>("propose", false)
        .unwrap();
    let check = builder
        .assessment::<ConsistencyAssessment>("check")
        .unwrap();
    let evidence = builder.evidence_join("evidence", policy).unwrap();
    let approval = builder.approval::<ChangeApprover>("approval").unwrap();
    let apply = builder.operation_binding::<ApplyBinding>("apply").unwrap();
    let done = builder.terminal("done").unwrap();
    let refused = builder.terminal("refused").unwrap();
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &check,
        )
        .control(
            &check,
            ApplicationWorkflowControlOutcome::Completed,
            &evidence,
        )
        .control(
            &evidence,
            ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            &approval,
        )
        .control(
            &evidence,
            ApplicationWorkflowControlOutcome::EvidenceFailed,
            &refused,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Approved,
            &apply,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Rejected,
            &refused,
        )
        .control(&apply, ApplicationWorkflowControlOutcome::Completed, &done)
        .proposal_for_assessment(&propose, &check)
        .proposal_for_approval(&propose, &approval)
        .joined_evidence(&evidence, &approval)
        .approval_authority(&approval, &apply)
        .operation_input(&propose, &apply);
    for _ in 0..edges {
        builder.assessment_evidence(&check, &evidence);
    }
    builder.finish().unwrap().validate()
}

#[test]
fn evidence_accepts_one_real_check_but_not_empty_or_duplicate_inputs() {
    for policy in [
        ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        ApplicationWorkflowEvidenceJoinPolicy::AllRequiredCompleted,
    ] {
        assert!(singleton(policy, 1).is_ok());
        assert_eq!(
            singleton(policy, 0)
                .err()
                .expect("empty inventory refused")
                .kind(),
            ApplicationWorkflowValidationDenialKind::IncompleteEvidenceJoin
        );
        assert_eq!(
            singleton(policy, 2)
                .err()
                .expect("duplicate inventory refused")
                .kind(),
            ApplicationWorkflowValidationDenialKind::DuplicateConnection
        );
    }
}
