//! One public reason, with its private post-effect registration requirement.
use super::RebaseVerificationReason as Reason;
use crate::domain_computation::primary_graph::application_attempt::IndexedSelectionReobserveDenial as Indexed;
use crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason as Registration;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct RebaseVerificationFailure {
    reason: Reason,
    registration: Registration,
}
impl RebaseVerificationFailure {
    pub(in crate::domain_computation::primary_graph) fn reason(&self) -> &Reason {
        &self.reason
    }
    pub(in crate::domain_computation::primary_graph) fn into_reason(self) -> Reason {
        self.reason
    }
    pub(in crate::domain_computation::primary_graph) fn into_registration_requirement(
        self,
    ) -> Registration {
        self.registration
    }
    pub(in crate::domain_computation::primary_graph) fn execution(
        cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
    ) -> Self {
        Self {
            reason: Reason::Execution(cause),
            registration: Registration::RegistrationIncomplete,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn managed_index(
        cause: crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexWorkDenial,
    ) -> Self {
        Self {
            reason: Reason::ManagedIndex(cause),
            registration: Registration::RegistrationIncomplete,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn native_unavailable() -> Self {
        Self {
            reason: Reason::NativeRevisionUnavailable,
            registration: Registration::NativeRevisionUnavailable,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn native_fact_unavailable(
        ordinal: usize,
    ) -> Self {
        Self {
            reason: Reason::NativeFactRevisionUnavailable,
            registration: Registration::NativeFactRevisionUnavailable(ordinal),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn indexed_denied(cause: Indexed) -> Self {
        Self {
            reason: Reason::IndexedSelectionDenied(cause),
            registration: Registration::IndexedSelectionDenied(cause),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn indexed_fact_denied(
        ordinal: usize,
        cause: Indexed,
    ) -> Self {
        Self {
            reason: Reason::IndexedSelectionFactDenied(cause),
            registration: Registration::IndexedSelectionFactDenied(ordinal, cause),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn unsupported() -> Self {
        Self {
            reason: Reason::UnsupportedDecisionFact,
            registration: Registration::UnsupportedFact,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn native_payload_storage(
        class: worth_execution::ExecutionAllocationDenialClass,
        requested: Option<u64>,
    ) -> Self {
        Self {
            reason: Reason::NativePayloadStorage { class, requested },
            registration: Registration::RegistrationIncomplete,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn admission_denied(
        cause: worth_relational::facade::mvcc::CompanionPreflightStop,
    ) -> Self {
        // A post-effect stop cannot undo the effect; its registration is incomplete.
        Self {
            reason: Reason::AdmissionDenied(cause),
            registration: Registration::RegistrationIncomplete,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn at_fact(self, ordinal: usize) -> Self {
        match self.reason {
            Reason::NativeRevisionUnavailable => Self::native_fact_unavailable(ordinal),
            Reason::IndexedSelectionDenied(cause) => Self::indexed_fact_denied(ordinal, cause),
            Reason::ManagedIndex(_)
            | Reason::Execution(_)
            | Reason::NativeFactRevisionUnavailable
            | Reason::IndexedSelectionFactDenied(_)
            | Reason::UnsupportedDecisionFact
            | Reason::NativePayloadStorage { .. }
            | Reason::AdmissionDenied(_) => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_fact_reason_and_registration_keep_the_actual_stage_and_position() {
        let reason = RebaseVerificationFailure::native_unavailable().at_fact(37);
        assert_eq!(reason.reason(), &Reason::NativeFactRevisionUnavailable);
        assert_eq!(
            Registration::from(reason),
            Registration::NativeFactRevisionUnavailable(37)
        );
        assert_eq!(
            RebaseVerificationFailure::native_unavailable().into_reason(),
            Reason::NativeRevisionUnavailable
        );
    }
    #[test]
    fn indexed_fact_reason_and_registration_keep_stage_position_and_denial() {
        for denial in [Indexed::Overflow, Indexed::WorkBudgetExceeded] {
            let reason = RebaseVerificationFailure::indexed_denied(denial).at_fact(53);
            assert_eq!(reason.reason(), &Reason::IndexedSelectionFactDenied(denial));
            assert_eq!(
                Registration::from(reason),
                Registration::IndexedSelectionFactDenied(53, denial)
            );
            assert_eq!(
                RebaseVerificationFailure::indexed_denied(denial).into_reason(),
                Reason::IndexedSelectionDenied(denial)
            );
        }
    }
}

mod court_doors_for_test;
