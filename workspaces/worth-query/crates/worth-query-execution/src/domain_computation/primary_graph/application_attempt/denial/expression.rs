//! Expression-language refusals keep their original denial as the attempt cause.
use super::{
    AttemptDenialCause, WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

impl WorthQueryApplicationAttemptDenial {
    pub(in crate::domain_computation::primary_graph) fn condition_expression(
        subject: impl Into<String>,
        denial: worth_foundational::expression_api::ExpressionDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationAttemptDenialKind::WorkflowConditionExpressionDenied,
            subject: subject.into(),
            cause: AttemptDenialCause::Expression(Box::new(denial)),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn body_requirement_expression(
        subject: impl Into<String>,
        denial: worth_foundational::expression_api::ExpressionDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationAttemptDenialKind::OperationBodyRequirementExpressionDenied,
            subject: subject.into(),
            cause: AttemptDenialCause::Expression(Box::new(denial)),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn body_value_expression(
        subject: impl Into<String>,
        denial: worth_foundational::expression_api::ExpressionDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationAttemptDenialKind::OperationBodyValueExpressionDenied,
            subject: subject.into(),
            cause: AttemptDenialCause::Expression(Box::new(denial)),
        }
    }
}
