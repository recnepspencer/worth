use super::*;

fn condition_definition<Query>(
) -> Result<ValidatedWorkflowDefinition<ReviewedChange>, Box<dyn std::error::Error>>
where
    Query: ApplicationQueryMarkerIdentity<TestSchema> + 'static,
    Query::ResultBinding: ApplicationStructuredValueBinding<Value = bool>,
{
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new(
        "conditional-assessment",
        limits(),
    )?;
    let propose = builder.operation::<ProposeChange>("propose", false)?;
    let condition = builder.condition::<Query>("condition")?;
    let satisfied = builder.terminal("satisfied")?;
    let unsatisfied = builder.terminal("unsatisfied")?;
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &condition,
        )
        .control(
            &condition,
            ApplicationWorkflowControlOutcome::ConditionSatisfied,
            &satisfied,
        )
        .control(
            &condition,
            ApplicationWorkflowControlOutcome::ConditionUnsatisfied,
            &unsatisfied,
        )
        .condition_subject(&propose, &condition);
    Ok(builder.finish()?.validate()?)
}

#[test]
fn typed_condition_is_valid_and_query_changes_canonical_meaning(
) -> Result<(), Box<dyn std::error::Error>> {
    let baseline = condition_definition::<ConsistencyCondition>()?;
    let alternate = condition_definition::<ComplianceCondition>()?;
    assert_ne!(baseline.content_identity(), alternate.content_identity());
    Ok(())
}

#[test]
fn command_condition_is_canonical_with_builder_authoring() -> Result<(), Box<dyn std::error::Error>>
{
    use ApplicationWorkflowAuthoringCommand as Command;
    use ApplicationWorkflowControlOutcome::{Completed, ConditionSatisfied, ConditionUnsatisfied};
    let command = ApplicationWorkflowCommandAdapter::author::<ReviewedChange>(
        "conditional-assessment",
        limits(),
        vec![
            Command::operation::<ReviewedChange, ProposeChange>("propose", false)?,
            Command::condition::<ReviewedChange, ConsistencyCondition>("condition")?,
            Command::terminal("satisfied")?,
            Command::terminal("unsatisfied")?,
            Command::start("propose")?,
            Command::control("propose", Completed, "condition")?,
            Command::control("condition", ConditionSatisfied, "satisfied")?,
            Command::control("condition", ConditionUnsatisfied, "unsatisfied")?,
            Command::data(
                "propose",
                ApplicationWorkflowDataFlow::ConditionSubject,
                "condition",
            )?,
        ],
    )?
    .validate()?;
    let builder = condition_definition::<ConsistencyCondition>()?;
    assert_eq!(builder.content_identity(), command.content_identity());
    assert_eq!(builder.nodes(), command.nodes());
    assert_eq!(builder.connections(), command.connections());
    Ok(())
}

#[test]
fn condition_requires_exactly_one_subject() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new(
        "missing-condition-subject",
        limits(),
    )?;
    let propose = builder.operation::<ProposeChange>("propose", false)?;
    let condition = builder.condition::<ConsistencyCondition>("condition")?;
    let satisfied = builder.terminal("satisfied")?;
    let unsatisfied = builder.terminal("unsatisfied")?;
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &condition,
        )
        .control(
            &condition,
            ApplicationWorkflowControlOutcome::ConditionSatisfied,
            &satisfied,
        )
        .control(
            &condition,
            ApplicationWorkflowControlOutcome::ConditionUnsatisfied,
            &unsatisfied,
        );
    let denial = match builder.finish()?.validate() {
        Ok(_) => panic!("subject is absent"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        ApplicationWorkflowValidationDenialKind::MissingConditionSubject
    );
    Ok(())
}

#[test]
fn condition_requires_both_control_arms() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new(
        "missing-condition-arm",
        limits(),
    )?;
    let propose = builder.operation::<ProposeChange>("propose", false)?;
    let condition = builder.condition::<ConsistencyCondition>("condition")?;
    let satisfied = builder.terminal("satisfied")?;
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &condition,
        )
        .control(
            &condition,
            ApplicationWorkflowControlOutcome::ConditionSatisfied,
            &satisfied,
        )
        .condition_subject(&propose, &condition);
    let denial = match builder.finish()?.validate() {
        Ok(_) => panic!("false arm is absent"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        ApplicationWorkflowValidationDenialKind::MissingControlOutcome
    );
    Ok(())
}
