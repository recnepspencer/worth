use worth_store::physical_runtime::{
    PhysicalRecoveryPublicationCandidate, PhysicalRecoveryPublicationCommand,
    PhysicalRecoveryPublicationCommandOutcome,
};

use crate::entry::PhysicalRecoveryOutcome;
use crate::progression::{NamespaceDurablePhysicalRecovery, StagedPhysicalRecovery};

mod catalog_repair;
mod counters;
mod outcome;
mod state;

use state::PublicationState;
pub(crate) use state::{PublicationPayload, PublicationStateBacking};

pub(crate) fn publish_recovery(
    staged: StagedPhysicalRecovery,
) -> Result<NamespaceDurablePhysicalRecovery, PhysicalRecoveryOutcome> {
    let (state, publication) = PublicationState::from_staged(staged);
    let planned_effects = state.planned_effects();
    let (expectation, candidates) = publication.into_command_parts();
    let Some(mut candidates) = materialize_candidates(candidates) else {
        return Err(state.block_invalid());
    };
    let mut protocol = expectation.root_protocol();
    let mut staging_generation = expectation.staging_generation();
    if candidates.is_empty() {
        match catalog_repair::plan(state.media(), &expectation, state.prior_effects_unproven()) {
            Ok(None) => return state.publish_preexisting(expectation),
            Ok(Some(repair)) => {
                candidates = repair.candidates;
                protocol = repair.protocol;
                staging_generation = repair.generation;
            }
            Err(()) => return Err(state.block_invalid()),
        }
    }
    let Some(command) = PhysicalRecoveryPublicationCommand::new(
        expectation.plan_identity(),
        staging_generation,
        candidates,
        protocol,
    ) else {
        return Err(state.block_invalid());
    };
    match state.execute(command) {
        PhysicalRecoveryPublicationCommandOutcome::Completed(completed) => {
            outcome::completed(state, expectation, planned_effects, completed)
        }
        PhysicalRecoveryPublicationCommandOutcome::DeniedBeforeEffect(denial) => {
            Err(outcome::denied(state, planned_effects, denial))
        }
        PhysicalRecoveryPublicationCommandOutcome::Indeterminate(result) => {
            Err(outcome::indeterminate(state, planned_effects, result))
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
