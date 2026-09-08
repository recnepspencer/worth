use worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample;
use worth_store_recovery_physics::{
    reconcile_materialized_operation_fates, ImmutablePhysicalRedoPlan, PhysicalRedoPlanCounters,
    PhysicalRedoTargetIdentity, ReconciledOperationFates, RecoveryPlanningCounters,
};

use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension, PhysicalRecoveryLimitFailure,
    PhysicalRecoveryOutcome, PhysicalRecoveryPlanningDenial,
};

use super::admitted_basis::AdmittedPlanningBasis;
use super::context::PlanningContext;
use super::counters;
use super::page_observation::{self, PageObservationFailure};

pub(super) struct PageObservationResult {
    pub(super) observations: Vec<worth_store_recovery_physics::RecoveryPageObservation>,
    pub(super) artifact_reads: u64,
    pub(super) bytes_read: u64,
    pub(super) candidate_artifact_reads: u64,
    pub(super) candidate_bytes_read: u64,
    pub(super) candidate_peak_materialization_bytes: u64,
    pub(super) successor_root_integrity_admissions: u64,
    pub(super) successor_root_interpretations: u64,
    pub(super) inline_truth: Option<super::page_observation::InlineAllocationTruth>,
    pub(super) selected_source: crate::progression::RecoverySelectedSourceInventory,
    pub(super) manifest_budget: super::manifest_entry_budget::ManifestEntryBudget,
    pub(super) integrity: crate::integrity_ingress::RecoveryIntegrityIngressCounters,
}

pub(super) struct ResolvedPlanningBasis {
    pub(super) sample: StoreRecoveryBindingFreshnessSample,
    pub(super) fates: ReconciledOperationFates,
    pub(super) redo: ImmutablePhysicalRedoPlan,
    pub(super) targets: Box<[PhysicalRedoTargetIdentity]>,
    pub(super) redo_bytes: u64,
    pub(super) distinct_targets: u64,
    pub(super) observed_pages: PageObservationResult,
}

