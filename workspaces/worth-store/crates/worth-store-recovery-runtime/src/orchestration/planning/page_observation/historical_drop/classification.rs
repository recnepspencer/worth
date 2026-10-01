//! Classify authenticated historical V3 results against selected-root media.

use sha2::{Digest, Sha256};
use std::sync::Arc;
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimSourceBasisV1, BlobRecordKind, BlobRecordV1,
    CurrentPhysicalRecordPlacement, PersistedPhysicalRecoveryBlobSemantic,
    PhysicalRecordFormatDeclaration,
};
use worth_store_recovery_physics::{
    AdmittedPhysicalRedoMembers, PhysicalRedoTarget, PhysicalSourceSelection,
    RecoveryOperationFate, RecoveryPageObservation,
};

use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::orchestration::planning::{
    manifest_entry_budget::ManifestEntryBudget, selected_source_inventory::ResidentAllowance,
};

use super::super::{historical_chain, ordered_history, PageObservationFailure};
use super::{selected_control, source_root, target_record, HistoricalDropEvidence};
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
    let mut chain_scratch = 0;
    let mut control_resident = None;
    let historical_count = redo
        .admitted_drop_members()
        .filter(|(_, fate, projection, _)| {
            *fate != RecoveryOperationFate::ProvenNoEffect
                && matches!(projection.blob_semantic(),
                PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(binding)
                    if binding.candidate_root_generation() <= selected_root.generation())
        })
        .count();
    let ordered_required = historical_count > 1
        || redo
            .admitted_drop_members()
            .any(|(_, fate, projection, _)| {
                fate != RecoveryOperationFate::ProvenNoEffect
                    && matches!(projection.blob_semantic(),
                    PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(binding)
                        if binding.candidate_root_generation() < selected_root.generation())
            });
    let mut ordered = None;
    let mut ordered_releases = None;
    for (operation, fate, projection, wal_record) in redo.admitted_drop_members() {
        let PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(binding) =
            projection.blob_semantic()
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
        if ordered_required && ordered.is_none() {
            let (history, releases, scratch) = ordered_history::admit(
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
            chain_scratch = chain_scratch.max(scratch);
            ordered = Some(Arc::new(history));
            ordered_releases = Some(releases);
        }
        let (selected_descriptor, selected_manifest, manifest) = if ordered_required {
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
            {
                return Err(invalid(Stage::OrderedHistory));
            }
            (None, None, release.manifest.clone())
        } else {
            if control_resident.is_none() {
                let routes_bytes = u64::try_from(routes.len())
                    .ok()
                    .and_then(|count| {
                        count.checked_mul(
                            std::mem::size_of::<CurrentPhysicalRecordPlacement>() as u64
                        )
                    })
                    .ok_or_else(|| invalid(Stage::SelectedControls))?;
                let live = selected_inventory
                    .owned_heap_bytes()
                    .and_then(|bytes| bytes.checked_add(routes_bytes))
                    .and_then(|bytes| bytes.checked_add(trace.owned_heap_bytes()?))
                    .ok_or_else(|| invalid(Stage::SelectedControls))?;
                let mut allowance = ResidentAllowance::new(maximum_staging_bytes);
                allowance
                    .bytes(live)
                    .map_err(|_| invalid(Stage::SelectedControls))?;
                control_resident = Some(allowance);
            }
            let resident = control_resident
                .as_mut()
                .ok_or_else(|| invalid(Stage::SelectedControls))?;
            let descriptor_frame = selected_control(
                discovery,
                routes,
                binding.record(),
                BlobRecordKind::ReclaimDescriptorV3,
                format,
                budget,
                trace,
                resident,
            )
            .ok_or_else(|| invalid(Stage::SelectedControls))?;
            if descriptor_frame.bytes() != wal_record {
                return Err(invalid(Stage::SelectedControls));
            }
            let manifest_frame = selected_control(
                discovery,
                routes,
                base.manifest_record(),
                BlobRecordKind::DropSetManifestV3,
                format,
                budget,
                trace,
                resident,
            )
            .ok_or_else(|| invalid(Stage::SelectedControls))?;
            let Ok(BlobRecordV1::DropSetManifestV3(manifest)) =
                decode_blob_record(manifest_frame.bytes())
            else {
                return Err(invalid(Stage::SelectedControls));
            };
            (Some(descriptor_frame), Some(manifest_frame), manifest)
        };
        let observed_manifest_sha = if let Some(frame) = selected_manifest.as_ref() {
            <[u8; 32]>::from(Sha256::digest(frame.bytes()))
        } else {
            ordered_releases
                .as_ref()
                .and_then(|releases| {
                    releases
                        .iter()
                        .find(|release| release.operation == operation)
                })
                .map(|release| release.manifest_frame.payload_sha256())
                .ok_or_else(|| invalid(Stage::OrderedHistory))?
        };
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
        let chain = if ordered_required {
            if ordered_releases.as_ref().is_none_or(
                |releases: &Vec<ordered_history::OrderedReleasedObservation>| {
                    releases
                        .iter()
                        .filter(|release| {
                            release.operation == operation
                                && release.descriptor == descriptor
                                && release.manifest == manifest
                                && release.candidate_root_generation
                                    == binding.candidate_root_generation()
                        })
                        .count()
                        != 1
                },
            ) {
                return Err(invalid(Stage::SourceResultHistory));
            }
            None
        } else if selected_root.generation() > binding.candidate_root_generation() {
            let mut members = redo
                .admitted_root_step_members()
                .filter(|member| member.operation() == operation);
            let first_member = members
                .next()
                .ok_or_else(|| invalid(Stage::SourceResultHistory))?;
            if members.next().is_some() {
                return Err(invalid(Stage::SourceResultHistory));
            }
            let (chain, scratch) = historical_chain::admit(
                discovery,
                selection,
                selected_root,
                selected_inventory,
                routes,
                first_member,
                &manifest,
                redo,
                format,
                budget,
                byte_limit,
                maximum_entries,
                maximum_staging_bytes,
                trace,
            )
            .ok_or_else(|| invalid(Stage::SourceResultHistory))?;
            chain_scratch = chain_scratch.max(scratch);
            Some(chain)
        } else {
            if source
                .record_count()
                .checked_sub(u64::from(manifest.count()))
                .and_then(|count| count.checked_add(1))
                != Some(selected_root.record_count())
            {
                return Err(invalid(Stage::SourceResultHistory));
            }
            None
        };
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
            let witness = if let Some(history) = ordered.as_ref() {
                let release = ordered_releases
                    .as_ref()
                    .and_then(|releases| {
                        releases
                            .iter()
                            .find(|release| release.operation == operation)
                    })
                    .ok_or_else(invalid_target)?;
                redo.admit_historical_released_drop_target_with_ordered_history(
                    selection,
                    target,
                    operation,
                    &release.descriptor_frame,
                    &release.manifest_frame,
                    history,
                )
            } else if let Some(chain) = chain.as_ref() {
                redo.admit_historical_released_drop_target_with_chain(
                    selection,
                    target,
                    operation,
                    selected_descriptor.as_ref().ok_or_else(invalid_target)?,
                    selected_manifest.as_ref().ok_or_else(invalid_target)?,
                    chain,
                )
            } else {
                redo.admit_historical_released_drop_target(
                    selection,
                    target,
                    operation,
                    selected_descriptor.as_ref().ok_or_else(invalid_target)?,
                    selected_manifest.as_ref().ok_or_else(invalid_target)?,
                )
            }
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
            chain,
            ordered_history: ordered.clone(),
        });
    }
    Ok((
        observations,
        remaining,
        evidence,
        chain_scratch,
        ordered_releases,
    ))
}
