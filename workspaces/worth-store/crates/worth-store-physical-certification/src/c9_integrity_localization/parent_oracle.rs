use std::ops::Range;
use std::path::{Path, PathBuf};

use super::corruption_operator::RootCorruptionOperation;
use super::{
    CleanRootArtifactManifest, DeclaredRootCorruption, RootArtifactIdentity,
    RootLocalizationCounters,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExpectedRootPosture {
    Damaged,
    Missing,
    Duplicate,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExpectedRootCause {
    CoveredByteIntegrityMismatch,
    ChecksumFieldDamage,
    FramingLengthMismatch,
    ScopeSubstitution,
    ChildReferenceMismatch,
    Truncated,
    ArtifactRemoval,
    ArtifactDuplication,
    UnsupportedFormatVersion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExpectedMinimumBlastRadius {
    NoDamageClaim,
    CanonicalFrame,
    CompleteArtifact,
    ReachableSubtree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpectedRootLocalization {
    target: RootArtifactIdentity,
    posture: ExpectedRootPosture,
    cause: ExpectedRootCause,
    expected_ranges: Vec<Range<u64>>,
    minimum_blast_radius: ExpectedMinimumBlastRadius,
    minimum_reachable_paths: Vec<PathBuf>,
    manifest_identity: [u8; 32],
    edit_identity: [u8; 32],
}

pub(crate) fn derive_parent_expectation(
    manifest: &CleanRootArtifactManifest,
    edit: &DeclaredRootCorruption,
    counters: &mut RootLocalizationCounters,
) -> Option<ExpectedRootLocalization> {
    if !edit.is_exact_for(manifest) {
        return None;
    }
    let record = manifest.record(edit.target())?;
    let (posture, cause, expected_ranges, minimum_blast_radius) = match edit.operation() {
        RootCorruptionOperation::CoveredByteFlip { offset, .. } => (
            ExpectedRootPosture::Damaged,
            ExpectedRootCause::CoveredByteIntegrityMismatch,
            std::iter::once(*offset..*offset + 1).collect(),
            ExpectedMinimumBlastRadius::CanonicalFrame,
        ),
        RootCorruptionOperation::ChecksumFieldFlip { .. } => (
            ExpectedRootPosture::Damaged,
            ExpectedRootCause::ChecksumFieldDamage,
            vec![record.checksum_range()],
            ExpectedMinimumBlastRadius::CanonicalFrame,
        ),
        RootCorruptionOperation::FramingLengthLie { .. } => (
            ExpectedRootPosture::Damaged,
            ExpectedRootCause::FramingLengthMismatch,
            vec![record.length_range()],
            ExpectedMinimumBlastRadius::CanonicalFrame,
        ),
        RootCorruptionOperation::ScopeSubstitution { .. } => (
            ExpectedRootPosture::Damaged,
            ExpectedRootCause::ScopeSubstitution,
            record.substitution_changed_ranges().to_vec(),
            ExpectedMinimumBlastRadius::CompleteArtifact,
        ),
        RootCorruptionOperation::PointerCorruption { range, .. } => (
            ExpectedRootPosture::Damaged,
            ExpectedRootCause::ChildReferenceMismatch,
            vec![range.clone()],
            ExpectedMinimumBlastRadius::ReachableSubtree,
        ),
        RootCorruptionOperation::StrictPrefixTruncation { retained_length } => (
            ExpectedRootPosture::Damaged,
            ExpectedRootCause::Truncated,
            std::iter::once(*retained_length..record.exact_length()).collect(),
            ExpectedMinimumBlastRadius::CompleteArtifact,
        ),
        RootCorruptionOperation::ArtifactRemoval => (
            ExpectedRootPosture::Missing,
            ExpectedRootCause::ArtifactRemoval,
            std::iter::once(0..record.exact_length()).collect(),
            ExpectedMinimumBlastRadius::ReachableSubtree,
        ),
        RootCorruptionOperation::ArtifactDuplication { .. } => (
            ExpectedRootPosture::Duplicate,
            ExpectedRootCause::ArtifactDuplication,
            std::iter::once(0..record.exact_length()).collect(),
            ExpectedMinimumBlastRadius::CompleteArtifact,
        ),
        RootCorruptionOperation::UnsupportedFormatVersion { range, .. } => (
            ExpectedRootPosture::Unsupported,
            ExpectedRootCause::UnsupportedFormatVersion,
            vec![range.clone()],
            ExpectedMinimumBlastRadius::NoDamageClaim,
        ),
    };
    counters.record_oracle_derivation();
    let minimum_reachable_paths = if matches!(
        edit.operation(),
        RootCorruptionOperation::PointerCorruption { .. }
            | RootCorruptionOperation::ArtifactRemoval
    ) {
        record.expected_reachable_paths().to_vec()
    } else {
        Vec::new()
    };
    Some(ExpectedRootLocalization {
        target: edit.target(),
        posture,
        cause,
        expected_ranges,
        minimum_blast_radius,
        minimum_reachable_paths,
        manifest_identity: manifest.identity(),
        edit_identity: edit.identity(),
    })
}

impl ExpectedRootLocalization {
    pub(crate) const fn target(&self) -> RootArtifactIdentity {
        self.target
    }
    pub(crate) const fn posture(&self) -> ExpectedRootPosture {
        self.posture
    }
    pub(crate) const fn cause(&self) -> ExpectedRootCause {
        self.cause
    }
    pub(crate) fn expected_ranges(&self) -> &[Range<u64>] {
        &self.expected_ranges
    }
    pub(crate) const fn minimum_blast_radius(&self) -> ExpectedMinimumBlastRadius {
        self.minimum_blast_radius
    }
    pub(crate) fn minimum_reachable_paths(&self) -> impl Iterator<Item = &Path> {
        self.minimum_reachable_paths.iter().map(PathBuf::as_path)
    }
    pub(crate) const fn manifest_identity(&self) -> [u8; 32] {
        self.manifest_identity
    }
    pub(crate) const fn edit_identity(&self) -> [u8; 32] {
        self.edit_identity
    }
}
