use super::*;

#[test]
fn guarded_operations_require_exactly_one_input_after_authority_validation() {
    for inputs in [0, 2] {
        let refused = definition(inputs, true, true).validate().err().unwrap();
        assert_eq!(
            refused.kind(),
            ApplicationWorkflowValidationDenialKind::MissingOperationInput
        );
        assert_eq!(refused.subject(), "apply");
    }
    assert!(definition(1, true, true).validate().is_ok());
    assert_eq!(
        definition(0, true, false).validate().err().unwrap().kind(),
        ApplicationWorkflowValidationDenialKind::MissingWorkflowAuthority
    );
}

#[test]
fn unguarded_operation_input_cardinality_is_unconstrained() {
    for inputs in [0, 1, 2] {
        assert!(definition(inputs, false, false).validate().is_ok());
    }
}

fn definition(
    inputs: usize,
    guarded: bool,
    approval_edge: bool,
) -> AuthoredWorkflowDefinition<ReviewedChange> {
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new("input-cardinality", limits())
            .unwrap();
    let first = builder.operation::<ProposeChange>("first", false).unwrap();
    let second = builder.operation::<ProposeChange>("second", false).unwrap();
    let consistency = builder
        .assessment::<ConsistencyAssessment>("consistency")
        .unwrap();
    let compliance = builder
        .assessment::<ComplianceAssessment>("compliance")
        .unwrap();
    let evidence = builder
        .evidence_join(
            "evidence",
            ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        )
        .unwrap();
    let approval = builder.approval::<ChangeApprover>("approval").unwrap();
    let apply = builder.operation::<ApplyChange>("apply", guarded).unwrap();
    let done = builder.terminal("done").unwrap();
    let rejected = builder.terminal("rejected").unwrap();
    use ApplicationWorkflowControlOutcome::*;
    builder
        .start(&first)
        .control(&first, Completed, &second)
        .control(&second, Completed, &consistency)
        .control(&consistency, Completed, &compliance)
        .control(&compliance, Completed, &evidence)
        .control(&evidence, EvidenceSatisfied, &approval)
        .control(&evidence, EvidenceFailed, &rejected)
        .control(&approval, Approved, &apply)
        .control(&approval, Rejected, &rejected)
        .control(&apply, Completed, &done)
        .proposal_for_assessment(&second, &consistency)
        .proposal_for_assessment(&second, &compliance)
        .assessment_evidence(&consistency, &evidence)
        .assessment_evidence(&compliance, &evidence)
        .proposal_for_approval(&second, &approval)
        .joined_evidence(&evidence, &approval);
    if approval_edge {
        builder.approval_authority(&approval, &apply);
    }
    if inputs >= 1 {
        builder.operation_input(&first, &apply);
    }
    if inputs >= 2 {
        builder.operation_input(&second, &apply);
    }
    builder.finish().unwrap()
}
