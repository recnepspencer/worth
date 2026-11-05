//! Moved whole from `domain_computation::primary_graph::invariant_projection::consumed_output::publication_recovery`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::output_lineage::mark_state::FullVerificationReason;

pub(in crate::domain_computation::primary_graph) enum PublicationRecoveryStop {
    Reason(FullVerificationReason),
    Closure(crate::domain_computation::primary_graph::vocabulary::invariant_projection::verification::ConsumedOutputVerificationStop),
    Registration(crate::domain_computation::primary_graph::vocabulary::output_lineage::derived::SettlementRegistrationStop),
    Admission(worth_relational::facade::mvcc::CompanionPreflightStop),
}

impl PublicationRecoveryStop {
    pub(in crate::domain_computation::primary_graph) fn reason(self) -> FullVerificationReason {
        match self {
            Self::Reason(reason) => reason,
            Self::Closure(_) | Self::Registration(_) | Self::Admission(_) => {
                FullVerificationReason::RegistrationIncomplete
            }
        }
    }
}
