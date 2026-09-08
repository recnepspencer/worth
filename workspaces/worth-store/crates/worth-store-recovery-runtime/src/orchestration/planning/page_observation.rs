use std::collections::BTreeMap;

use worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    RecordArtifactFile,
};
use worth_store_recovery_physics::{
    PhysicalRedoTarget, PhysicalRedoTargetIdentity, RecoveryPageObservation,
};

mod allocation_truth;
mod failure;
mod materialized;
mod selected_basis;

pub(in crate::orchestration::planning) use allocation_truth::InlineAllocationTruth;
pub(super) use failure::PageObservationFailure;
use materialized::{observe_extent, observe_inline};

pub(super) struct PageObservationAttempt {
    pub(super) result: Result<ObservedPageBasis, PageObservationFailure>,
    pub(super) artifact_reads: u64,
    pub(super) bytes_read: u64,
    pub(super) integrity: crate::integrity_ingress::RecoveryIntegrityIngressCounters,
}

pub(super) struct ObservedPageBasis {
    pub(super) observations: Vec<RecoveryPageObservation>,
    pub(super) inline_truth: Option<allocation_truth::InlineAllocationTruth>,
    pub(super) selected_source: crate::progression::RecoverySelectedSourceInventory,
    pub(super) manifest_budget: super::manifest_entry_budget::ManifestEntryBudget,
}

pub(super) use selected_basis::{artifact_read_ceiling, ArtifactReadCeilingDenial};

pub(super) fn observe_selected_pages(
    media: AdmittedRecoveryFilesystemMedia,
    root_manifest: &DurablePhysicalRootManifest,
    retained_fallback: Option<(
        &DurablePhysicalRootManifest,
        PhysicalRecordFormatDeclaration,
    )>,
    placements: &[CurrentPhysicalRecordPlacement],
    admitted_redo: &worth_store_recovery_physics::AdmittedPhysicalRedoMembers,
    format: PhysicalRecordFormatDeclaration,
    maximum_entries: u64,
    admitted_manifest_entries: u64,
    maximum_manifest_entries: u64,
    maximum_bytes: u64,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> (AdmittedRecoveryFilesystemMedia, PageObservationAttempt) {
    let targets = admitted_redo.observation_targets();
    let mut discovery = media
        .bounded_discovery(maximum_entries, maximum_bytes)
        .expect("admitted nonzero recovery limits create a bounded planning reader");
    let mut integrity = crate::integrity_ingress::RecoveryIntegrityIngressTrace::new();
    let result = observe(
        &mut discovery,
        root_manifest,
        retained_fallback,
        placements,
        &targets,
        admitted_redo,
        format,
        admitted_manifest_entries,
        maximum_manifest_entries,
        maximum_bytes,
        &mut integrity,
        integrity_trace,
    );
    let counters = discovery.counters();
    let page_counters = integrity.counters();
    integrity_trace.append(integrity);
    (
        discovery.finish(),
        PageObservationAttempt {
            result,
            artifact_reads: counters.addressed_artifacts_read,
            bytes_read: counters.bytes_read,
            integrity: page_counters,
        },
    )
}

fn observe(
    discovery: &mut worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery,
    root_manifest: &DurablePhysicalRootManifest,
    retained_fallback: Option<(
        &DurablePhysicalRootManifest,
        PhysicalRecordFormatDeclaration,
    )>,
    placements: &[CurrentPhysicalRecordPlacement],
    targets: &[PhysicalRedoTarget],
    admitted_redo: &worth_store_recovery_physics::AdmittedPhysicalRedoMembers,
    format: PhysicalRecordFormatDeclaration,
    admitted_manifest_entries: u64,
    maximum_manifest_entries: u64,
    byte_limit: u64,
    integrity: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<ObservedPageBasis, PageObservationFailure> {
    let already_observed = admitted_manifest_entries.saturating_sub(maximum_manifest_entries);
    let mut budget = super::manifest_entry_budget::ManifestEntryBudget::new(
        admitted_manifest_entries,
        already_observed,
    );
    let mut selected_source = super::selected_source_inventory::observe_with_budget(
        discovery,
        root_manifest,
        format,
        &mut budget,
        byte_limit,
        integrity_trace,
    )?;
    if let Some((fallback, fallback_format)) = retained_fallback {
        let fallback_source = super::selected_source_inventory::observe_with_budget(
            discovery,
            fallback,
            fallback_format,
            &mut budget,
            byte_limit,
            integrity_trace,
        )?;
        let mut source_artifacts = selected_source.source_artifacts.into_vec();
        source_artifacts.extend(fallback_source.source_artifacts);
        source_artifacts.sort_unstable();
        source_artifacts.dedup();
        selected_source.source_artifacts = source_artifacts.into_boxed_slice();
    }
    let mut inline_targets = BTreeMap::new();
    let mut extent_targets = BTreeMap::<u64, BTreeMap<u32, &PhysicalRedoTarget>>::new();
    for target in targets {
        match target.identity() {
            PhysicalRedoTargetIdentity::InlinePage { segment, page, .. } => {
                inline_targets.entry((segment, page)).or_insert(target);
            }
            PhysicalRedoTargetIdentity::ExtentChunk { extent, chunk, .. } => {
                extent_targets
                    .entry(extent)
                    .or_default()
                    .entry(chunk)
                    .or_insert(target);
            }
        }
    }
    let mut observations = Vec::new();
    let absence_identity =
        selected_basis::selected_absence_identity(root_manifest, placements, format);
    let mut extent_manifests = BTreeMap::new();
    for placement in placements {
        match *placement {
            CurrentPhysicalRecordPlacement::Inline(inline) => {
                let Some(target) =
                    inline_targets.remove(&(inline.segment().get(), inline.page().get()))
                else {
                    continue;
                };
                observations.push(observe_inline(
                    discovery,
                    inline,
                    target,
                    format,
                    byte_limit,
                    &selected_source.segment_pages,
                    integrity,
                )?);
            }
            CurrentPhysicalRecordPlacement::Extent(extent) => {
                let Some(matching) = extent_targets.remove(&extent.extent().get()) else {
                    continue;
                };
                for target in matching.into_values() {
                    observations.push(observe_extent(
                        discovery,
                        extent,
                        target,
                        format,
                        byte_limit,
                        &mut extent_manifests,
                        integrity,
                    )?);
                }
            }
        }
    }
    let absent_targets = inline_targets
        .into_values()
        .chain(extent_targets.into_values().flat_map(BTreeMap::into_values))
        .collect();
    let absent = allocation_truth::admit_absent_targets(
        root_manifest,
        placements,
        absent_targets,
        &selected_source,
        absence_identity,
        admitted_redo,
    )?;
    observations.extend(absent.observations);
    Ok(ObservedPageBasis {
        observations,
        inline_truth: absent.inline_truth,
        selected_source,
        manifest_budget: budget,
    })
}

pub(super) fn required_source(
    result: Result<
        worth_store::physical_runtime::ObservedRecoveryArtifact,
        worth_store::physical_runtime::RecoveryDiscoveryFailure,
    >,
    target: Option<PhysicalRedoTargetIdentity>,
    artifact: RecordArtifactFile,
) -> Result<worth_store::physical_runtime::ObservedRecoveryArtifact, PageObservationFailure> {
    match result {
        Ok(observed) => Ok(observed),
        Err(worth_store::physical_runtime::RecoveryDiscoveryFailure::ByteLimitExceeded {
            ..
        }) => Err(PageObservationFailure::ByteLimit),
        Err(failure) => Err(PageObservationFailure::Media { target, failure }),
    }
}
