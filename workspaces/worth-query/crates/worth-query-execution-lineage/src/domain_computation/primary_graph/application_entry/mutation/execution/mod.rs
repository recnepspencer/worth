pub mod decision_query;
pub use decision_query::native_decision_cause;

use super::HandlerExecutionDenial;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryOperationProjectionDenial,
};

/// Refusal to run a mutation handler for an admitted operation.
#[derive(Debug)]
pub enum MutationHandlerExecutionDenial {
    DurableBindingAdmission(
        worth_query_installation::facade::WorthQueryApplicationOperationInstallationDenial,
    ),
    DurableStateAdmission(
        crate::domain_computation::execution_runtime::product_world::WorthQueryDurableStateAdmissionDenial,
    ),
    ForeignAdvancementPhase(
        crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase,
    ),
    Projection(WorthQueryOperationProjectionDenial),
    Attempt(WorthQueryApplicationAttemptDenial),
    Handler(HandlerExecutionDenial),
    WorkflowControl,
    InputNotAdmitted,
    HandlerNotInstalled,
}

impl std::fmt::Display for MutationHandlerExecutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DurableBindingAdmission(cause) => cause.fmt(formatter),
            Self::DurableStateAdmission(cause) => cause.fmt(formatter),
            Self::ForeignAdvancementPhase(_) => {
                formatter.write_str("the phase belongs to another installed runtime")
            }
            Self::Projection(denial) => denial.fmt(formatter),
            Self::Attempt(denial) => denial.fmt(formatter),
            Self::Handler(denial) => denial.fmt(formatter),
            Self::WorkflowControl => formatter.write_str(
                "the workflow kernel records this control binding; no mutation handler serves it",
            ),
            Self::InputNotAdmitted => {
                formatter.write_str("the mutation input was not admitted by this operation")
            }
            Self::HandlerNotInstalled => {
                formatter.write_str("no mutation handler is installed for this binding")
            }
        }
    }
}

impl std::error::Error for MutationHandlerExecutionDenial {}
