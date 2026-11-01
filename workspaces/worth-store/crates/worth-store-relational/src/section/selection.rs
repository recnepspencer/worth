//! Read-only Home selection evidence, below the application composition owner.
use worth_relational::facade::durability::{
    CheckpointSelectionIdentity, CheckpointSelectionObservation, OpaqueCapturedCheckpointSubject,
};
use worth_store::physical_runtime::AnchoredRecordMembership;

/// The Home implementation consults its original attempt and actual receipt.
/// It neither publishes nor acknowledges Native work through this port.
pub trait RelationalCheckpointSelectionSource: Send + Sync + std::fmt::Debug {
    /// Observe the SAME pending capture and its actual receipt without copying
    /// membership. This observes receipt identity; it does not certify membership.
    fn selected_version(
        &self,
        _subject: &OpaqueCapturedCheckpointSubject,
    ) -> Result<
        Option<worth_store::physical_runtime::AnchoredRecordVersion>,
        worth_relational::facade::durability::RelationalDurableOwnedReadRefusal,
    > {
        Err(worth_relational::facade::durability::RelationalDurableOwnedReadRefusal::UnsupportedProvider)
    }
    fn selected_membership(
        &self,
        subject: &OpaqueCapturedCheckpointSubject,
    ) -> Option<AnchoredRecordMembership>;
}

pub(crate) fn observe(
    source: &dyn RelationalCheckpointSelectionSource,
    subject: &OpaqueCapturedCheckpointSubject,
) -> CheckpointSelectionObservation {
    match source.selected_version(subject) {
        Ok(Some(version)) => {
            CheckpointSelectionObservation::Selected(CheckpointSelectionIdentity {
                generation: version.generation(),
                manifest_sha256: version.payload_sha256(),
            })
        }
        Ok(None) => CheckpointSelectionObservation::NotSelected,
        Err(_) => CheckpointSelectionObservation::Unknown,
    }
}
