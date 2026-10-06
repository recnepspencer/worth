use crate::orchestration::recovery_budget::RecoveryAllowance;
use std::collections::BTreeMap;

use worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia;
use worth_store::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrameView, StoreRecoveryBindingFreshnessSample,
};
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
};
use worth_store_recovery_physics::{
    PhysicalRedoTarget, PhysicalRedoTargetIdentity, PhysicalSourceSelection,
    RecoveryPageObservation,
};

mod absent_target;
mod allocation_truth;
mod failure;
mod historical_drop;
#[cfg(test)]
mod inline_image_fixture;
mod materialized;
mod ordered_history;
mod selected_basis;
mod tier_routes;

use absent_target::{AbsentTarget, SelectedFrontier};
pub(in crate::orchestration::planning) use allocation_truth::InlineAllocationTruth;
pub(super) use failure::{PageLimit, PageObservationFailure};
use materialized::{observe_extent, observe_inline, selected_inline_target};

pub(super) struct PageObservationAttempt {
    pub(super) result: Result<ObservedPageBasis, PageObservationFailure>,
    pub(super) artifact_reads: u64,
    pub(super) bytes_read: u64,
    pub(super) integrity: crate::integrity_ingress::RecoveryIntegrityIngressCounters,
    /// The manifest entries observation charged, whether or not it passed.
    pub(super) manifest_budget: super::manifest_entry_budget::ManifestEntryBudget,
}

pub(super) struct ObservedPageBasis {
    pub(super) observations: Vec<RecoveryPageObservation>,
    pub(super) inline_truth: Option<allocation_truth::InlineAllocationTruth>,
    pub(super) selected_source: crate::progression::RecoverySelectedSourceInventory,
    pub(super) tier_custody: Option<worth_store_recovery_physics::VerifiedSelectedTierEpochCustody>,
    pub(super) historical_drops: Vec<historical_drop::HistoricalDropEvidence>,
    pub(super) ordered_releases: Option<Vec<ordered_history::OrderedReleasedObservation>>,
    pub(super) ordered_history_peak_scratch_bytes: u64,
}
pub(super) use historical_drop::HistoricalDropEvidence;
pub(super) use ordered_history::OrderedReleasedObservation;

#[derive(Clone, Copy)]
pub(super) struct TierEvidence<'a> {
    pub(super) selection: &'a PhysicalSourceSelection,
    pub(super) checkpoint: Option<&'a worth_store_physical_integrity::VerifiedCheckpointStream>,
    pub(super) sample: &'a StoreRecoveryBindingFreshnessSample,
    pub(super) selected_wal: IntegrityAdmittedRecoveryWalFrameView<'a>,
}

pub(super) fn observe_selected_pages(
    media: AdmittedRecoveryFilesystemMedia,
    tier_evidence: TierEvidence<'_>,
    root_manifest: &DurablePhysicalRootManifest,
    retained_fallback: Option<(
        &DurablePhysicalRootManifest,
        PhysicalRecordFormatDeclaration,
    )>,
    placements: &[CurrentPhysicalRecordPlacement],
    admitted_redo: &worth_store_recovery_physics::AdmittedPhysicalRedoMembers,
    format: PhysicalRecordFormatDeclaration,
    limits: &crate::entry::PhysicalRecoveryLimitDeclaration,
    maximum_manifest_entries: u64,
    maximum_bytes: u64,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> (AdmittedRecoveryFilesystemMedia, PageObservationAttempt) {
    let admitted_manifest_entries = limits.manifest_entries;
    // The walk's scratch is recovery's staging bytes.
    let staging = RecoveryAllowance::declared(
        limits,
        crate::entry::PhysicalRecoveryLimitDimension::StagingBytes,
    );
    let targets = admitted_redo.observation_targets();
    let mut discovery = media
        .bounded_discovery(
            crate::orchestration::reader_limit::UNCOUNTED_READS,
            maximum_bytes,
        )
        .expect("admitted nonzero recovery limits create a bounded planning reader");
    let mut integrity = crate::integrity_ingress::RecoveryIntegrityIngressTrace::new();
    let mut manifest_budget = super::manifest_entry_budget::ManifestEntryBudget::declared(
        limits,
        admitted_manifest_entries.saturating_sub(maximum_manifest_entries),
    );
    let result = observe(
        &mut discovery,
        tier_evidence,
        root_manifest,
        retained_fallback,
        placements,
        &targets,
        admitted_redo,
        format,
        admitted_manifest_entries,
        &mut manifest_budget,
        staging,
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
            manifest_budget,
        },
    )
}

