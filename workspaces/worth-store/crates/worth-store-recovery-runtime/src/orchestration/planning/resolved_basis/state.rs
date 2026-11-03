//! The actual selected planning evidence and its retained memory census.
use super::super::counters;
use worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample;
use worth_store_recovery_physics::{
    ImmutablePhysicalRedoPlan, PhysicalRedoTargetIdentity, ReconciledOperationFates,
    RecoveryPlanningCounters,
};

pub(in crate::orchestration::planning) struct PageObservationResult {
    pub(in crate::orchestration::planning) observations:
        Vec<worth_store_recovery_physics::RecoveryPageObservation>,
    pub(in crate::orchestration::planning) artifact_reads: u64,
    pub(in crate::orchestration::planning) bytes_read: u64,
    pub(in crate::orchestration::planning) source_copy_reads: u64,
    pub(in crate::orchestration::planning) source_copy_bytes_read: u64,
    pub(in crate::orchestration::planning) source_copy_peak_scratch_bytes: u64,
    pub(in crate::orchestration::planning) historical_publication_reads: u64,
    pub(in crate::orchestration::planning) historical_publication_bytes_read: u64,
    pub(in crate::orchestration::planning) historical_publication_peak_scratch_bytes: u64,
    pub(in crate::orchestration::planning) candidate_artifact_reads: u64,
    pub(in crate::orchestration::planning) candidate_bytes_read: u64,
    pub(in crate::orchestration::planning) candidate_peak_materialization_bytes: u64,
    pub(in crate::orchestration::planning) successor_root_integrity_admissions: u64,
    pub(in crate::orchestration::planning) successor_root_interpretations: u64,
    pub(in crate::orchestration::planning) inline_truth:
        Option<super::super::page_observation::InlineAllocationTruth>,
    pub(in crate::orchestration::planning) selected_source:
        crate::progression::RecoverySelectedSourceInventory,
    pub(in crate::orchestration::planning) manifest_budget:
        super::super::manifest_entry_budget::ManifestEntryBudget,
    pub(in crate::orchestration::planning) tier_custody:
        Option<worth_store_recovery_physics::VerifiedSelectedTierEpochCustody>,
    pub(in crate::orchestration::planning) historical_drops:
        Vec<super::super::page_observation::HistoricalDropEvidence>,
    pub(in crate::orchestration::planning) ordered_releases:
        Option<Vec<super::super::page_observation::OrderedReleasedObservation>>,
    pub(in crate::orchestration::planning) integrity:
        crate::integrity_ingress::RecoveryIntegrityIngressCounters,
}

pub(in crate::orchestration::planning) struct ResolvedPlanningState {
    pub(super) unused_backing_bytes: u64,
    pub(super) capture_peak: u64,
    pub(in crate::orchestration::planning) sample: StoreRecoveryBindingFreshnessSample,
    pub(in crate::orchestration::planning) fates: ReconciledOperationFates,
    pub(in crate::orchestration::planning) redo: ImmutablePhysicalRedoPlan,
    pub(in crate::orchestration::planning) targets: Box<[PhysicalRedoTargetIdentity]>,
    pub(in crate::orchestration::planning) redo_bytes: u64,
    pub(in crate::orchestration::planning) distinct_targets: u64,
    pub(in crate::orchestration::planning) observed_pages: PageObservationResult,
    pub(in crate::orchestration::planning) verified_drops:
        Vec<worth_store_physical_format::PersistedRecordIdentity>,
    pub(in crate::orchestration::planning) verified_historical_release_operations: Vec<[u8; 32]>,
    pub(in crate::orchestration::planning) historical_consumed:
        Option<worth_store_recovery_physics::HistoricalConsumedOperationSet>,
    pub(in crate::orchestration::planning) verified_historical_release_sources: Vec<(
        [u8; 32],
        worth_store_physical_format::DurablePhysicalRootManifest,
        worth_store_physical_format::DurableFreeSpaceManifestHeader,
    )>,
    pub(in crate::orchestration::planning) custody: crate::progression::PlanningCustody,
    pub(in crate::orchestration::planning) validated_manifest_cleanup:
        Option<crate::orchestration::ValidatedManifestResidueCleanup>,
}

impl ResolvedPlanningState {
    pub(in crate::orchestration::planning) fn unused_backing_bytes(&self) -> u64 {
        self.unused_backing_bytes
    }
    pub(in crate::orchestration::planning) fn capture_peak(&self) -> u64 {
        self.capture_peak
    }
    pub(in crate::orchestration::planning) fn planning_counters_at_peak(
        &self,
        peak: u64,
    ) -> RecoveryPlanningCounters {
        self.planning_counters()
            .with_peak_recovery_bytes(self.capture_peak.max(peak))
    }

    pub(in crate::orchestration::planning) fn planning_counters(&self) -> RecoveryPlanningCounters {
        counters::after_fates(
            &self.sample,
            &self.fates,
            self.redo.counters(),
            self.observed_pages
                .artifact_reads
                .saturating_add(self.observed_pages.candidate_artifact_reads)
                .saturating_add(self.observed_pages.source_copy_reads)
                .saturating_add(self.observed_pages.historical_publication_reads),
            self.observed_pages
                .bytes_read
                .saturating_add(self.observed_pages.candidate_bytes_read)
                .saturating_add(self.observed_pages.source_copy_bytes_read)
                .saturating_add(self.observed_pages.historical_publication_bytes_read),
        )
        .with_successor_candidate_observation(
            self.observed_pages.candidate_artifact_reads,
            self.observed_pages.candidate_bytes_read,
            self.observed_pages.candidate_peak_materialization_bytes,
        )
        .with_page_extent_integrity(
            self.observed_pages.integrity.attempted(),
            self.observed_pages.integrity.admitted(),
            self.observed_pages.integrity.rejected(),
            self.observed_pages.integrity.owner_projection_entries(),
            self.observed_pages.integrity.owner_decoder_entries(),
        )
        .with_peak_recovery_bytes(self.capture_peak)
    }
}
