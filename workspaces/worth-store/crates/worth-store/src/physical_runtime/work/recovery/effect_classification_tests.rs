use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use super::{
    super::PhysicalWorkOperationFamily as Family,
    effect_classification::{journaling, PhysicalEffectJournaling},
    PhysicalCheckpointRecoveryAction as Action, PhysicalWorkRecoveryTarget as Target,
};
use PhysicalEffectJournaling::{BeforeEffect, OnlyIfRetained};

const TARGET_ROWS: usize = 14;
const FAMILY_ROWS: usize = 9;

/// How a family's effects are journaled.
#[derive(Debug, Clone, Copy)]
enum FamilyJournaling {
    /// The target decides.
    ByTarget,
    /// Always a pure flush.
    OnlyIfRetained,
}

#[test]
fn every_family_and_target_variant_has_pinned_journaling() {
    assert_covers(
        target_table().map(|(target, _)| target_row(target)),
        TARGET_ROWS,
    );
    assert_covers(
        family_table().map(|(family, _)| family_row(family)),
        FAMILY_ROWS,
    );
    for (family, family_journaling) in family_table() {
        for (target, target_journaling) in target_table() {
            let expected = match family_journaling {
                FamilyJournaling::ByTarget => target_journaling,
                FamilyJournaling::OnlyIfRetained => OnlyIfRetained,
            };
            assert_eq!(
                journaling(family, target),
                expected,
                "{family:?} effect on {target:?}"
            );
        }
    }
}

fn assert_covers<const N: usize>(rows: [usize; N], expected: usize) {
    let mut rows = rows.to_vec();
    rows.sort_unstable();
    rows.dedup();
    assert_eq!(rows, (0..expected).collect::<Vec<_>>());
}

fn target_table() -> [(Target, PhysicalEffectJournaling); TARGET_ROWS] {
    let file = RecordArtifactFile::BootstrapCatalog;
    let checkpoint = |action| Target::Checkpoint {
        sequence: 1,
        action,
    };
    [
        (
            Target::Range(RecordFrameCoordinate::new(file, 8, 8).unwrap()),
            BeforeEffect,
        ),
        (
            Target::WalArtifactInterval {
                segment: 1,
                generation: 1,
                offset: 0,
                byte_count: 64,
            },
            BeforeEffect,
        ),
        (
            checkpoint(Action::CreateCandidate { byte_count: 64 }),
            BeforeEffect,
        ),
        (
            checkpoint(Action::AppendCandidate {
                offset: 64,
                byte_count: 64,
            }),
            BeforeEffect,
        ),
        (checkpoint(Action::SynchronizeCandidate), OnlyIfRetained),
        (checkpoint(Action::RemoveCandidate), BeforeEffect),
        (checkpoint(Action::PublishCandidate), BeforeEffect),
        (checkpoint(Action::SynchronizeNamespace), OnlyIfRetained),
        (
            Target::WalSegmentReclamation {
                segment: 1,
                generation: 1,
            },
            BeforeEffect,
        ),
        (Target::ArtifactFileSynchronization(file), OnlyIfRetained),
        (Target::ArtifactParentSynchronization(file), OnlyIfRetained),
        (Target::CatalogReplacement(file), BeforeEffect),
        (Target::RecordNamespaceSynchronization, OnlyIfRetained),
        (Target::ArtifactRemoval(file), BeforeEffect),
    ]
}

fn family_table() -> [(Family, FamilyJournaling); FAMILY_ROWS] {
    [
        (Family::ArtifactMetadataRead, FamilyJournaling::ByTarget),
        (Family::ArtifactRangeRead, FamilyJournaling::ByTarget),
        (Family::ArtifactRangeWrite, FamilyJournaling::ByTarget),
        (Family::ArtifactPublication, FamilyJournaling::ByTarget),
        (Family::CheckpointCapture, FamilyJournaling::ByTarget),
        (Family::WalAppend, FamilyJournaling::ByTarget),
        (Family::DurabilityBarrier, FamilyJournaling::OnlyIfRetained),
        (Family::WalReclamation, FamilyJournaling::ByTarget),
        (Family::RootPublication, FamilyJournaling::ByTarget),
    ]
}

/// No wildcard: a new target variant or checkpoint action does not compile
/// until it has a row in the table.
const fn target_row(target: Target) -> usize {
    match target {
        Target::Range(_) => 0,
        Target::WalArtifactInterval { .. } => 1,
        Target::Checkpoint { action, .. } => match action {
            Action::CreateCandidate { .. } => 2,
            Action::AppendCandidate { .. } => 3,
            Action::SynchronizeCandidate => 4,
            Action::RemoveCandidate => 5,
            Action::PublishCandidate => 6,
            Action::SynchronizeNamespace => 7,
        },
        Target::WalSegmentReclamation { .. } => 8,
        Target::ArtifactFileSynchronization(_) => 9,
        Target::ArtifactParentSynchronization(_) => 10,
        Target::CatalogReplacement(_) => 11,
        Target::RecordNamespaceSynchronization => 12,
        Target::ArtifactRemoval(_) => 13,
    }
}

/// No wildcard: a new family does not compile until it has a row.
const fn family_row(family: Family) -> usize {
    match family {
        Family::ArtifactMetadataRead => 0,
        Family::ArtifactRangeRead => 1,
        Family::ArtifactRangeWrite => 2,
        Family::ArtifactPublication => 3,
        Family::CheckpointCapture => 4,
        Family::WalAppend => 5,
        Family::DurabilityBarrier => 6,
        Family::WalReclamation => 7,
        Family::RootPublication => 8,
    }
}
