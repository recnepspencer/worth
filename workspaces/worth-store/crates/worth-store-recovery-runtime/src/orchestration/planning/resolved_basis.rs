use worth_store_recovery_physics::{
    reconcile_materialized_operation_fates, PhysicalRedoPlanCounters,
};

use crate::entry::PhysicalRecoveryOutcome;

use super::admitted_basis::AdmittedPlanningBasis;
use super::context::PlanningContext;
use super::counters;
use super::page_observation;

mod state;
mod storage;
pub(super) use state::{PageObservationResult, ResolvedPlanningState};
pub(super) use storage::ResolvedPlanningBasis;

pub(super) fn resolve(
    mut context: PlanningContext,
    admitted: AdmittedPlanningBasis,
) -> Result<(PlanningContext, ResolvedPlanningBasis), PhysicalRecoveryOutcome> {
    let remaining_observation_bytes = context
        .limits
        .observation_bytes
        .saturating_sub(context.counters.bytes_observed);
    let selected_wal = context
        .integrity
        .admitted_wal()
        .recoverable_frame_view(context.selection.wal_tail());
    let tier_evidence = page_observation::TierEvidence {
        selection: &context.selection,
        checkpoint: context
            .coordination
            .owner()
            .checkpoint()
            .map(|shared| shared.stream()),
        sample: &admitted.sample,
        selected_wal,
    };
    let media = context.authority.media;
    let (media, attempt) = page_observation::observe_selected_pages(
        media,
        tier_evidence,
        context.selection.root().selected().manifest(),
        context
            .selection
            .root()
            .retained_previous()
            .map(|previous| (previous.manifest(), previous.selector().format())),
        context.selection.page_facts().placements(),
        &admitted.redo,
        admitted.format,
        &context.limits,
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
            source_copy_reads: 0,
            source_copy_bytes_read: 0,
            source_copy_peak_scratch_bytes: 0,
            historical_publication_reads: 0,
            historical_publication_bytes_read: 0,
            historical_publication_peak_scratch_bytes: observed.ordered_history_peak_scratch_bytes,
            candidate_artifact_reads: 0,
            candidate_bytes_read: 0,
            candidate_peak_materialization_bytes: 0,
            successor_root_integrity_admissions: 0,
            successor_root_interpretations: 0,
            inline_truth: observed.inline_truth,
            selected_source: observed.selected_source,
            manifest_budget: attempt.manifest_budget,
            tier_custody: observed.tier_custody,
            historical_drops: observed.historical_drops,
            ordered_releases: observed.ordered_releases,
            integrity: attempt.integrity,
        },
        Err(denial) => {
            let (limit, page) = match denial.evidence() {
                Ok(page) => (None, Some(page)),
                Err(limit) => (limit.in_recovery(&context.limits), None),
            };
            return Err(context.page_block(
                planning_counters,
                "selected-source-inventory-and-pages",
                limit,
                page,
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
        Err(denial) => return Err(context.redo_denial_block(denial_counters, denial)),
    };
    let redo = match redo
        .admit_inline_allocation_truth(context.selection.page_facts().placements(), inline_truth)
    {
        Ok(redo) => redo,
        Err(denial) => return Err(context.redo_denial_block(denial_counters, denial)),
    };
    let fates = reconcile_materialized_operation_fates(admitted.fates, &redo);
    let state = ResolvedPlanningState {
        unused_backing_bytes: 0,
        capture_peak: 0,
        sample: admitted.sample,
        fates,
        redo,
        targets: admitted.targets,
        redo_bytes: admitted.redo_bytes,
        distinct_targets: admitted.distinct_targets,
        observed_pages,
        verified_drops: Default::default(),
        verified_historical_release_operations: Default::default(),
        historical_consumed: None,
        verified_historical_release_sources: Default::default(),
        custody: crate::progression::PlanningCustody::Unresolved,
        validated_manifest_cleanup: None,
    };
    let counters = state.planning_counters();
    let basis = match ResolvedPlanningBasis::capture(&context, state) {
        Ok(basis) => basis,
        Err(denial) => return Err(storage::deny(context, counters, denial)),
    };
    Ok((context, basis))
}
