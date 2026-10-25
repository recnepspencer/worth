//! Retained heaps behind an attempt denial, including nested authority causes.
use super::{AttemptDenialCause, WorthQueryApplicationAttemptDenial};
use worth_execution::ChargedBytes;
impl ChargedBytes for WorthQueryApplicationAttemptDenial {
    fn additional_charged_bytes(&self) -> u64 {
        (self.subject.capacity() as u64).saturating_add(
            self.cause()
                .map_or(0, ChargedBytes::additional_charged_bytes),
        )
    }
}

impl ChargedBytes for AttemptDenialCause {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            AttemptDenialCause::IndexedLookup(_) => std::mem::size_of::<
                worth_relational::facade::indexes::BoundedEntityFieldLookupDenial,
            >() as u64,
            AttemptDenialCause::OutputObservation(_) => std::mem::size_of::<crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop>() as u64,
            AttemptDenialCause::None | AttemptDenialCause::RelationalExecution(_) => 0,
            // Native query stops contain only inline reports, identities and enums.
            AttemptDenialCause::NativeQueryRead(_) => std::mem::size_of::<
                worth_relational::facade::runtime::QueryReadExecutionStop,
            >() as u64,
            AttemptDenialCause::RetainedSource(_) => {
                std::mem::size_of::<super::super::retained_decision_facts::StoreDenial>() as u64
            }
            AttemptDenialCause::Allocation(_) => {
                std::mem::size_of::<worth_execution::ExecutionAllocationDenial>() as u64
            }
            AttemptDenialCause::RequestAuthority(cause) => cause.additional_charged_bytes(),
            AttemptDenialCause::Expression(cause) => {
                use worth_foundational::expression_api::ExpressionDenialDetail as Detail;
                let text = match cause.detail() {
                    Detail::UnknownBinding(text)
                    | Detail::AmbiguousBinding(text)
                    | Detail::MissingOperand(text) => text.capacity() as u64,
                    Detail::TypeMismatch { expected, found } => {
                        (expected.capacity() as u64).saturating_add(found.capacity() as u64)
                    }
                    Detail::FunctionContractMismatch { function, .. } => function.capacity() as u64,
                    Detail::Syntax(_)
                    | Detail::UnsupportedVersion { .. }
                    | Detail::UnsupportedFeature(_)
                    | Detail::TypeRequired(_)
                    | Detail::InvalidValue(_)
                    | Detail::AbsentValue
                    | Detail::Bounds(_)
                    | Detail::ArithmeticOverflow(_)
                    | Detail::ArithmeticDomain(_)
                    | Detail::DivisionByZero
                    | Detail::ResourceExceeded { .. } => 0,
                };
                (std::mem::size_of::<worth_foundational::expression_api::ExpressionDenial>() as u64)
                    .saturating_add(text)
            }
        }
    }
}
