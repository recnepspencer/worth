//! Query support retention keeps the request owner's cause, not producer availability.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementDenial as Advance, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind as Attempt,
    WorthQueryManagedComputationInterruption as Interrupted,
};
pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn candidate_support_denied(
    cause: WorthQueryApplicationAttemptDenial,
) -> WorthQueryOutputDemandDenial {
    use WorthQueryOutputDemandDenialKind as Kind;
    let subject = format!("{}: {cause:?}", cause.subject());
    let kind = if let Some(interruption) = cause.source_retention_interruption() {
        match interruption {
            worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled => Kind::Cancelled,
            worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded => Kind::TimedOut,
        }
    } else if let Some(native) = cause.relational_execution_cause() {
        return WorthQueryOutputDemandDenial::new(
            crate::domain_computation::primary_graph::output_reuse::read_denial(native).kind(),
            subject,
        );
    } else if let Some(allocation) = cause.allocation_denial() {
        match allocation.classify() {
            worth_execution::ExecutionAllocationDenialClass::Request(stop) => {
                match crate::domain_computation::primary_graph::application_contribution::execution_denial::advancement_scope_stop(stop) {
                    Advance::Interrupted(Interrupted::Cancelled) => Kind::Cancelled,
                    Advance::Interrupted(Interrupted::DeadlineExceeded) => Kind::TimedOut,
                    stop => Kind::ExecutionRequest(stop),
                }
            }
            worth_execution::ExecutionAllocationDenialClass::Physical(_) => Kind::RetentionBudgetExceeded,
        }
    } else {
        match cause.kind() {
            Attempt::RetainedSourceStorageDenied
            | Attempt::CandidateCapacityExceeded
            | Attempt::AllocationDenied => Kind::RetentionBudgetExceeded,
            Attempt::ForeignPhase => Kind::RequiredSuccessorStateUnavailable,
            // Other attempt causes keep the existing execution owner's classification.
            _ => return super::execution_failed(
                &subject,
                crate::domain_computation::primary_graph::MutationHandlerExecutionDenial::Attempt(
                    cause,
                ),
            )
            .into_output_denial(),
        }
    };
    WorthQueryOutputDemandDenial::new(kind, subject)
}
