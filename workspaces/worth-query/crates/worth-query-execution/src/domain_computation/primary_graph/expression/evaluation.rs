use worth_foundational::expression_api::{
    expressions, ExpressionDenial, ExpressionFunctionCatalog, ExpressionInputs, ExpressionSchema,
    ExpressionType, ExpressionValue,
};
use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowCondition, MIGRATED_WORKFLOW_CONDITION_OPERAND,
};

use crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowConditionExpression;

/// Evaluates a published condition over its operand values, given in name
/// order with the types their installed queries publish. The draft is
/// readmitted as an untrusted artifact under the declaration profile; a
/// denial at any step decides nothing.
pub(in crate::domain_computation::primary_graph) fn evaluate_condition<'operand>(
    expression: &CompiledWorkflowConditionExpression,
    operands: Vec<(&'operand str, &'operand ExpressionType, ExpressionValue)>,
) -> Result<bool, ExpressionDenial> {
    let profile = ApplicationWorkflowCondition::PROFILE;
    let reader = expressions().reading_within(profile);
    let draft = match expression {
        CompiledWorkflowConditionExpression::Migrated => {
            reader.parse(MIGRATED_WORKFLOW_CONDITION_OPERAND)?
        }
        CompiledWorkflowConditionExpression::Draft(draft) => reader.decode(draft)?,
    };
    let schema = operands
        .iter()
        .try_fold(ExpressionSchema::builder(), |schema, (name, ty, _)| {
            schema.operand(name, (*ty).clone())
        })?
        .build();
    let catalog = ExpressionFunctionCatalog::builder(&schema, profile).build();
    let admitted = draft.admit_as(&schema, &catalog, profile, &ExpressionType::Bool)?;
    let inputs = operands
        .into_iter()
        .try_fold(
            ExpressionInputs::builder(&schema),
            |inputs, (name, _, value)| inputs.bind(name, value),
        )?
        .build();
    let value = admitted
        .compile()
        .evaluate(&inputs, &profile)
        .into_result()?;
    Ok(value
        .as_bool()
        .expect("a condition admitted as Bool evaluates to a Bool"))
}
