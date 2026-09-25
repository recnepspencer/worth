use super::*;

#[test]
fn bounded_retry_is_the_only_valid_back_edge() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new("bounded-retry", limits())?;
    let first = builder.operation::<ProposeChange>("first", false)?;
    let revise = builder.operation::<ApplyChange>("revise", false)?;
    let exhausted = builder.terminal("exhausted")?;
    builder
        .start(&first)
        .control(
            &first,
            ApplicationWorkflowControlOutcome::Completed,
            &revise,
        )
        .retry(
            &revise,
            ApplicationWorkflowRetry::new(
                ApplicationWorkflowControlOutcome::Completed,
                "proposal-revision",
                2,
            )
            .expect("the retry is bounded and has a reason"),
            &first,
        )
        .control(
            &revise,
            ApplicationWorkflowControlOutcome::RetryExhausted,
            &exhausted,
        );

    builder.finish()?.validate()?;
    Ok(())
}

#[test]
fn back_navigation_cannot_be_authored_as_a_control_result() -> Result<(), Box<dyn std::error::Error>>
{
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(
        "not-an-authored-back-edge",
        limits(),
    )?;
    let proposal = builder.operation::<ProposeChange>("proposal", false)?;
    let terminal = builder.terminal("completed")?;
    builder
        .start(&proposal)
        .control(
            &proposal,
            ApplicationWorkflowControlOutcome::Completed,
            &terminal,
        )
        .control(
            &proposal,
            ApplicationWorkflowControlOutcome::NavigatedBack,
            &terminal,
        );
    let denial = match builder.finish()?.validate() {
        Ok(_) => panic!("Back is an owner action, not an authored control result"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        ApplicationWorkflowValidationDenialKind::UnexpectedControlOutcome
    );
    Ok(())
}

#[test]
fn retry_requires_a_nonzero_bound_and_reason() {
    assert!(ApplicationWorkflowRetry::new(
        ApplicationWorkflowControlOutcome::Completed,
        "proposal-revision",
        0,
    )
    .is_none());
    assert!(
        ApplicationWorkflowRetry::new(ApplicationWorkflowControlOutcome::Completed, "   ", 1,)
            .is_none()
    );
}

#[test]
fn retry_must_point_to_an_upstream_node() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new("forward-retry", limits())?;
    let first = builder.operation::<ProposeChange>("first", false)?;
    let later = builder.operation::<ApplyChange>("later", false)?;
    let exhausted = builder.terminal("exhausted")?;
    builder
        .start(&first)
        .retry(
            &first,
            ApplicationWorkflowRetry::new(
                ApplicationWorkflowControlOutcome::Completed,
                "not-a-back-edge",
                1,
            )
            .expect("the retry metadata is valid"),
            &later,
        )
        .control(
            &first,
            ApplicationWorkflowControlOutcome::RetryExhausted,
            &exhausted,
        )
        .control(
            &later,
            ApplicationWorkflowControlOutcome::Completed,
            &exhausted,
        );

    let denial = match builder.finish()?.validate() {
        Ok(_) => panic!("the retry edge is not backward"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        ApplicationWorkflowValidationDenialKind::ControlCycle
    );
    Ok(())
}

#[test]
fn retry_requires_an_exhaustion_successor() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(
        "missing-retry-exhaustion",
        limits(),
    )?;
    let first = builder.operation::<ProposeChange>("first", false)?;
    let revise = builder.operation::<ApplyChange>("revise", false)?;
    let terminal = builder.terminal("terminal")?;
    builder
        .start(&first)
        .control(
            &first,
            ApplicationWorkflowControlOutcome::Completed,
            &revise,
        )
        .retry(
            &revise,
            ApplicationWorkflowRetry::new(
                ApplicationWorkflowControlOutcome::Completed,
                "proposal-revision",
                1,
            )
            .expect("the retry metadata is valid"),
            &first,
        );
    let _ = terminal;

    let denial = match builder.finish()?.validate() {
        Ok(_) => panic!("retry exhaustion has no successor"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        ApplicationWorkflowValidationDenialKind::MissingControlOutcome
    );
    Ok(())
}

#[test]
fn terminal_cannot_retry_into_the_control_graph() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new("terminal-retry", limits())?;
    let operation = builder.operation::<ProposeChange>("operation", false)?;
    let terminal = builder.terminal("terminal")?;
    builder
        .start(&operation)
        .control(
            &operation,
            ApplicationWorkflowControlOutcome::Completed,
            &terminal,
        )
        .retry(
            &terminal,
            ApplicationWorkflowRetry::new(
                ApplicationWorkflowControlOutcome::Completed,
                "invalid-terminal-retry",
                1,
            )
            .unwrap(),
            &operation,
        );

    let denial = match builder.finish()?.validate() {
        Ok(_) => panic!("a terminal retry must not validate"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        ApplicationWorkflowValidationDenialKind::TerminalHasSuccessor
    );
    Ok(())
}
