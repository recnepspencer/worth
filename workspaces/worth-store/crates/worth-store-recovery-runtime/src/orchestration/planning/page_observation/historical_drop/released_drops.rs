//! Classify authenticated historical V3 results against selected-root media.

use sha2::{Digest, Sha256};
use std::sync::Arc;
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV3, BlobReclaimSourceBasisV1, BlobRecordV1,
    DropSetManifestV3, PersistedBlobSemanticRecordBinding, PersistedPhysicalRecoveryOperation,
    PersistedPhysicalRecoveryProjection,
};
use worth_store_recovery_physics::{
    RecoveryOperationFate, RecoveryPageObservation, VerifiedOrderedRootHistory,
};

use super::super::ordered_history::OrderedReleasedObservation;
use super::super::{AbsentTarget, PageObservationFailure};
use super::classification::HistoryWalk;
use super::{source_root, target_record, HistoricalDropEvidence};
use crate::entry::HistoricalDropAdmissionStage as Stage;

/// The targets historical V3 drops removed, and the ordered history when one
/// of those drops needed it walked.
pub(super) struct ReleasedDrops<'target> {
    pub(super) remaining: Vec<AbsentTarget<'target>>,
    pub(super) observations: Vec<RecoveryPageObservation>,
    pub(super) evidence: Vec<HistoricalDropEvidence>,
    pub(super) walked: Option<WalkedReleases>,
}

/// The one walk every historical V3 drop of a classification shares.
pub(super) struct WalkedReleases {
    pub(super) history: Arc<VerifiedOrderedRootHistory>,
    pub(super) releases: Vec<OrderedReleasedObservation>,
    pub(super) scratch: u64,
}

/// One admitted V3 drop member at or below the selected root, bound to the
/// descriptor its WAL record carries.
struct HistoricalDrop<'redo> {
    operation: [u8; 32],
    binding: &'redo PersistedBlobSemanticRecordBinding,
    projection: &'redo PersistedPhysicalRecoveryProjection,
    wal_record: &'redo [u8],
    descriptor: BlobReclaimDescriptorV3,
}

pub(super) fn classify<'target>(
    walk: &mut HistoryWalk<'_>,
    targets: &[AbsentTarget<'target>],
) -> Result<ReleasedDrops<'target>, PageObservationFailure> {
    let mut drops = ReleasedDrops {
        remaining: targets.to_vec(),
        observations: Vec::new(),
        evidence: Vec::new(),
        walked: None,
    };
    let redo = walk.redo;
    for (operation, fate, projection, wal_record) in redo.admitted_drop_members() {
        let selected_generation = walk.selected_root.generation();
        if let Some(drop) =
            HistoricalDrop::bound(selected_generation, operation, fate, projection, wal_record)?
        {
            drops.classify(walk, drop)?;
        }
    }
    Ok(drops)
}

impl<'redo> HistoricalDrop<'redo> {
    /// `None` for a member that is not a completed V3 drop at or below the
    /// selected root.
    fn bound(
        selected_generation: u64,
        operation: [u8; 32],
        fate: RecoveryOperationFate,
        projection: &'redo PersistedPhysicalRecoveryProjection,
        wal_record: &'redo [u8],
    ) -> Result<Option<Self>, PageObservationFailure> {
        let PersistedPhysicalRecoveryOperation::RecordsDropped { binding, .. } =
            projection.operation()
        else {
            unreachable!()
        };
        if fate == RecoveryOperationFate::ProvenNoEffect
            || selected_generation < binding.candidate_root_generation()
            || projection.source_root_generation().checked_add(1)
                != Some(binding.candidate_root_generation())
        {
            return Ok(None);
        }
        let Ok(BlobRecordV1::ReclaimDescriptorV3(descriptor)) = decode_blob_record(wal_record)
        else {
            return Ok(None);
        };
        let base = descriptor.base();
        if <[u8; 32]>::from(Sha256::digest(wal_record)) != binding.record_payload_sha256()
            || descriptor.custody().request().idempotency() != operation
            || base.source_root_generation() != projection.source_root_generation()
            || base.candidate_root_generation() != binding.candidate_root_generation()
            || base.manifest_record() == binding.record()
        {
            return Err(invalid(operation, Stage::DescriptorBinding));
        }
        Ok(Some(Self {
            operation,
            binding,
            projection,
            wal_record,
            descriptor,
        }))
    }

    /// Whether the release the ordered walk replayed is this exact drop.
    fn names(&self, release: &OrderedReleasedObservation) -> bool {
        release.descriptor == self.descriptor
            && release.descriptor_frame.bytes() == self.wal_record
            && release.descriptor_frame.record() == self.binding.record()
            && release.manifest_frame.record() == self.descriptor.base().manifest_record()
            && release.candidate_root_generation == self.binding.candidate_root_generation()
    }

    /// Whether the replayed manifest is the one the descriptor names.
    fn carries(&self, manifest: &DropSetManifestV3, manifest_frame_sha256: [u8; 32]) -> bool {
        let base = self.descriptor.base();
        manifest.store() == base.store()
            && manifest.reclaim_attempt() == base.reclaim_attempt()
            && manifest.source_basis_digest() == base.source_basis_digest()
            && manifest.count() == base.manifest_count()
            && manifest_frame_sha256 == base.manifest_frame_sha256()
            && matches!(
                manifest.source_basis(),
                BlobReclaimSourceBasisV1::ReleasedGeneration(_)
            )
    }
}

