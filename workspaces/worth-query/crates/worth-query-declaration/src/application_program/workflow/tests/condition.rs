use super::*;

fn condition_definition<Query>(
) -> Result<ValidatedWorkflowDefinition<ReviewedGeometry>, Box<dyn std::error::Error>>
where
    Query: ApplicationQueryMarkerIdentity<TestSchema> + 'static,
    Query::ResultBinding: ApplicationStructuredValueBinding<Value = bool>,
{
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(
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
    let baseline = condition_definition::<StructuralCondition>()?;
    let alternate = condition_definition::<ManufacturabilityCondition>()?;
    assert_ne!(baseline.content_identity(), alternate.content_identity());
    Ok(())
}

#[test]
fn command_condition_is_canonical_with_builder_authoring() -> Result<(), Box<dyn std::error::Error>>
{
    use ApplicationWorkflowAuthoringCommand as Command;
    use ApplicationWorkflowControlOutcome::{Completed, ConditionSatisfied, ConditionUnsatisfied};
    let command = ApplicationWorkflowCommandAdapter::author::<ReviewedGeometry>(
        "conditional-assessment",
        limits(),
        vec![
            Command::operation::<ReviewedGeometry, ProposeChange>("propose", false)?,
            Command::condition::<ReviewedGeometry, StructuralCondition>("condition")?,
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
    let builder = condition_definition::<StructuralCondition>()?;
    assert_eq!(builder.content_identity(), command.content_identity());
    assert_eq!(builder.nodes(), command.nodes());
    assert_eq!(builder.connections(), command.connections());
    Ok(())
}

#[test]
fn condition_requires_exactly_one_subject() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(
        "missing-condition-subject",
        limits(),
    )?;
    let propose = builder.operation::<ProposeChange>("propose", false)?;
    let condition = builder.condition::<StructuralCondition>("condition")?;
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
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(
        "missing-condition-arm",
        limits(),
    )?;
    let propose = builder.operation::<ProposeChange>("propose", false)?;
    let condition = builder.condition::<StructuralCondition>("condition")?;
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
