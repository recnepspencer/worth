use crate::entry::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension, PhysicalRecoveryLimitFailure,
    PhysicalRecoveryOutcome, PhysicalRecoverySuccessorCandidateDenial,
};
use crate::orchestration::recovery_budget::RecoveryAllowance;
use crate::progression::{
    derive_execution_basis, requires_successor_candidate, CandidateMaterializationCost,
    ExecutionBasisDenial, PlanningMemoryDenial, PlanningResidentAllowance, RecoveryPublicationPlan,
    RecoveryQuiescencePlan, RecoveryStagingLayoutPlan,
};

use super::super::context::PlanningContext;
use super::super::manifest_entry_budget::ManifestEntryBudget;
use super::super::resolved_basis::ResolvedPlanningBasis;
use super::super::successor_candidate_observation;

#[cfg(test)]
#[path = "execution_basis/candidate_limit_tests.rs"]
mod candidate_limit_tests;

pub(super) struct ExecutionProducts {
    pub(super) staging: RecoveryStagingLayoutPlan,
    pub(super) publication: RecoveryPublicationPlan,
    pub(super) quiescence: RecoveryQuiescencePlan,
    pub(super) candidate_materialization: CandidateMaterializationCost,
    pub(super) planning_construction_peak: u64,
}

pub(super) fn derive(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
) -> Result<(PlanningContext, ExecutionProducts), PhysicalRecoveryOutcome> {
    let retained_memory_bytes = super::super::resident_memory::live_bytes(&context, basis)
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<ExecutionProducts>() as u64))
        .and_then(|bytes| {
            bytes.checked_add(std::mem::size_of::<
                Option<crate::progression::RecoveryObservedSuccessorCandidate>,
            >() as u64)
        })
        .and_then(|bytes| {
            bytes.checked_add(std::mem::size_of::<PlanningResidentAllowance>() as u64)
        });
    // A live size past every count is no limit: no count states it.
    let Some(retained_memory_bytes) = retained_memory_bytes else {
        return Err(context.cost_denial_block(
            basis.planning_counters(),
            worth_store_recovery_physics::RecoveryPlanCostDenial::RecoveryMemoryBytes,
            None,
        ));
    };
    let mut allowance = match PlanningResidentAllowance::new(
        retained_memory_bytes,
        context.limits.recovery_memory_bytes,
    ) {
        Ok(allowance) => allowance,
        Err(PlanningMemoryDenial::RecoveryMemoryBytes { observed }) => {
            let limit = RecoveryAllowance::declared(
                &context.limits,
                PhysicalRecoveryLimitDimension::RecoveryMemoryBytes,
            )
            .past(observed);
            return Err(context.cost_denial_block(
                basis.planning_counters().with_peak_recovery_bytes(observed),
                worth_store_recovery_physics::RecoveryPlanCostDenial::RecoveryMemoryBytes,
                limit,
            ));
        }
        Err(PlanningMemoryDenial::Allocation { .. }) => {
            unreachable!("seeding live storage does not allocate")
        }
    };
    let Some(historical_consumed) = basis.historical_consumed.as_ref() else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let candidate_required = match requires_successor_candidate(
        &context.selection,
        &basis.fates,
        &basis.redo,
        historical_consumed,
    ) {
        Ok(required) => required,
        Err(_) => return Err(context.redo_block(basis.planning_counters(), None)),
    };
    let successor_candidate = if candidate_required {
        let remaining_observation_bytes = context
            .limits
            .observation_bytes
            .saturating_sub(context.counters.bytes_observed)
            .saturating_sub(basis.observed_pages.bytes_read)
            .saturating_sub(basis.observed_pages.source_copy_bytes_read)
            .saturating_sub(basis.observed_pages.historical_publication_bytes_read);
        let media = context.authority.media;
        let selected = context.selection.root().selected();
        let (media, attempt) = successor_candidate_observation::observe(
            media,
            selected.manifest(),
            selected.selector().format(),
            &mut basis.observed_pages.manifest_budget,
            remaining_observation_bytes,
            &mut context.integrity_trace,
            &mut allowance,
        );
        context.authority.media = media;
        basis.observed_pages.candidate_artifact_reads = attempt.artifact_reads;
        basis.observed_pages.candidate_bytes_read = attempt.bytes_read;
        basis.observed_pages.candidate_peak_materialization_bytes =
            attempt.peak_materialization_bytes;
        basis.observed_pages.successor_root_integrity_admissions = attempt
            .root_protocol_counters
            .successor_root_integrity_admissions();
        basis.observed_pages.successor_root_interpretations = attempt
            .root_protocol_counters
            .successor_root_interpretations();
        context.record_successor_root_route(attempt.root_protocol_counters);
        match attempt.result {
            Ok(candidate) => candidate,
            Err(denial) => {
                let budget = &basis.observed_pages.manifest_budget;
                let limit = candidate_limit(&context.limits, &denial, budget, &allowance);
                let required_peak = match &denial {
                    PhysicalRecoverySuccessorCandidateDenial::RecoveryMemoryBytes { .. } => {
                        allowance.peak().max(allowance.refused().unwrap_or(0))
                    }
                    _ => allowance.peak(),
                };
                let artifact = format!("{:?}", denial.artifact());
                return Err(context.successor_candidate_block(
                    basis
                        .planning_counters()
                        .with_peak_recovery_bytes(required_peak),
                    &artifact,
                    limit,
                    denial,
                ));
            }
        }
    } else {
        None
    };
    let (
        staging,
        publication,
        quiescence,
        candidate_materialization,
        root_protocol_counters,
        planning_construction_peak,
    ) = match derive_execution_basis(
        context.authority.media.store_identity(),
        &context.selection,
        &basis.sample,
        &basis.fates,
        &basis.redo,
        historical_consumed,
        &basis.observed_pages.selected_source,
        &basis.verified_drops,
        basis.validated_manifest_cleanup,
        match &basis.custody {
            crate::progression::PlanningCustody::PendingPrepared { replay, .. } => Some(replay),
            _ => None,
        },
        successor_candidate,
        context.limits.manifest_entries,
        context.limits.staging_bytes,
        context.limits.dirty_frames,
        &mut allowance,
    ) {
        Ok(execution) => execution,
        Err(ExecutionBasisDenial::PublicationCandidateAllocation {
            requested_bytes,
            cause,
        }) => {
            return Err(context.block_with_planning_attempt_denial(
                crate::entry::PhysicalRecoveryBlockKind::RedoPlanning,
                basis
                    .planning_counters()
                    .with_peak_recovery_bytes(allowance.peak()),
                "publication-candidate-allocation",
                None,
                crate::entry::PhysicalRecoveryPlanningDenial::PublicationCandidateAllocation {
                    requested_bytes,
                    cause,
                },
            ));
        }
        Err(ExecutionBasisDenial::ImageAllocation {
            requested_bytes,
            cause,
        }) => {
            return Err(context.block_with_planning_attempt_denial(
                crate::entry::PhysicalRecoveryBlockKind::RedoPlanning,
                basis.planning_counters(),
                "execution-image-allocation",
                None,
                crate::entry::PhysicalRecoveryPlanningDenial::ExecutionImageAllocation {
                    requested_bytes,
                    cause,
                },
            ));
        }
        Err(ExecutionBasisDenial::RecoveryMemoryBytes { observed }) => {
            let limit = RecoveryAllowance::declared(
                &context.limits,
                PhysicalRecoveryLimitDimension::RecoveryMemoryBytes,
            )
            .past(observed);
            return Err(context.cost_denial_block(
                basis.planning_counters().with_peak_recovery_bytes(observed),
                worth_store_recovery_physics::RecoveryPlanCostDenial::RecoveryMemoryBytes,
                limit,
            ));
        }
        Err(ExecutionBasisDenial::StagingBytes { observed }) => {
            let limit = RecoveryAllowance::declared(
                &context.limits,
                PhysicalRecoveryLimitDimension::StagingBytes,
            )
            .past(observed);
            return Err(context.cost_denial_block(
                basis.planning_counters(),
                worth_store_recovery_physics::RecoveryPlanCostDenial::StagingBytes,
                limit,
            ));
        }
        Err(ExecutionBasisDenial::DirtyFrames { observed }) => {
            let limit = RecoveryAllowance::declared(
                &context.limits,
                PhysicalRecoveryLimitDimension::DirtyFrames,
            )
            .past(observed);
            return Err(context.cost_denial_block(
                basis.planning_counters(),
                worth_store_recovery_physics::RecoveryPlanCostDenial::DirtyFrames,
                limit,
            ));
        }
        Err(ExecutionBasisDenial::SuccessorCandidate(denial)) => {
            let artifact = format!("{:?}", denial.artifact());
            return Err(context.block_with_planning_attempt_denial(
                crate::entry::PhysicalRecoveryBlockKind::PageAdmission,
                basis.planning_counters(),
                &artifact,
                None,
                crate::entry::PhysicalRecoveryPlanningDenial::SuccessorCandidate(denial),
            ));
        }
        Err(ExecutionBasisDenial::RootProtocol {
            artifact,
            denial,
            counters,
        }) => {
            context.record_staged_selector_route(counters);
            return Err(context.root_protocol_block(basis.planning_counters(), artifact, denial));
        }
        Err(ExecutionBasisDenial::Invalid) => {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
    };
    context.record_staged_selector_route(root_protocol_counters);
    Ok((
        context,
        ExecutionProducts {
            staging,
            publication,
            quiescence,
            candidate_materialization,
            planning_construction_peak,
        },
    ))
}

