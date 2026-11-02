//! Public cause vocabulary omits positions already consumed by native fact lookup.
use super::RebaseVerificationFailure as Failure;
use crate::domain_computation::primary_graph::application_attempt::IndexedSelectionReobserveDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RebaseVerificationReason {
    ManagedIndex(crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexWorkDenial),
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    NativeRevisionUnavailable,
    NativeFactRevisionUnavailable,
    IndexedSelectionDenied(IndexedSelectionReobserveDenial),
    IndexedSelectionFactDenied(IndexedSelectionReobserveDenial),
    UnsupportedDecisionFact,
    NativePayloadStorage { class: worth_execution::ExecutionAllocationDenialClass, requested: Option<u64> },
    AdmissionDenied(worth_relational::facade::mvcc::CompanionPreflightStop),
}

impl Failure {
    pub(in crate::domain_computation::primary_graph) fn into_diagnostic(
        self,
    ) -> RebaseVerificationReason {
        use RebaseVerificationReason as Cause;
        match self {
            Self::ManagedIndex(cause) => Cause::ManagedIndex(cause),
            Self::Execution(cause) => Cause::Execution(cause),
            Self::NativeRevisionUnavailable => Cause::NativeRevisionUnavailable,
            Self::NativeFactRevisionUnavailable(_) => Cause::NativeFactRevisionUnavailable,
            Self::IndexedSelectionDenied(cause) => Cause::IndexedSelectionDenied(cause),
            Self::IndexedSelectionFactDenied(_, cause) => Cause::IndexedSelectionFactDenied(cause),
            Self::UnsupportedDecisionFact => Cause::UnsupportedDecisionFact,
            Self::NativePayloadStorage { class, requested } => {
                Cause::NativePayloadStorage { class, requested }
            }
            Self::AdmissionDenied(cause) => Cause::AdmissionDenied(cause),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_rebase_native_fact_cause_keeps_its_stage() {
        assert_eq!(
            Failure::NativeFactRevisionUnavailable(37).into_diagnostic(),
            RebaseVerificationReason::NativeFactRevisionUnavailable
        );
    }
    #[test]
    fn public_rebase_indexed_fact_cause_keeps_stage_and_exact_denial() {
        for denial in [
            IndexedSelectionReobserveDenial::Overflow,
            IndexedSelectionReobserveDenial::WorkBudgetExceeded,
        ] {
            assert_eq!(
                Failure::IndexedSelectionFactDenied(53, denial).into_diagnostic(),
                RebaseVerificationReason::IndexedSelectionFactDenied(denial)
            );
        }
    }
}
