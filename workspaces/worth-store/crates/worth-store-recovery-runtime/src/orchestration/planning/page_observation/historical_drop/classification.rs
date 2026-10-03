//! Classify authenticated historical V3 results against selected-root media.

use sha2::{Digest, Sha256};
use std::sync::Arc;
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimSourceBasisV1, BlobRecordV1, CurrentPhysicalRecordPlacement,
    PersistedPhysicalRecoveryOperation, PhysicalRecordFormatDeclaration,
};
use worth_store_recovery_physics::{
    AdmittedPhysicalRedoMembers, PhysicalRedoTarget, PhysicalSourceSelection,
    RecoveryOperationFate, RecoveryPageObservation,
};

use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::orchestration::planning::manifest_entry_budget::ManifestEntryBudget;

use super::super::{ordered_history, PageObservationFailure};
use super::{source_root, target_record, HistoricalDropEvidence};
use crate::entry::HistoricalDropAdmissionStage as Stage;
use crate::progression::RecoverySelectedSourceInventory;

pub(in crate::orchestration::planning::page_observation) fn classify<'target>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selection: &PhysicalSourceSelection,
    selected_root: &worth_store_physical_format::DurablePhysicalRootManifest,
    routes: &[CurrentPhysicalRecordPlacement],
    targets: &[&'target PhysicalRedoTarget],
    redo: &AdmittedPhysicalRedoMembers,
    selected_inventory: &RecoverySelectedSourceInventory,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    byte_limit: u64,
    maximum_entries: u64,
    maximum_staging_bytes: u64,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> Result<
    (
        Vec<RecoveryPageObservation>,
        Vec<&'target PhysicalRedoTarget>,
        Vec<HistoricalDropEvidence>,
        u64,
        Option<Vec<ordered_history::OrderedReleasedObservation>>,
    ),
    PageObservationFailure,
> {
    let mut remaining = targets.to_vec();
    let mut observations = Vec::new();
    let mut evidence = Vec::new();
    let mut history_scratch = 0;
    let historical_count = redo
        .admitted_drop_members()
        .filter(|(_, fate, projection, _)| {
            *fate != RecoveryOperationFate::ProvenNoEffect
                && matches!(projection.operation(),
                PersistedPhysicalRecoveryOperation::RecordsDropped { binding, .. }
                    if binding.candidate_root_generation() <= selected_root.generation())
        })
        .count();
    let mut ordered = None;
    let mut ordered_releases = None;
    for (operation, fate, projection, wal_record) in redo.admitted_drop_members() {
        let PersistedPhysicalRecoveryOperation::RecordsDropped { binding, .. } =
            projection.operation()
        else {
            unreachable!()
        };
        if fate == RecoveryOperationFate::ProvenNoEffect
            || selected_root.generation() < binding.candidate_root_generation()
            || projection.source_root_generation().checked_add(1)
                != Some(binding.candidate_root_generation())
        {
            continue;
        }
        let invalid = |stage| PageObservationFailure::HistoricalDrop {
            operation,
            stage,
            target: None,
        };
        let Ok(BlobRecordV1::ReclaimDescriptorV3(descriptor)) = decode_blob_record(wal_record)
        else {
            continue;
        };
        let base = descriptor.base();
        if <[u8; 32]>::from(Sha256::digest(wal_record)) != binding.record_payload_sha256()
            || descriptor.custody().request().idempotency() != operation
            || base.source_root_generation() != projection.source_root_generation()
            || base.candidate_root_generation() != binding.candidate_root_generation()
            || base.manifest_record() == binding.record()
        {
            return Err(invalid(Stage::DescriptorBinding));
        }
        if ordered.is_none() {
            let (admitted, releases, scratch) = ordered_history::admit(
                discovery,
                selection,
                selected_root,
                selected_inventory,
                routes,
                redo,
                format,
                budget,
                byte_limit,
                maximum_entries,
                maximum_staging_bytes,
                trace,
            )
            .ok_or_else(|| invalid(Stage::OrderedHistory))?;
            if releases.len() != historical_count {
                return Err(invalid(Stage::OrderedHistory));
            }
            history_scratch = history_scratch.max(scratch);
            ordered = Some(Arc::new(admitted));
            ordered_releases = Some(releases);
        }
        // Every historical V3 drop changed the release-custody head, whose
        // exact effect only the ordered checkpoint-to-selected walk replays.
        let release = ordered_releases
            .as_ref()
            .and_then(
                |releases: &Vec<ordered_history::OrderedReleasedObservation>| {
                    let mut matches = releases
                        .iter()
                        .filter(|release| release.operation == operation);
                    let value = matches.next()?;
                    matches.next().is_none().then_some(value)
                },
            )
            .ok_or_else(|| invalid(Stage::OrderedHistory))?;
        if release.descriptor != descriptor
            || release.descriptor_frame.bytes() != wal_record
            || release.descriptor_frame.record() != binding.record()
            || release.manifest_frame.record() != base.manifest_record()
            || release.candidate_root_generation != binding.candidate_root_generation()
        {
            return Err(invalid(Stage::OrderedHistory));
        }
        let history = ordered
            .clone()
            .ok_or_else(|| invalid(Stage::OrderedHistory))?;
        let manifest = release.manifest.clone();
        let observed_manifest_sha = release.manifest_frame.payload_sha256();
        if manifest.store() != base.store()
            || manifest.reclaim_attempt() != base.reclaim_attempt()
            || manifest.source_basis_digest() != base.source_basis_digest()
            || manifest.count() != base.manifest_count()
            || observed_manifest_sha != base.manifest_frame_sha256()
            || !matches!(
                manifest.source_basis(),
                BlobReclaimSourceBasisV1::ReleasedGeneration(_)
            )
            || !manifest
                .dropped()
                .iter()
                .all(|record| !routes.iter().any(|route| route.record() == *record))
        {
            return Err(invalid(Stage::ManifestBinding));
        }
        let source = source_root(
            discovery,
            projection.source_root_generation(),
            format,
            budget,
        )
        .ok_or_else(|| invalid(Stage::SourceRoot))?;
        if <[u8; 32]>::from(Sha256::digest(source.encode(format)))
            != descriptor.custody().source_root_frame_sha256()
        {
            return Err(invalid(Stage::SourceRoot));
        }
        let mut matching = Vec::new();
        remaining.retain(|target| {
            let dropped = target_record(target)
                .is_some_and(|record| manifest.dropped().binary_search(&record).is_ok());
            if dropped {
                matching.push(*target);
            }
            !dropped
        });
        for target in matching {
            let invalid_target = || PageObservationFailure::HistoricalDrop {
                operation,
                stage: Stage::TargetWitness,
                target: Some(target.identity()),
            };
            let witness = redo
                .admit_historical_released_drop_target_with_ordered_history(
                    selection,
                    target,
                    operation,
                    &release.descriptor_frame,
                    &release.manifest_frame,
                    &history,
                )
                .ok_or_else(invalid_target)?;
            observations.push(
                RecoveryPageObservation::historical_released_drop(target, witness)
                    .ok_or_else(invalid_target)?,
            );
        }
        evidence.push(HistoricalDropEvidence {
            operation,
            descriptor_record: binding.record(),
            descriptor,
            manifest,
            ordered_history: history,
        });
    }
    // A target published by an ordered edge and removed by a later ordinary
    // retirement edge (for example a V3 drop's replacement directory frame
    // superseded by a derived-directory retirement) is classified from the
    // already-verified history; no further media is read.
    if let Some(history) = ordered.as_deref() {
        remaining.retain(|target| {
            redo.admit_historical_retired_target_with_ordered_history(selection, target, history)
                .and_then(|witness| {
                    RecoveryPageObservation::historical_retired_target(target, witness)
                })
                .map(|observation| observations.push(observation))
                .is_none()
        });
    }
    Ok((
        observations,
        remaining,
        evidence,
        history_scratch,
        ordered_releases,
    ))
}
