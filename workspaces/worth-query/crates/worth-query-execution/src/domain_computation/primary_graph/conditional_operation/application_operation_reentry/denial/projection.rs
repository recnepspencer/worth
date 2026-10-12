//! Projection admission and its work boundary share conditional execution stops.
use super::WorthQueryTemporalReentryDenial;
use crate::domain_computation::primary_graph::{WorthQueryOperationProjectionDenial as Failure, WorthQueryOperationProjectionDenialKind as Kind};
use crate::domain_computation::primary_graph::conditional_operation::application_operation_reentry::WorthQueryTemporalAdmissionTerminalFailure as Terminal;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
impl WorthQueryTemporalReentryDenial {
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn from_projection(
        denial: Failure,
    ) -> Self {
        if let Some(stop) = denial.invariant_denial().and_then(Self::retention_stop) {
            return stop;
        }
        match denial.kind() {
            Kind::Authorization(kind) => Self::authorization_kind(kind, |kind| {
                Terminal::Projection(Kind::Authorization(kind))
            }),
            Kind::InvariantAdmission(kind) => Self::invariant_kind(kind, |kind| {
                Terminal::Projection(Kind::InvariantAdmission(kind))
            }),
            Kind::WorkBudgetExceeded => Self::from_native_stop(Native::WorkExhausted),
        }
    }
}
