use worth_foundational::expression_api::{ExpressionDenialFamily, ExpressionType, IntegerType};

use super::*;

type Operands = ApplicationWorkflowConditionOperands<ReviewedChange>;

fn consistent() -> Operands {
    Operands::new().query::<ConsistencyCondition>("consistent")
}

fn reviewed() -> Operands {
    consistent().query::<ReviewCount>("reviews")
}

fn condition_definition(
    source: &str,
    operands: Operands,
) -> Result<ValidatedWorkflowDefinition<ReviewedChange>, Box<dyn std::error::Error>> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new(
        "conditional-assessment",
        limits(),
    )?;
    let propose = builder.operation::<ProposeChange>("propose", false)?;
    let condition = builder.condition("condition", source, operands)?;
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

fn condition_denial(source: &str, operands: Operands) -> ApplicationWorkflowConditionDenial {
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new("denied-condition", limits())
            .expect("the definition identity is valid");
    match builder.condition("condition", source, operands) {
        Err(ApplicationWorkflowAuthoringDenial::Condition(denial)) => denial,
        Err(other) => panic!("{source}: expected a condition denial, found {other}"),
        Ok(_) => panic!("{source} was admitted"),
    }
}

fn expression_family(denial: ApplicationWorkflowConditionDenial) -> ExpressionDenialFamily {
    match denial {
        ApplicationWorkflowConditionDenial::Expression(denial) => denial.family(),
        other => panic!("expected an expression denial, found {other}"),
    }
}

#[test]
fn query_source_and_operand_names_change_canonical_meaning(
) -> Result<(), Box<dyn std::error::Error>> {
    let baseline = condition_definition("consistent", consistent())?;
    let other_query = condition_definition(
        "consistent",
        Operands::new().query::<ComplianceCondition>("consistent"),
    )?;
    let other_source = condition_definition("!consistent", consistent())?;
    let other_name = condition_definition(
        "compliant",
        Operands::new().query::<ConsistencyCondition>("compliant"),
    )?;
    let identities = [
        baseline.content_identity(),
        other_query.content_identity(),
        other_source.content_identity(),
        other_name.content_identity(),
    ];
    for (index, identity) in identities.iter().enumerate() {
        assert!(!identities[index + 1..].contains(identity));
    }
    Ok(())
}

#[test]
fn source_spelling_and_operand_order_do_not_change_canonical_meaning(
) -> Result<(), Box<dyn std::error::Error>> {
    let baseline = condition_definition("consistent && (reviews ?? 0) >= 2", reviewed())?;
    let respelled = condition_definition(
        "consistent&&(reviews??0)>=2",
        Operands::new()
            .query::<ReviewCount>("reviews")
            .query::<ConsistencyCondition>("consistent"),
    )?;
    assert_eq!(baseline.content_identity(), respelled.content_identity());
    Ok(())
}

#[test]
fn condition_reads_several_typed_operands() -> Result<(), Box<dyn std::error::Error>> {
    let definition = condition_definition("consistent && (reviews ?? 0) >= 2", reviewed())?;
    let condition = definition
        .nodes()
        .iter()
        .find_map(|node| match node.kind() {
            ApplicationWorkflowNodeKind::Condition(condition) => Some(condition),
            _ => None,
        })
        .expect("the definition has a condition");
    let names: Vec<&str> = condition
        .operands()
        .iter()
        .map(|operand| operand.name())
        .collect();
    assert_eq!(names, ["consistent", "reviews"]);
    let reviews = condition.operand("reviews").expect("reviews is declared");
    assert_eq!(
        reviews.query().identifier(),
        "worth.query.tests.workflow.review-count.v1"
    );
    assert_eq!(
        reviews.query().expression_type(),
        &ExpressionType::option(ExpressionType::Integer(IntegerType::Int64))
    );
    assert!(condition.operand("absent").is_none());
    let decoded =
        ApplicationWorkflowCondition::decode(condition.draft(), condition.operands().to_vec())?;
    assert_eq!(&decoded, condition);
    Ok(())
}

#[test]
fn condition_admission_denies_before_the_graph_accepts_it() {
    assert_eq!(
        condition_denial("consistent", Operands::new()),
        ApplicationWorkflowConditionDenial::NoOperands
    );
    assert_eq!(
        expression_family(condition_denial("reviews ?? 0", reviewed())),
        ExpressionDenialFamily::TypeMismatch
    );
    assert_eq!(
        expression_family(condition_denial("consistent && approved", consistent())),
        ExpressionDenialFamily::UnknownBinding
    );
    assert_eq!(
        expression_family(condition_denial("consistent &&", consistent())),
        ExpressionDenialFamily::Syntax
    );
    assert_eq!(
        expression_family(condition_denial(
            "consistent",
            consistent().query::<ComplianceCondition>("consistent")
        )),
        ExpressionDenialFamily::AmbiguousBinding
    );
}

#[test]
fn migrated_condition_reads_its_query_as_the_decision() -> Result<(), Box<dyn std::error::Error>> {
    let migrated =
        ApplicationWorkflowCondition::migrated(ApplicationWorkflowConditionQuery::declared::<
            ReviewedChange,
            ConsistencyCondition,
        >())?;
    let authored = condition_definition(
        MIGRATED_WORKFLOW_CONDITION_OPERAND,
        Operands::new().query::<ConsistencyCondition>(MIGRATED_WORKFLOW_CONDITION_OPERAND),
    )?;
    assert!(authored.nodes().iter().any(
        |node| matches!(node.kind(), ApplicationWorkflowNodeKind::Condition(condition) if *condition == migrated)
    ));
    let non_bool =
        ApplicationWorkflowCondition::migrated(ApplicationWorkflowConditionQuery::declared::<
            ReviewedChange,
            ReviewCount,
        >())
        .expect_err("an Option<Int64> query is not a decision");
    assert_eq!(
        expression_family(non_bool),
        ExpressionDenialFamily::TypeMismatch
    );
    Ok(())
}

#[test]
fn command_condition_is_canonical_with_builder_authoring() -> Result<(), Box<dyn std::error::Error>>
{
    use ApplicationWorkflowAuthoringCommand as Command;
    use ApplicationWorkflowControlOutcome::{Completed, ConditionSatisfied, ConditionUnsatisfied};
    let source = "consistent && (reviews ?? 0) >= 2";
    let command = ApplicationWorkflowCommandAdapter::author::<ReviewedChange>(
        "conditional-assessment",
        limits(),
        vec![
            Command::operation::<ReviewedChange, ProposeChange>("propose", false)?,
            Command::condition("condition", source, reviewed())?,
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
    let builder = condition_definition(source, reviewed())?;
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
    let condition = builder.condition("condition", "consistent", consistent())?;
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
    let condition = builder.condition("condition", "consistent", consistent())?;
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
