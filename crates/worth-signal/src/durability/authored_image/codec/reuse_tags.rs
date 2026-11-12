//! Explicit tags for authored reuse declarations.
use super::{Denial, Read, Write};
use crate::data::reuse::ArtifactSemanticBoundary;
pub(super) fn write_artifact_boundary(
    out: &mut Write<'_>,
    value: ArtifactSemanticBoundary,
) -> Result<(), Denial> {
    out.byte(match value {
        ArtifactSemanticBoundary::TopologyRegime => 0,
        ArtifactSemanticBoundary::ToleranceRegime => 1,
        ArtifactSemanticBoundary::SemanticRegionIdentity => 2,
        ArtifactSemanticBoundary::AuthorityLane => 3,
        ArtifactSemanticBoundary::SnapshotLineage => 4,
        ArtifactSemanticBoundary::ArtifactFamilyBasis => 5,
        ArtifactSemanticBoundary::StructuralDependencyBasis => 6,
        ArtifactSemanticBoundary::PartitionRegionBasis => 7,
        ArtifactSemanticBoundary::PersistentCorrespondence => 8,
        ArtifactSemanticBoundary::CompositionRegionSet => 9,
    })
}
pub(super) fn read_artifact_boundary(
    read: &mut Read<'_>,
) -> Result<ArtifactSemanticBoundary, Denial> {
    Ok(match read.byte()? {
        0 => ArtifactSemanticBoundary::TopologyRegime,
        1 => ArtifactSemanticBoundary::ToleranceRegime,
        2 => ArtifactSemanticBoundary::SemanticRegionIdentity,
        3 => ArtifactSemanticBoundary::AuthorityLane,
        4 => ArtifactSemanticBoundary::SnapshotLineage,
        5 => ArtifactSemanticBoundary::ArtifactFamilyBasis,
        6 => ArtifactSemanticBoundary::StructuralDependencyBasis,
        7 => ArtifactSemanticBoundary::PartitionRegionBasis,
        8 => ArtifactSemanticBoundary::PersistentCorrespondence,
        9 => ArtifactSemanticBoundary::CompositionRegionSet,
        _ => return Err(Denial::Malformed),
    })
}
use crate::data::reuse::ReuseStrategy;
pub(super) fn write_reuse_strategy(
    out: &mut Write<'_>,
    value: ReuseStrategy,
) -> Result<(), Denial> {
    out.byte(match value {
        ReuseStrategy::OutputSuppression => 0,
        ReuseStrategy::MemoizedArtifactReuse => 1,
        ReuseStrategy::SnapshotRestoreReuse => 2,
        ReuseStrategy::ReconciliationAdoption => 3,
        ReuseStrategy::CrossIdentityPersistentMatch => 4,
        ReuseStrategy::PartialArtifactSplicing => 5,
    })
}
pub(super) fn read_reuse_strategy(read: &mut Read<'_>) -> Result<ReuseStrategy, Denial> {
    Ok(match read.byte()? {
        0 => ReuseStrategy::OutputSuppression,
        1 => ReuseStrategy::MemoizedArtifactReuse,
        2 => ReuseStrategy::SnapshotRestoreReuse,
        3 => ReuseStrategy::ReconciliationAdoption,
        4 => ReuseStrategy::CrossIdentityPersistentMatch,
        5 => ReuseStrategy::PartialArtifactSplicing,
        _ => return Err(Denial::Malformed),
    })
}
