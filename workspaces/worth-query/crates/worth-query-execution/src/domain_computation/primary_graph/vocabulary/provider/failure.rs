//! Moved whole from `domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::failure`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::application_attempt::indexed_entity_selection::IndexedSelectionReobserveDenial as Indexed;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::mark_state::FullVerificationReason as Registration;
use crate::domain_computation::primary_graph::vocabulary::provider::diagnostic::RebaseVerificationReason as Reason;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebaseVerificationFailure {
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
        cause: crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial,
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

impl RebaseVerificationFailure {
    #[doc(hidden)]
    pub fn into_reason_for_test(self) -> Reason {
        Self::into_reason(self)
    }
}