fn observe(
    discovery: &mut worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery,
    tier_evidence: TierEvidence<'_>,
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
    budget: &mut super::manifest_entry_budget::ManifestEntryBudget,
    staging: RecoveryAllowance,
    integrity: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<ObservedPageBasis, PageObservationFailure> {
    let selected_unit = budget.charge_root()?;
    let mut selected_source = super::selected_source_inventory::observe_with_budget(
        discovery,
        root_manifest,
        format,
        &selected_unit,
        budget,
        integrity_trace,
    )?;
    let tier_custody = tier_routes::validate(
        root_manifest.generation(),
        placements,
        &selected_source.free_space,
        root_manifest.tier_epoch_anchor(),
        tier_evidence,
    )?;
    if let Some((fallback, fallback_format)) = retained_fallback {
        let fallback_unit = budget.charge_root()?;
        let fallback_source = super::selected_source_inventory::observe_with_budget(
            discovery,
            fallback,
            fallback_format,
            &fallback_unit,
            budget,
            integrity_trace,
        )?;
        let mut source_artifacts = selected_source.source_artifacts.into_vec();
        source_artifacts.extend(fallback_source.source_artifacts);
        source_artifacts.sort_unstable();
        source_artifacts.dedup();
        selected_source.source_artifacts = source_artifacts.into_boxed_slice();
    }
    let mut inline_targets = BTreeMap::<(u64, u64), Vec<&PhysicalRedoTarget>>::new();
    let mut extent_targets = BTreeMap::<u64, BTreeMap<u32, &PhysicalRedoTarget>>::new();
    for target in targets {
        match target.identity() {
            PhysicalRedoTargetIdentity::InlinePage { segment, page, .. } => {
                inline_targets
                    .entry((segment, page))
                    .or_default()
                    .push(target);
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
                let Some(matching) =
                    inline_targets.remove(&(inline.segment().get(), inline.page().get()))
                else {
                    continue;
                };
                let target = selected_inline_target(
                    inline,
                    &matching,
                    format,
                    &selected_source.segment_pages,
                );
                observations.push(observe_inline(
                    discovery,
                    inline,
                    target,
                    format,
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
                        &mut extent_manifests,
                        integrity,
                    )?);
                }
            }
        }
    }
    let absent_targets = inline_targets
        .into_values()
        .filter_map(AbsentTarget::inline_page)
        .chain(
            extent_targets
                .into_values()
                .flat_map(BTreeMap::into_values)
                .map(AbsentTarget::extent_chunk),
        )
        .collect::<Vec<_>>();
    let historical_drop::ClassifiedTargets {
        observations: historical_observations,
        absent: absent_targets,
        drops: historical_drops,
        history_scratch,
        ordered_releases,
    } = historical_drop::classify(
        historical_drop::HistoryWalk {
            discovery,
            selection: tier_evidence.selection,
            selected_root: root_manifest,
            selected_inventory: &selected_source,
            routes: placements,
            redo: admitted_redo,
            release_intents: tier_evidence.sample.release_intents(),
            format,
            budget,
            maximum_entries: admitted_manifest_entries,
            staging,
            trace: integrity_trace,
        },
        &absent_targets,
    )?;
    observations.extend(historical_observations);
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
        tier_custody,
        historical_drops,
        ordered_releases,
        ordered_history_peak_scratch_bytes: history_scratch,
    })
}

pub(super) fn required_source(
    result: Result<
        worth_store::physical_runtime::ObservedRecoveryArtifact,
        worth_store::physical_runtime::RecoveryDiscoveryFailure,
    >,
    target: Option<PhysicalRedoTargetIdentity>,
) -> Result<worth_store::physical_runtime::ObservedRecoveryArtifact, PageObservationFailure> {
    result.map_err(|failure| PageObservationFailure::media(target, failure))
}
