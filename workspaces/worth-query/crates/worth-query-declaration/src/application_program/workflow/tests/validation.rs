use super::*;

#[test]
fn authored_effect_ceiling_is_enforced_before_graph_execution(
) -> Result<(), Box<dyn std::error::Error>> {
    let limits = ApplicationWorkflowDefinitionLimits::new(
        4,
        4,
        1,
        ApplicationWorkflowComponentLimits::new(4, 2, 8, 8, 8).unwrap(),
        4096,
    )
    .expect("the focused limits are nonzero");
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new("effect-ceiling", limits)?;
    let first = builder.operation::<ProposeChange>("first", false)?;
    let second = builder.operation::<ApplyChange>("second", false)?;
    let terminal = builder.terminal("terminal")?;
    builder
        .start(&first)
        .control(
            &first,
            ApplicationWorkflowControlOutcome::Completed,
            &second,
        )
        .control(
            &second,
            ApplicationWorkflowControlOutcome::Completed,
            &terminal,
        );
    let denial = match builder.finish()?.validate() {
        Ok(_) => panic!("two effect nodes exceeded the authored ceiling of one"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        ApplicationWorkflowValidationDenialKind::EffectLimitExceeded
    );
    Ok(())
}

fn denial(
    author: impl FnOnce(
        &mut ApplicationWorkflowDefinitionBuilder<ReviewedGeometry>,
    ) -> Result<(), ApplicationWorkflowAuthoringDenial>,
) -> ApplicationWorkflowValidationDenialKind {
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new("control-denial", limits())
            .expect("the fixture identity is valid");
    author(&mut builder).expect("the malformed graph remains authorable data");
    match builder
        .finish()
        .expect("the malformed graph remains a complete draft")
        .validate()
    {
        Ok(_) => panic!("the validator accepted incomplete control meaning"),
        Err(denial) => denial.kind(),
    }
}

#[test]
fn control_outcome_denials_are_distinct() {
    assert_eq!(
        denial(|builder| {
            let start = builder.operation::<ProposeChange>("start", false)?;
            let terminal = builder.terminal("terminal")?;
            builder.start(&start);
            let _ = terminal;
            Ok(())
        }),
        ApplicationWorkflowValidationDenialKind::MissingControlOutcome,
    );

    assert_eq!(
        denial(|builder| {
            let start = builder.operation::<ProposeChange>("start", false)?;
            let first = builder.terminal("first")?;
            let second = builder.terminal("second")?;
            builder
                .start(&start)
                .control(&start, ApplicationWorkflowControlOutcome::Completed, &first)
                .control(
                    &start,
                    ApplicationWorkflowControlOutcome::Completed,
                    &second,
                );
            Ok(())
        }),
        ApplicationWorkflowValidationDenialKind::AmbiguousControlOutcome,
    );

    assert_eq!(
        denial(|builder| {
            let start = builder.operation::<ProposeChange>("start", false)?;
            let assessment = builder.assessment::<StructuralAssessment>("assessment")?;
            let completed = builder.terminal("completed")?;
            let unexpected = builder.terminal("unexpected")?;
            builder
                .start(&start)
                .control(
                    &start,
                    ApplicationWorkflowControlOutcome::Completed,
                    &assessment,
                )
                .control(
                    &assessment,
                    ApplicationWorkflowControlOutcome::Completed,
                    &completed,
                )
                .control(
                    &assessment,
                    ApplicationWorkflowControlOutcome::Approved,
                    &unexpected,
                )
                .proposal_for_assessment(&start, &assessment);
            Ok(())
        }),
        ApplicationWorkflowValidationDenialKind::UnexpectedControlOutcome,
    );

    assert_eq!(
        denial(|builder| {
            let start = builder.operation::<ProposeChange>("start", false)?;
            let terminal = builder.terminal("terminal")?;
            let after = builder.terminal("after")?;
            builder
                .start(&start)
                .control(
                    &start,
                    ApplicationWorkflowControlOutcome::Completed,
                    &terminal,
                )
                .control(
                    &terminal,
                    ApplicationWorkflowControlOutcome::Completed,
                    &after,
                );
            Ok(())
        }),
        ApplicationWorkflowValidationDenialKind::TerminalHasSuccessor,
    );
}

#[test]
fn data_producer_must_dominate_its_consumer() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new("backward-data", limits())?;
    let first = builder.operation::<ProposeChange>("first", false)?;
    let second = builder.operation::<ApplyChange>("second", false)?;
    let terminal = builder.terminal("terminal")?;
    builder
        .start(&first)
        .control(
            &first,
            ApplicationWorkflowControlOutcome::Completed,
            &second,
        )
        .control(
            &second,
            ApplicationWorkflowControlOutcome::Completed,
            &terminal,
        )
        .operation_input(&second, &first);
    let denial = match builder.finish()?.validate() {
        Ok(_) => panic!("a later producer cannot supply an earlier consumer"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        ApplicationWorkflowValidationDenialKind::UnavailableDataFlow
    );
    Ok(())
}

#[test]
fn operation_cannot_supply_its_own_input() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new("self-data", limits())?;
    let operation = builder.operation::<ProposeChange>("operation", false)?;
    let terminal = builder.terminal("terminal")?;
    builder
        .start(&operation)
        .control(
            &operation,
            ApplicationWorkflowControlOutcome::Completed,
            &terminal,
        )
        .operation_input(&operation, &operation);
    let denial = match builder.finish()?.validate() {
        Ok(_) => panic!("an operation cannot produce data before its own execution"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        ApplicationWorkflowValidationDenialKind::UnavailableDataFlow
    );
    Ok(())
}

#[test]
fn operation_input_requires_the_exact_portable_input_type() -> Result<(), Box<dyn std::error::Error>>
{
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(
        "incompatible-operation-input",
        limits(),
    )?;
    let source = builder.operation::<CollisionOperationAb>("source", false)?;
    let target = builder.operation::<CollisionOperationA>("target", false)?;
    let terminal = builder.terminal("terminal")?;
    builder
        .start(&source)
        .control(
            &source,
            ApplicationWorkflowControlOutcome::Completed,
            &target,
        )
        .control(
            &target,
            ApplicationWorkflowControlOutcome::Completed,
            &terminal,
        )
        .operation_input(&source, &target);
    let denial = match builder.finish()?.validate() {
        Ok(_) => panic!("an operation input cannot cross portable input types"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        ApplicationWorkflowValidationDenialKind::IncompatibleOperationInput
    );
    Ok(())
}

#[test]
fn data_cannot_cross_a_mutually_exclusive_control_arm() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(
        "conditional-data",
        limits(),
    )?;
    let propose = builder.operation::<ProposeChange>("propose", false)?;
    let structural = builder.assessment::<StructuralAssessment>("structural")?;
    let manufacturability =
        builder.assessment::<ManufacturabilityAssessment>("manufacturability")?;
    let evidence = builder.evidence_join(
        "evidence",
        ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
    )?;
    let approval = builder.approval::<GeometryApprover>("approval")?;
    let approved = builder.operation::<ApplyChange>("approved", true)?;
    let rejected = builder.operation::<ApplyChange>("rejected", false)?;
    let terminal = builder.terminal("terminal")?;
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &structural,
        )
        .control(
            &structural,
            ApplicationWorkflowControlOutcome::Completed,
            &manufacturability,
        )
        .control(
            &manufacturability,
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
            &rejected,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Approved,
            &approved,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Rejected,
            &rejected,
        )
        .control(
            &approved,
            ApplicationWorkflowControlOutcome::Completed,
            &terminal,
        )
        .control(
            &rejected,
            ApplicationWorkflowControlOutcome::Completed,
            &terminal,
        )
        .proposal_for_assessment(&propose, &structural)
        .proposal_for_assessment(&propose, &manufacturability)
        .assessment_evidence(&structural, &evidence)
        .assessment_evidence(&manufacturability, &evidence)
        .proposal_for_approval(&propose, &approval)
        .joined_evidence(&evidence, &approval)
        .approval_authority(&approval, &approved)
        .operation_input(&approved, &rejected);
    let denial = match builder.finish()?.validate() {
        Ok(_) => panic!("one conditional arm cannot feed another"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        ApplicationWorkflowValidationDenialKind::UnavailableDataFlow
    );
    Ok(())
}

#[test]
fn guarded_operation_requires_exact_approval_source() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new("denied", limits())?;
    let propose = builder.operation::<ProposeChange>("propose", false)?;
    let apply = builder.operation::<ApplyChange>("apply", true)?;
    let terminal = builder.terminal("terminal")?;
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &apply,
        )
        .control(
            &apply,
            ApplicationWorkflowControlOutcome::Completed,
            &terminal,
        )
        .operation_input(&propose, &apply);
    let denial = match builder.finish()?.validate() {
        Ok(_) => panic!("authority is absent"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        ApplicationWorkflowValidationDenialKind::MissingWorkflowAuthority
    );
    Ok(())
}
