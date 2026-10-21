//! Content admission keeps actual request controls out of producer failure.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    retained_decision_facts::StoreDenial, WorthQueryProducerIdentityDenial as Identity,
};
use crate::domain_computation::primary_graph::application_contribution::{
    execution_denial::advancement_scope_stop, WorthQueryAdvancementDenial as Advance,
    WorthQueryManagedComputationInterruption as Interrupted,
};
use worth_execution::ExecutionAllocationDenialClass;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;

pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn identity_failed(
    subject: &str,
    error: Identity,
) -> ProducerExecutionStop {
    let control = match &error {
        Identity::DependencyStorage(StoreDenial::RequestInterruption(interruption)) => {
            Some(match interruption {
                WorthQueryRequestInterruption::Cancelled => {
                    Advance::Interrupted(Interrupted::Cancelled)
                }
                WorthQueryRequestInterruption::DeadlineExceeded => {
                    Advance::Interrupted(Interrupted::DeadlineExceeded)
                }
            })
        }
        Identity::DependencyStorage(StoreDenial::Allocation(allocation)) => {
            match allocation.classify() {
                ExecutionAllocationDenialClass::Request(stop) => Some(advancement_scope_stop(stop)),
                ExecutionAllocationDenialClass::Physical(_) => None,
            }
        }
        _ => None,
    };
    match control {
        Some(Advance::Interrupted(interruption)) => request_admission_rejected(denial(
            match interruption {
                Interrupted::Cancelled => WorthQueryOutputDemandDenialKind::Cancelled,
                Interrupted::DeadlineExceeded => WorthQueryOutputDemandDenialKind::TimedOut,
            },
            subject.to_owned(),
        )),
        Some(stop) => denial(
            WorthQueryOutputDemandDenialKind::ExecutionRequest(stop),
            format!("{subject}: {error:?}"),
        )
        .into(),
        None if error == Identity::LineageLookupBudgetExceeded => denial(
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
            subject.to_owned(),
        )
        .into(),
        None => failed(subject, error).into(),
    }
}

#[cfg(test)]
mod tests;
