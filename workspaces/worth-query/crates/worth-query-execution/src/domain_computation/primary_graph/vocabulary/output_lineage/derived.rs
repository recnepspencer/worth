//! Moved whole from `domain_computation::primary_graph::output_lineage::invalidation::derived`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::output_lineage::mark_state::FullVerificationReason;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;
use im::OrdSet;
use std::sync::Arc;
use worth_relational::facade::mvcc::CompanionCellEditStop;
use worth_relational::facade::mvcc::CompanionPreflightStop;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettlementRegistrationStop {
    Alignment(FullVerificationReason),
    /// The registration's basis or equality belongs to another source.
    Foreign,
    /// The selected source snapshot cannot be positioned.
    SourceUnavailable,
    Admission(CompanionPreflightStop),
    Edit(CompanionCellEditStop),
}

pub enum SourceSettlementCurrentness {
    Clean,
    Dirty(OrdSet<usize>),
    PendingUpstream(OrdSet<Arc<RecordedSettlementIdentity>>),
    FullVerificationRequired(FullVerificationReason),
    /// The selected source belongs to another runtime. Marks here say nothing
    /// about it, and no reader takes it as a reason to verify.
    Foreign,
}

impl SourceSettlementCurrentness {
    /// No mark row answers for the settlement here: the source is foreign, or
    /// its reason is one of [`FullVerificationReason::no_row_answers`].
    pub(in crate::domain_computation::primary_graph) const fn no_row_answers(&self) -> bool {
        match self {
            Self::Foreign => true,
            Self::FullVerificationRequired(reason) => reason.no_row_answers(),
            Self::Clean | Self::Dirty(_) | Self::PendingUpstream(_) => false,
        }
    }
}

impl From<CompanionPreflightStop> for SettlementRegistrationStop {
    fn from(value: CompanionPreflightStop) -> Self {
        Self::Admission(value)
    }
}
