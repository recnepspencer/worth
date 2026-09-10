use super::{ArtifactTransitionKind, InvalidationCause};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for ArtifactTransitionKind {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::Replaced
            | Self::MemoizedReuse
            | Self::SnapshotRestoreReuse
            | Self::ReconciliationAdoption
            | Self::Refreshed { output_change: _ }
            | Self::CrossIdentityPersistentReuse {
                correspondence_kind: _,
            }
            | Self::PartialArtifactSplice {
                composition_region_count: _,
                recomputed_region_count: _,
            } => Ok(Charge::ZERO),
        }
    }
}
impl RetainedStorageMeasurement for InvalidationCause {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::SourceAspectChanged { aspect_index: _ }
            | Self::DirectDependencyChanged {
                dependency: _,
                aspect_index: _,
            }
            | Self::TransitiveDependencyChanged { aspect_index: _ }
            | Self::PendingDependencyRevalidation { upstream: _ } => Ok(Charge::ZERO),
        }
    }
}
