//! Moved whole from `domain_computation::primary_graph::operation_body::requirement::value`: the items below and every impl they own.
use worth_foundational::expression_api::ExpressionValue;

pub(in crate::domain_computation::primary_graph) struct PreparedRequirementOperand {
    pub(in crate::domain_computation::primary_graph) value: ExpressionValue,
    pub(in crate::domain_computation::primary_graph) _reservation:
        crate::domain_computation::request_execution::QueryMemoryReservation,
}
