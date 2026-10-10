//! Typed source acquisition failures; text is diagnostic only.
use crate::error::{BridgeExecutionDenial, BridgeTypedError};

/// The cause supplied by a committed-patch, branch-head or snapshot source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBridgeSourceErrorTag {
    /// An opaque external source reported its own failure.
    ExternalSourceFailure,
    /// A retained source binding did not satisfy its exact contract.
    Binding(BridgeSourceBindingDenial),
    /// The native runtime refused commit selection.
    CommitSelection(worth_relational::facade::change_source::RelationalCommitSelectionDenial),
    /// The native runtime refused an observation read.
    ObservationRead(worth_relational::facade::change_source::RelationalObservationReadDenial),
    /// The carried request refused this source contact.
    ExecutionDenied(BridgeExecutionDenial),
    /// Bridge refused the native publication's envelope lowering.
    PublicationDenied(crate::relational_source::RelationalBridgePublicationDenial),
    /// The native commit is not yet visible.
    PublicationDeferred(crate::relational_source::RelationalBridgePublicationDeferred),
    /// The native publication authority became stale.
    PublicationStale(crate::relational_source::RelationalBridgePublicationStale),
    /// The native publication needs a graph rebind.
    PublicationRebindRequired(crate::relational_source::RelationalBridgePublicationRebindRequired),
}

/// Source custody and publication causes remain distinguishable from diagnostics.
pub type RelationalBridgeSourceError = BridgeTypedError<RelationalBridgeSourceErrorTag>;

impl RelationalBridgeSourceError {
    pub(crate) fn execution_denied(denial: BridgeExecutionDenial) -> Self {
        Self::new(
            RelationalBridgeSourceErrorTag::ExecutionDenied(denial),
            denial.label(),
        )
    }

    pub(crate) fn historical_delivery_kind(&self) -> crate::error::BridgeDeliveryErrorKind {
        match self.kind() {
            RelationalBridgeSourceErrorTag::ExecutionDenied(denial) => {
                crate::error::BridgeDeliveryErrorKind::ExecutionDenied(denial)
            }
            RelationalBridgeSourceErrorTag::ExternalSourceFailure
            | RelationalBridgeSourceErrorTag::Binding(_)
            | RelationalBridgeSourceErrorTag::CommitSelection(_)
            | RelationalBridgeSourceErrorTag::ObservationRead(_)
            | RelationalBridgeSourceErrorTag::PublicationDenied(_)
            | RelationalBridgeSourceErrorTag::PublicationDeferred(_)
            | RelationalBridgeSourceErrorTag::PublicationStale(_)
            | RelationalBridgeSourceErrorTag::PublicationRebindRequired(_) => {
                crate::error::BridgeDeliveryErrorKind::HistoricalTruthViewUnavailable(self.clone())
            }
        }
    }

    pub(crate) fn delivery_kind(&self) -> crate::error::BridgeDeliveryErrorKind {
        match self.kind() {
            RelationalBridgeSourceErrorTag::ExecutionDenied(denial) => {
                crate::error::BridgeDeliveryErrorKind::ExecutionDenied(denial)
            }
            RelationalBridgeSourceErrorTag::ExternalSourceFailure
            | RelationalBridgeSourceErrorTag::Binding(_)
            | RelationalBridgeSourceErrorTag::CommitSelection(_)
            | RelationalBridgeSourceErrorTag::ObservationRead(_)
            | RelationalBridgeSourceErrorTag::PublicationDenied(_)
            | RelationalBridgeSourceErrorTag::PublicationDeferred(_)
            | RelationalBridgeSourceErrorTag::PublicationStale(_)
            | RelationalBridgeSourceErrorTag::PublicationRebindRequired(_) => {
                crate::error::BridgeDeliveryErrorKind::SnapshotAcquisitionFailure(self.clone())
            }
        }
    }
}

/// Exact binding failures owned by Bridge's retained source tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeSourceBindingDenial {
    /// The commit identity has no native relational projection.
    UnsupportedCommitIdentity,
    /// The snapshot identity has no native relational projection.
    UnsupportedSnapshotIdentity,
    /// No admitted head is bound to the requested branch.
    BranchHeadNotBound,
    /// Multiple branches bind the requested head commit.
    AmbiguousBranchHead,
    /// No retained observation is bound to the snapshot.
    SnapshotNotBound,
    /// The retained observation selects another snapshot version.
    SnapshotVersionMismatch,
    /// No retained observation is bound to the commit.
    CommitNotBound,
    /// Multiple retained observations are bound to the commit.
    AmbiguousCommitObservation,
    /// Another source registration owns the retained observation.
    ForeignRegistrationOwner,
    /// The observation selects another truth branch.
    TruthBranchMismatch,
    /// The record belongs to another partition authority.
    PartitionAuthorityMismatch,
    /// An entity projection received a relation identity.
    EntityIdentityRequired,
}