impl ResolvedPlanningBasis {
    pub(super) fn planning_counters(&self) -> RecoveryPlanningCounters {
        counters::after_fates(
            &self.sample,
            &self.fates,
            self.redo.counters(),
            self.observed_pages
                .artifact_reads
                .saturating_add(self.observed_pages.candidate_artifact_reads),
            self.observed_pages
                .bytes_read
                .saturating_add(self.observed_pages.candidate_bytes_read),
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
    }
}

pub(super) fn resolve(
    mut context: PlanningContext,
    admitted: AdmittedPlanningBasis,
) -> Result<(PlanningContext, ResolvedPlanningBasis), PhysicalRecoveryOutcome> {
    let observation_targets = admitted.redo.observation_targets();
    let read_ceiling = match page_observation::artifact_read_ceiling(
        context.selection.page_facts().placements(),
        &observation_targets,
        admitted.remaining_manifest_entries,
        context.selection.root().retained_previous().is_some(),
    ) {
        Ok(ceiling) => ceiling,
        Err(page_observation::ArtifactReadCeilingDenial::ManifestEntriesExhausted) => {
            let admitted_entries = context.limits.manifest_entries;
            let planning_counters = counters::after_fates(
                &admitted.sample,
                &admitted.fates,
                PhysicalRedoPlanCounters::default(),
                0,
                0,
            );
            return Err(context.block_with_planning_attempt_denial(
                PhysicalRecoveryBlockKind::PageAdmission,
                planning_counters,
                "selected-source-inventory",
                Some(PhysicalRecoveryLimitFailure {
                    dimension: PhysicalRecoveryLimitDimension::ManifestEntries,
                    observed: admitted_entries.saturating_add(1),
                    admitted: admitted_entries,
                }),
                PhysicalRecoveryPlanningDenial::Page(
                    PageObservationFailure::ManifestEntryLimit.evidence(),
                ),
            ));
        }
        Err(page_observation::ArtifactReadCeilingDenial::Overflow) => {
            let admitted_entries = context.limits.manifest_entries;
            let planning_counters = counters::after_fates(
                &admitted.sample,
                &admitted.fates,
                PhysicalRedoPlanCounters::default(),
                0,
                0,
            );
            return Err(context.block_with_planning_attempt_denial(
                PhysicalRecoveryBlockKind::PageAdmission,
                planning_counters,
                "selected-source-inventory",
                Some(PhysicalRecoveryLimitFailure {
                    dimension: PhysicalRecoveryLimitDimension::ManifestEntries,
                    observed: u64::MAX,
                    admitted: admitted_entries,
                }),
                PhysicalRecoveryPlanningDenial::Page(
                    PageObservationFailure::ManifestEntryLimit.evidence(),
                ),
            ));
        }
    };
    let remaining_observation_bytes = context
        .limits
        .observation_bytes
        .saturating_sub(context.counters.bytes_observed);
    if remaining_observation_bytes == 0 {
        let admitted_bytes = context.limits.observation_bytes;
        let planning_counters = counters::after_fates(
            &admitted.sample,
            &admitted.fates,
            PhysicalRedoPlanCounters::default(),
            0,
            0,
        );
        return Err(context.block_with_planning_attempt_denial(
            PhysicalRecoveryBlockKind::PageAdmission,
            planning_counters,
            "selected-source-inventory",
            Some(PhysicalRecoveryLimitFailure {
                dimension: PhysicalRecoveryLimitDimension::ObservationBytes,
                observed: admitted_bytes.saturating_add(1),
                admitted: admitted_bytes,
            }),
            PhysicalRecoveryPlanningDenial::Page(PageObservationFailure::ByteLimit.evidence()),
        ));
    }
    let media = context.authority.media;
    let (media, attempt) = page_observation::observe_selected_pages(
        media,
        context.selection.root().selected().manifest(),
        context
            .selection
            .root()
            .retained_previous()
            .map(|previous| (previous.manifest(), previous.selector().format())),
        context.selection.page_facts().placements(),
        &admitted.redo,
        admitted.format,
        read_ceiling.addressed_reads,
        context.limits.manifest_entries,
        admitted.remaining_manifest_entries,
        remaining_observation_bytes,
        &mut context.integrity_trace,
    );
    context.authority.media = media;
    let planning_counters = counters::after_fates(
        &admitted.sample,
        &admitted.fates,
        PhysicalRedoPlanCounters::default(),
        attempt.artifact_reads,
        attempt.bytes_read,
    )
    .with_page_extent_integrity(
        attempt.integrity.attempted(),
        attempt.integrity.admitted(),
        attempt.integrity.rejected(),
        attempt.integrity.owner_projection_entries(),
        attempt.integrity.owner_decoder_entries(),
    );
    let mut observed_pages = match attempt.result {
        Ok(observed) => PageObservationResult {
            observations: observed.observations,
            artifact_reads: attempt.artifact_reads,
            bytes_read: attempt.bytes_read,
            candidate_artifact_reads: 0,
            candidate_bytes_read: 0,
            candidate_peak_materialization_bytes: 0,
            successor_root_integrity_admissions: 0,
            successor_root_interpretations: 0,
            inline_truth: observed.inline_truth,
            selected_source: observed.selected_source,
            manifest_budget: observed.manifest_budget,
            integrity: attempt.integrity,
        },
        Err(denial) => {
            let limit = observation_limit(&context, &denial, remaining_observation_bytes);
            return Err(context.block_with_planning_attempt_denial(
                PhysicalRecoveryBlockKind::PageAdmission,
                planning_counters,
                "selected-source-inventory-and-pages",
                limit,
                PhysicalRecoveryPlanningDenial::Page(denial.evidence()),
            ));
        }
    };
    let observations = std::mem::take(&mut observed_pages.observations);
    let denial_counters = counters::after_fates(
        &admitted.sample,
        &admitted.fates,
        PhysicalRedoPlanCounters::default(),
        observed_pages.artifact_reads,
        observed_pages.bytes_read,
    )
    .with_page_extent_integrity(
        observed_pages.integrity.attempted(),
        observed_pages.integrity.admitted(),
        observed_pages.integrity.rejected(),
        observed_pages.integrity.owner_projection_entries(),
        observed_pages.integrity.owner_decoder_entries(),
    );
    let inline_truth = observed_pages
        .inline_truth
        .map(|truth| (truth.next_segment, truth.page_capacity));
    let redo = match admitted.redo.plan(observations) {
        Ok(plan) => plan,
        Err(denial) => return Err(context.redo_denial_block(denial_counters, None, denial)),
    };
    let redo = match redo
        .admit_inline_allocation_truth(context.selection.page_facts().placements(), inline_truth)
    {
        Ok(redo) => redo,
        Err(denial) => return Err(context.redo_denial_block(denial_counters, None, denial)),
    };
    let fates = reconcile_materialized_operation_fates(admitted.fates, &redo);
    Ok((
        context,
        ResolvedPlanningBasis {
            sample: admitted.sample,
            fates,
            redo,
            targets: admitted.targets,
            redo_bytes: admitted.redo_bytes,
            distinct_targets: admitted.distinct_targets,
            observed_pages,
        },
    ))
}

fn observation_limit(
    context: &PlanningContext,
    denial: &PageObservationFailure,
    remaining_observation_bytes: u64,
) -> Option<PhysicalRecoveryLimitFailure> {
    match denial {
        PageObservationFailure::ByteLimit => Some(PhysicalRecoveryLimitFailure {
            dimension: PhysicalRecoveryLimitDimension::ObservationBytes,
            observed: context
                .counters
                .bytes_observed
                .saturating_add(remaining_observation_bytes)
                .saturating_add(1),
            admitted: context.limits.observation_bytes,
        }),
        PageObservationFailure::ManifestEntryLimit => Some(PhysicalRecoveryLimitFailure {
            dimension: PhysicalRecoveryLimitDimension::ManifestEntries,
            observed: context.limits.manifest_entries.saturating_add(1),
            admitted: context.limits.manifest_entries,
        }),
        _ => None,
    }
}