fn invalid(operation: [u8; 32], stage: Stage) -> PageObservationFailure {
    PageObservationFailure::HistoricalDrop {
        operation,
        stage,
        target: None,
    }
}

/// Every historical V3 drop changed the release-custody head, whose exact
/// effect only the ordered checkpoint-to-selected walk replays. A walk that
/// ran out of an admitted limit is reported as that limit.
fn walked<'slot>(
    slot: &'slot mut Option<WalkedReleases>,
    walk: &mut HistoryWalk<'_>,
    operation: [u8; 32],
) -> Result<&'slot WalkedReleases, PageObservationFailure> {
    if slot.is_none() {
        let (history, releases, scratch) = walk.admit().map_err(|failure| {
            failure
                .stopped()
                .unwrap_or_else(|| invalid(operation, Stage::OrderedHistory))
        })?;
        if releases.len() != historical_count(walk) {
            return Err(invalid(operation, Stage::OrderedHistory));
        }
        *slot = Some(WalkedReleases {
            history: Arc::new(history),
            releases,
            scratch,
        });
    }
    slot.as_ref()
        .ok_or_else(|| invalid(operation, Stage::OrderedHistory))
}

impl ReleasedDrops<'_> {
    fn classify(
        &mut self,
        walk: &mut HistoryWalk<'_>,
        drop: HistoricalDrop<'_>,
    ) -> Result<(), PageObservationFailure> {
        let operation = drop.operation;
        let walked = walked(&mut self.walked, walk, operation)?;
        let mut matches = walked
            .releases
            .iter()
            .filter(|release| release.operation == operation);
        let release = match (matches.next(), matches.next()) {
            (Some(release), None) if drop.names(release) => release,
            _ => return Err(invalid(operation, Stage::OrderedHistory)),
        };
        if !drop.carries(&release.manifest, release.manifest_frame.payload_sha256())
            || !release
                .manifest
                .dropped()
                .iter()
                .all(|record| !walk.routes.iter().any(|route| route.record() == *record))
        {
            return Err(invalid(operation, Stage::ManifestBinding));
        }
        let (source, _source_unit) = source_root(
            walk.discovery,
            drop.projection.source_root_generation(),
            walk.format,
            walk.budget,
        )
        .map_err(|failure| {
            failure
                .stopped()
                .unwrap_or_else(|| invalid(operation, Stage::SourceRoot))
        })?;
        if <[u8; 32]>::from(Sha256::digest(source.encode(walk.format)))
            != drop.descriptor.custody().source_root_frame_sha256()
        {
            return Err(invalid(operation, Stage::SourceRoot));
        }
        observe_dropped(
            &mut self.remaining,
            &mut self.observations,
            walk,
            release,
            &walked.history,
        )?;
        self.evidence.push(HistoricalDropEvidence {
            operation,
            descriptor_record: drop.binding.record(),
            manifest: release.manifest.clone(),
            descriptor: drop.descriptor,
            ordered_history: walked.history.clone(),
        });
        Ok(())
    }
}

/// Moves the targets this release's manifest dropped out of `remaining` and
/// observes each from the release's verified control frames.
fn observe_dropped(
    remaining: &mut Vec<AbsentTarget<'_>>,
    observations: &mut Vec<RecoveryPageObservation>,
    walk: &HistoryWalk<'_>,
    release: &OrderedReleasedObservation,
    history: &VerifiedOrderedRootHistory,
) -> Result<(), PageObservationFailure> {
    let mut matching = Vec::new();
    remaining.retain(|target| {
        let dropped = target_record(target.first())
            .is_some_and(|record| release.manifest.dropped().binary_search(&record).is_ok());
        if dropped {
            matching.push(target.first());
        }
        !dropped
    });
    for target in matching {
        let invalid_target = || PageObservationFailure::HistoricalDrop {
            operation: release.operation,
            stage: Stage::TargetWitness,
            target: Some(target.identity()),
        };
        let witness = walk
            .redo
            .admit_historical_released_drop_target_with_ordered_history(
                walk.selection,
                target,
                release.operation,
                &release.descriptor_frame,
                &release.manifest_frame,
                history,
            )
            .ok_or_else(invalid_target)?;
        observations.push(
            RecoveryPageObservation::historical_released_drop(target, witness)
                .ok_or_else(invalid_target)?,
        );
    }
    Ok(())
}

/// The V3 drops the ordered walk must have replayed.
fn historical_count(walk: &HistoryWalk<'_>) -> usize {
    walk.redo
        .admitted_drop_members()
        .filter(|(_, fate, projection, _)| {
            *fate != RecoveryOperationFate::ProvenNoEffect
                && matches!(projection.operation(),
                PersistedPhysicalRecoveryOperation::RecordsDropped { binding, .. }
                    if binding.candidate_root_generation() <= walk.selected_root.generation())
        })
        .count()
}
