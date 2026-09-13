use crate::entry::{
    PhysicalRecoveryLimitDimension, PhysicalRecoveryLimitFailure, PhysicalRecoveryOutcome,
    PhysicalRecoverySuccessorCandidateDenial,
};
use crate::progression::{
    derive_execution_basis, requires_successor_candidate, CandidateMaterializationCost,
    ExecutionBasisDenial, RecoveryPublicationPlan, RecoveryQuiescencePlan,
    RecoveryStagingLayoutPlan,
};

use super::super::context::PlanningContext;
use super::super::resolved_basis::ResolvedPlanningBasis;
use super::super::successor_candidate_observation;

pub(super) struct ExecutionProducts {
    pub(super) staging: RecoveryStagingLayoutPlan,
    pub(super) publication: RecoveryPublicationPlan,
    pub(super) quiescence: RecoveryQuiescencePlan,
    pub(super) candidate_materialization: CandidateMaterializationCost,
}

pub(super) fn derive(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
) -> Result<(PlanningContext, ExecutionProducts), PhysicalRecoveryOutcome> {
    let candidate_required = match requires_successor_candidate(&basis.fates, &basis.redo) {
        Ok(required) => required,
        Err(_) => return Err(context.redo_block(basis.planning_counters(), None)),
    };
    let successor_candidate = if candidate_required {
        let remaining_observation_bytes = context
            .limits
            .observation_bytes
            .saturating_sub(context.counters.bytes_observed)
            .saturating_sub(basis.observed_pages.bytes_read);
        let media = context.authority.media;
        let (media, attempt) = successor_candidate_observation::observe(
            media,
            context.selection.root().selected().manifest(),
            context.selection.root().selected().selector().format(),
            &mut basis.observed_pages.manifest_budget,
            context.limits.manifest_entries,
            remaining_observation_bytes,
            &mut context.integrity_trace,
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
                let limit = candidate_limit(&context, &denial, remaining_observation_bytes);
                let artifact = format!("{:?}", denial.artifact());
                return Err(context.successor_candidate_block(
                    basis.planning_counters(),
                    &artifact,
                    limit,
                    denial,
                ));
            }
        }
    } else {
        None
    };
    let (staging, publication, quiescence, candidate_materialization, root_protocol_counters) =
        match derive_execution_basis(
            context.authority.media.store_identity(),
            &context.selection,
            &basis.sample,
            &basis.fates,
            &basis.redo,
            &basis.observed_pages.selected_source,
            successor_candidate,
            context.limits.staging_bytes,
            context.limits.dirty_frames,
        ) {
            Ok(execution) => execution,
            Err(ExecutionBasisDenial::StagingBytes { observed }) => {
                let admitted = context.limits.staging_bytes;
                return Err(context.cost_denial_block(
                    basis.planning_counters(),
                    worth_store_recovery_physics::RecoveryPlanCostDenial::StagingBytes,
                    PhysicalRecoveryLimitFailure {
                        dimension: PhysicalRecoveryLimitDimension::StagingBytes,
                        observed,
                        admitted,
                    },
                ));
            }
            Err(ExecutionBasisDenial::DirtyFrames { observed }) => {
                let admitted = context.limits.dirty_frames;
                return Err(context.cost_denial_block(
                    basis.planning_counters(),
                    worth_store_recovery_physics::RecoveryPlanCostDenial::DirtyFrames,
                    PhysicalRecoveryLimitFailure {
                        dimension: PhysicalRecoveryLimitDimension::DirtyFrames,
                        observed,
                        admitted,
                    },
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
                return Err(context.root_protocol_block(
                    basis.planning_counters(),
                    artifact,
                    denial,
                ));
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
        },
    ))
}

fn candidate_limit(
    context: &PlanningContext,
    denial: &PhysicalRecoverySuccessorCandidateDenial,
    remaining_observation_bytes: u64,
) -> Option<PhysicalRecoveryLimitFailure> {
    match denial {
        PhysicalRecoverySuccessorCandidateDenial::Discovery {
            failure:
                worth_store::physical_runtime::RecoveryDiscoveryFailure::ByteLimitExceeded {
                    observed,
                    ..
                },
            ..
        } => Some(PhysicalRecoveryLimitFailure {
            dimension: PhysicalRecoveryLimitDimension::ObservationBytes,
            observed: context
                .limits
                .observation_bytes
                .saturating_sub(remaining_observation_bytes)
                .saturating_add(*observed),
            admitted: context.limits.observation_bytes,
        }),
        PhysicalRecoverySuccessorCandidateDenial::ManifestEntryLimit {
            observed, admitted, ..
        } => Some(PhysicalRecoveryLimitFailure {
            dimension: PhysicalRecoveryLimitDimension::ManifestEntries,
            observed: *observed,
            admitted: *admitted,
        }),
        _ => None,
    }
}
