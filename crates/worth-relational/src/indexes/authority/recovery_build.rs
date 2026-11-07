//! Full-build recovery is an exact registered reconstruction entrance.
use super::{DerivedIndexBuildOutcome, DerivedIndexBuildRequest};
use crate::storage::substrate::maintenance::ReconstructionLane;

pub(super) fn reconstruct(
    request: DerivedIndexBuildRequest,
    build: impl FnOnce(DerivedIndexBuildRequest, &ReconstructionLane<'_>) -> DerivedIndexBuildOutcome,
) -> DerivedIndexBuildOutcome {
    ReconstructionLane::enter_reconstruction(|lane| build(request, lane))
}
