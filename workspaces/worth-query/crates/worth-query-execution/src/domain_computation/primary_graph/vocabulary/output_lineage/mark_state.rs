//! Moved whole from `domain_computation::primary_graph::output_lineage::invalidation::mark_state`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::provider::failure::RebaseVerificationFailure;

/// Why a retained row's first reader compares it in full. Each value is a
/// reason a row can record. A stop ends the request that meets it instead,
/// so none is here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FullVerificationReason {
    NativeRevisionUnavailable,
    NativeFactRevisionUnavailable(usize),
    IndexedSelectionDenied(crate::domain_computation::primary_graph::vocabulary::application_attempt::indexed_entity_selection::IndexedSelectionReobserveDenial),
    IndexedSelectionFactDenied(usize, crate::domain_computation::primary_graph::vocabulary::application_attempt::indexed_entity_selection::IndexedSelectionReobserveDenial),
    UnsupportedFact,
    CheckpointRestore,
    DifferentBranch,
    BeforeReadBasis,
    RetainedDeliveryGap,
    MissingSettlement,
    DeclaredChangeUnavailable,
    /// A stop ended this row's registration after its World effect, which
    /// stays authoritative, so the row holds no complete marks.
    RegistrationIncomplete,
    /// Original computation inputs were changed by its own publication.
    ComputationSuperseded,
}

impl FullVerificationReason {
    /// No mark row answers for the settlement at the selected source: there
    /// is none, it is still in its checkpoint posture, or it was read on
    /// another branch or at a later basis. A foreign source is the one other
    /// such case, and it is a stop, not a reason.
    pub(in crate::domain_computation::primary_graph) const fn no_row_answers(self) -> bool {
        matches!(
            self,
            Self::CheckpointRestore
                | Self::MissingSettlement
                | Self::DifferentBranch
                | Self::BeforeReadBasis
        )
    }
}

/// Why the row of a commit that could not rebase its facts requires
/// verification.
impl From<RebaseVerificationFailure> for FullVerificationReason {
    fn from(reason: RebaseVerificationFailure) -> Self {
        reason.into_registration_requirement()
    }
}
