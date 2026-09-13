use worth_store::physical_runtime::{
    PhysicalRecoveryPublicationCandidate, PhysicalRecoveryPublicationCommand,
    PhysicalRecoveryPublicationCommandOutcome,
};

use crate::entry::{
    PhysicalRecoveryOutcome, PhysicalRecoveryPublicationCounters,
    PhysicalRecoveryPublicationSettlement,
};
use crate::progression::{NamespaceDurablePhysicalRecovery, StagedPhysicalRecovery};

mod counters;
mod state;

use counters::{completed_counters, denial_counters, indeterminate_counters};
use state::PublicationState;

pub(crate) fn publish_recovery(
    staged: StagedPhysicalRecovery,
) -> Result<NamespaceDurablePhysicalRecovery, PhysicalRecoveryOutcome> {
    let (state, publication) = PublicationState::from_staged(staged);
    let planned_effects = state.planned_effects();
    let (expectation, candidates) = publication.into_command_parts();
    let Some(candidates) = materialize_candidates(candidates) else {
        return Err(state.block_invalid());
    };
    if candidates.is_empty() {
        return state.publish_preexisting(expectation);
    }
    let Some(command) = PhysicalRecoveryPublicationCommand::new(
        expectation.plan_identity(),
        expectation.staging_generation(),
        candidates,
        expectation.root_protocol(),
    ) else {
        return Err(state.block_invalid());
    };
    match state.execute(command) {
        PhysicalRecoveryPublicationCommandOutcome::Completed(completed) => {
            let counters = completed_counters(planned_effects, &completed);
            let settlement = PhysicalRecoveryPublicationSettlement::Completed(completed);
            if state.is_ready() {
                Ok(state.into_namespace(expectation, counters, settlement))
            } else {
                Err(state.indeterminate(counters, settlement))
            }
        }
        PhysicalRecoveryPublicationCommandOutcome::DeniedBeforeEffect(denial) => {
            let counters = denial_counters(planned_effects, &denial);
            let escaped = !denial.candidates().is_empty()
                || denial.candidate_materialization().is_some()
                || denial.root_protocol().is_some();
            let settlement = PhysicalRecoveryPublicationSettlement::DeniedBeforeEffect(denial);
            if escaped {
                Err(state.indeterminate(counters, settlement))
            } else {
                Err(state.block_settlement(counters, settlement))
            }
        }
        PhysicalRecoveryPublicationCommandOutcome::Indeterminate(outcome) => {
            let counters = indeterminate_counters(planned_effects, &outcome);
            Err(state.indeterminate(
                counters,
                PhysicalRecoveryPublicationSettlement::Indeterminate(outcome),
            ))
        }
    }
}

fn materialize_candidates(
    candidates: Box<[crate::progression::RecoveryPublicationCandidateArtifact]>,
) -> Option<Box<[PhysicalRecoveryPublicationCandidate]>> {
    candidates
        .into_vec()
        .into_iter()
        .map(|candidate| {
            let (artifact, bytes, digest) = candidate.into_command_parts();
            PhysicalRecoveryPublicationCandidate::new(artifact, bytes, digest)
        })
        .collect::<Option<Vec<_>>>()
        .map(Vec::into_boxed_slice)
}
