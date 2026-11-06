//! Descriptive result of the same installed precondition used by native admission.
mod evaluation;
mod value;
pub(in crate::domain_computation::primary_graph) use evaluation::evaluate_requirement;
pub(in crate::domain_computation::primary_graph) use evaluation::evaluate_requirements;
pub(in crate::domain_computation::primary_graph) use value::field_value;

fn expression_denial(
    cause: worth_foundational::expression_api::ExpressionDenial,
) -> crate::domain_computation::primary_graph::HandlerExecutionDenial {
    crate::domain_computation::primary_graph::HandlerExecutionDenial::new(crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial::body_requirement_expression("operation body requirement", cause))
}
