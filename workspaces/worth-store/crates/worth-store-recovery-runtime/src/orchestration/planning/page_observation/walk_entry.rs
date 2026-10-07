//! The walk entry: the one place a walk's manifest-entry budget is built.
//! Every later phase charges the budget this entry hands on, so a walk has
//! exactly one.

use crate::orchestration::recovery_budget::RecoveryAllowance;

use worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
};

use super::{observe, PageObservationAttempt, TierEvidence};

pub(in crate::orchestration::planning) mod manifest_entry_budget;

pub(in crate::orchestration::planning) fn observe_selected_pages(
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
        .expect("a reader that counts no reads opens on any byte bound");
    let mut integrity = crate::integrity_ingress::RecoveryIntegrityIngressTrace::new();
    let mut manifest_budget = manifest_entry_budget::ManifestEntryBudget::declared(
        admitted_manifest_entries,
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
