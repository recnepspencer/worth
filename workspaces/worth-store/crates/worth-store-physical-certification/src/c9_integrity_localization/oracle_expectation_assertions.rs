use super::{
    CleanRootArtifactRecord, DeclaredRootCorruption, ExpectedMinimumBlastRadius, ExpectedRootCause,
    ExpectedRootLocalization, ExpectedRootPosture, RootCorruptionCode,
};

pub(super) fn assert_exact_parent_expectation(
    record: &CleanRootArtifactRecord,
    edit: &DeclaredRootCorruption,
    expected: &ExpectedRootLocalization,
) {
    let exact_length = record.exact_length();
    let (posture, cause, ranges, blast_radius) = match edit.code() {
        RootCorruptionCode::B => (
            ExpectedRootPosture::Damaged,
            ExpectedRootCause::CoveredByteIntegrityMismatch,
            std::iter::once(record.covered_edit_offset()..record.covered_edit_offset() + 1)
                .collect(),
            ExpectedMinimumBlastRadius::CanonicalFrame,
        ),
        RootCorruptionCode::K => (
            ExpectedRootPosture::Damaged,
            ExpectedRootCause::ChecksumFieldDamage,
            vec![record.checksum_range()],
            ExpectedMinimumBlastRadius::CanonicalFrame,
        ),
        RootCorruptionCode::L => (
            ExpectedRootPosture::Damaged,
            ExpectedRootCause::FramingLengthMismatch,
            vec![record.length_range()],
            ExpectedMinimumBlastRadius::CanonicalFrame,
        ),
        RootCorruptionCode::S => (
            ExpectedRootPosture::Damaged,
            ExpectedRootCause::ScopeSubstitution,
            record.substitution_changed_ranges().to_vec(),
            ExpectedMinimumBlastRadius::CompleteArtifact,
        ),
        RootCorruptionCode::P => (
            ExpectedRootPosture::Damaged,
            ExpectedRootCause::ChildReferenceMismatch,
            vec![record.pointer_range()],
            ExpectedMinimumBlastRadius::ReachableSubtree,
        ),
        RootCorruptionCode::T => (
            ExpectedRootPosture::Damaged,
            ExpectedRootCause::Truncated,
            std::iter::once(exact_length - 1..exact_length).collect(),
            ExpectedMinimumBlastRadius::CompleteArtifact,
        ),
        RootCorruptionCode::R => (
            ExpectedRootPosture::Missing,
            ExpectedRootCause::ArtifactRemoval,
            std::iter::once(0..exact_length).collect(),
            ExpectedMinimumBlastRadius::ReachableSubtree,
        ),
        RootCorruptionCode::D => (
            ExpectedRootPosture::Duplicate,
            ExpectedRootCause::ArtifactDuplication,
            std::iter::once(0..exact_length).collect(),
            ExpectedMinimumBlastRadius::CompleteArtifact,
        ),
        RootCorruptionCode::U => (
            ExpectedRootPosture::Unsupported,
            ExpectedRootCause::UnsupportedFormatVersion,
            vec![record.version_range()],
            ExpectedMinimumBlastRadius::NoDamageClaim,
        ),
    };
    assert_eq!(expected.posture(), posture);
    assert_eq!(expected.cause(), cause);
    assert_eq!(expected.expected_ranges(), ranges);
    assert_eq!(expected.minimum_blast_radius(), blast_radius);
    if matches!(edit.code(), RootCorruptionCode::P | RootCorruptionCode::R) {
        assert_eq!(
            expected.minimum_reachable_paths().collect::<Vec<_>>(),
            record
                .expected_reachable_paths()
                .iter()
                .map(std::path::PathBuf::as_path)
                .collect::<Vec<_>>()
        );
    } else {
        assert_eq!(expected.minimum_reachable_paths().count(), 0);
    }
}
