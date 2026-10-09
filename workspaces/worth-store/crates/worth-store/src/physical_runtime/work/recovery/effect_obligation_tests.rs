use std::num::NonZeroU64;

use worth_proof::TransitionOutcome;
use worth_store_physical_backend::{
    FilesystemAccessPosture, MediaCounterSnapshot, MediaOperationRole, QualifiedFilesystemMedia,
};
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use super::{
    super::PhysicalWorkOperationFamily, PhysicalCheckpointRecoveryAction, PhysicalEffectJournal,
    PhysicalEffectRecoveryInventory, PhysicalWorkRecoveryTarget,
};
use crate::physical_runtime::work::{
    PhysicalOperationIdentity, PhysicalWorkGeneration, PhysicalWorkIdentity,
};
use crate::physical_runtime::{
    FilesystemMediaAdmission, LifecycleGeneration, MediaShutdownOutcome, PhysicalRuntimeAdmission,
    PhysicalStore, RuntimeIdentity,
};

const INVENTORY_LIMIT: usize = 16;

#[test]
fn synchronization_only_effects_journal_nothing_unless_retained() {
    let root = tempfile::tempdir().unwrap();
    with_media(root.path(), |media| {
        let journal = PhysicalEffectJournal::new(media);
        for (operation, (family, target, digest)) in
            synchronization_only_effects().into_iter().enumerate()
        {
            let before = media.counters();
            let prepared = journal
                .prepare(
                    media,
                    identity(media, operation as u64 + 1),
                    family,
                    target,
                    digest,
                )
                .unwrap();
            journal.finish(media, prepared).unwrap();
            assert_eq!(
                journal_cost(before, media.counters()),
                JournalCost::default(),
                "{target:?} must not journal a completed flush"
            );
        }
        assert!(inventory(media).obligations().is_empty());

        let (family, target, digest) = synchronization_only_effects()[5];
        let before = media.counters();
        let prepared = journal
            .prepare(media, identity(media, 100), family, target, digest)
            .unwrap();
        journal.retain(media, prepared).unwrap();
        let cost = journal_cost(before, media.counters());
        assert_eq!((cost.records, cost.file_syncs), (1, 1));
        let inventory = inventory(media);
        assert_eq!(inventory.obligations().len(), 1);
        assert_eq!(inventory.obligations()[0].target(), target);
        assert!(inventory.requires_inspection());
    });
}

#[test]
fn journaled_effect_writes_its_record_before_and_retires_it_durably() {
    let root = tempfile::tempdir().unwrap();
    with_media(root.path(), |media| {
        let journal = PhysicalEffectJournal::new(media);
        // The first record also creates the journal directory.
        let warm = journal
            .prepare(
                media,
                identity(media, 1),
                range_family(),
                range_target(),
                digest(),
            )
            .unwrap();
        journal.finish(media, warm).unwrap();

        let before = media.counters();
        let prepared = journal
            .prepare(
                media,
                identity(media, 2),
                range_family(),
                range_target(),
                digest(),
            )
            .unwrap();
        let prepared_at = media.counters();
        assert_eq!(
            journal_cost(before, prepared_at),
            JournalCost {
                records: 1,
                file_syncs: 1,
                directory_syncs: 1,
                removals: 0,
            }
        );
        assert_eq!(inventory(media).obligations().len(), 1);
        journal.finish(media, prepared).unwrap();
        assert_eq!(
            journal_cost(prepared_at, media.counters()),
            JournalCost {
                directory_syncs: 1,
                removals: 1,
                ..JournalCost::default()
            },
            "a retirement is durable before the effect settles"
        );
        assert!(inventory(media).obligations().is_empty());
    });
}

#[derive(Debug, Default, PartialEq, Eq)]
struct JournalCost {
    records: u64,
    file_syncs: u64,
    directory_syncs: u64,
    removals: u64,
}

fn journal_cost(before: MediaCounterSnapshot, after: MediaCounterSnapshot) -> JournalCost {
    let delta = |role| after.attempts_for(role) - before.attempts_for(role);
    JournalCost {
        records: delta(MediaOperationRole::CreateNew),
        file_syncs: after.file_syncs() - before.file_syncs(),
        directory_syncs: after.directory_syncs() - before.directory_syncs(),
        removals: delta(MediaOperationRole::Delete),
    }
}

type SynchronizationOnlyEffect = (
    PhysicalWorkOperationFamily,
    PhysicalWorkRecoveryTarget,
    Option<[u8; 32]>,
);

fn synchronization_only_effects() -> [SynchronizationOnlyEffect; 6] {
    let catalog = RecordArtifactFile::BootstrapCatalog;
    let checkpoint = |action| PhysicalWorkRecoveryTarget::Checkpoint {
        sequence: 1,
        action,
    };
    [
        (
            PhysicalWorkOperationFamily::ArtifactPublication,
            PhysicalWorkRecoveryTarget::ArtifactFileSynchronization(catalog),
            None,
        ),
        (
            PhysicalWorkOperationFamily::ArtifactPublication,
            PhysicalWorkRecoveryTarget::ArtifactParentSynchronization(catalog),
            None,
        ),
        (
            PhysicalWorkOperationFamily::RootPublication,
            PhysicalWorkRecoveryTarget::RecordNamespaceSynchronization,
            None,
        ),
        (
            PhysicalWorkOperationFamily::CheckpointCapture,
            checkpoint(PhysicalCheckpointRecoveryAction::SynchronizeCandidate),
            None,
        ),
        (
            PhysicalWorkOperationFamily::CheckpointCapture,
            checkpoint(PhysicalCheckpointRecoveryAction::SynchronizeNamespace),
            None,
        ),
        (
            PhysicalWorkOperationFamily::DurabilityBarrier,
            PhysicalWorkRecoveryTarget::WalArtifactInterval {
                segment: 1,
                generation: 1,
                offset: 0,
                byte_count: 64,
            },
            digest(),
        ),
    ]
}

const fn range_family() -> PhysicalWorkOperationFamily {
    PhysicalWorkOperationFamily::ArtifactRangeWrite
}

fn range_target() -> PhysicalWorkRecoveryTarget {
    PhysicalWorkRecoveryTarget::Range(
        RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 8, 8).unwrap(),
    )
}

const fn digest() -> Option<[u8; 32]> {
    Some([0x5a; 32])
}

fn inventory(media: &QualifiedFilesystemMedia) -> PhysicalEffectRecoveryInventory {
    PhysicalEffectRecoveryInventory::inspect(media, INVENTORY_LIMIT)
}

fn identity(media: &QualifiedFilesystemMedia, operation: u64) -> PhysicalWorkIdentity {
    PhysicalWorkIdentity::from_instance_owner(
        media.store_identity(),
        RuntimeIdentity::from_reopened(NonZeroU64::new(3).unwrap()),
        PhysicalWorkGeneration::from_lifecycle(LifecycleGeneration::from_reopened(
            NonZeroU64::new(5).unwrap(),
        )),
        PhysicalOperationIdentity::from_reopened(NonZeroU64::new(operation).unwrap()),
    )
}

fn with_media(root: &std::path::Path, body: impl FnOnce(&QualifiedFilesystemMedia)) {
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let owned = match runtime.try_admit_filesystem_media(admission).into_raw() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("Store-owned backend admission failed"),
    };
    body(owned.record_serving_media());
    assert!(matches!(owned.close(), MediaShutdownOutcome::Released(_)));
}