fn candidate_limit(
    limits: &PhysicalRecoveryLimitDeclaration,
    denial: &PhysicalRecoverySuccessorCandidateDenial,
    budget: &ManifestEntryBudget,
    allowance: &PlanningResidentAllowance,
) -> Option<PhysicalRecoveryLimitFailure> {
    use PhysicalRecoverySuccessorCandidateDenial as Candidate;
    match denial {
        // The candidate's window was handed its maximum of recovery's
        // memory, beside what recovery already held.
        Candidate::RecoveryMemoryBytes { .. } => {
            RecoveryAllowance::declared(limits, PhysicalRecoveryLimitDimension::RecoveryMemoryBytes)
                .beside(allowance.refused()?, allowance.maximum())
                .map(Into::into)
        }
        // An artifact that outgrew the ceiling of its own read is damage.
        Candidate::Discovery { failure, .. } => {
            super::historical_publication::discovery_failure(failure.clone()).limit(limits, budget)
        }
        Candidate::ManifestEntryLimit { .. } => budget.refused().map(Into::into),
        Candidate::Allocation { .. }
        | Candidate::MissingArtifact { .. }
        | Candidate::InvalidArtifact { .. }
        | Candidate::RootProtocol { .. }
        | Candidate::Conflict { .. } => None,
    }
}
