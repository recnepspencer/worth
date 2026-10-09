use worth_store::physical_runtime::RecoveryCheckpointResidueOutcome;

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryBlock, PhysicalRecoveryBlockEvidence,
    PhysicalRecoveryBlockKind, PhysicalRecoveryOutcome, PhysicalRecoveryPublicationIndeterminate,
};
use crate::progression::ReopenedPhysicalRecovery;

/// This cleanup is not WAL reclamation. It removes only the unselected
/// `selected checkpoint sequence + 1` staging candidate under the same
/// recovery media owner, before closing the fresh-reopen authority.
pub(super) fn prepare(reopened: &ReopenedPhysicalRecovery) -> RecoveryCheckpointResidueOutcome {
    let Some(checkpoint) = reopened.state.coordination.owner().checkpoint() else {
        return RecoveryCheckpointResidueOutcome::Absent;
    };
    let maximum = reopened
        .state
        .authority
        .limits
        .declaration()
        .cleanup_bytes
        .min(
            reopened
                .state
                .authority
                .limits
                .declaration()
                .observation_bytes,
        );
    reopened
        .state
        .coordination
        .owner()
        .remove_unselected_checkpoint_candidate(
            &reopened.state.authority.media,
            checkpoint,
            maximum,
        )
}

pub(super) fn failure(
    reopened: ReopenedPhysicalRecovery,
    outcome: RecoveryCheckpointResidueOutcome,
) -> PhysicalRecoveryOutcome {
    let ReopenedPhysicalRecovery {
        state,
        publication_counters,
        publication_settlement,
        ..
    } = reopened;
    let store = state.authority.media.store_identity();
    let session_identity = state.authority.session.identity();
    let recovery_effects = state.authority.media.recovery_effect_count();
    let quiescent = state.coordination.into_owner().shutdown_is_quiescent();
    let AdmittedPlatformAuthority { media, session, .. } = state.authority;
    drop(media);
    match outcome {
        RecoveryCheckpointResidueOutcome::DeniedBeforeEffect(denial) if quiescent => {
            session.block();
            PhysicalRecoveryOutcome::Blocked(PhysicalRecoveryBlock::new(
                crate::entry::PhysicalRecoveryBlockCause::Damage(
                    PhysicalRecoveryBlockKind::Checkpoint,
                ),
                store,
                session_identity,
                PhysicalRecoveryBlockEvidence {
                    counters: state.discovery_counters,
                    planning_counters: Some(state.planning_counters),
                    root_protocol_counters: Some(state.root_protocol_counters),
                    source_denials: state.root_protocol_denials,
                    integrity_observations: state.integrity.into_observations(),
                    staging_counters: Some(state.staging_counters),
                    staging_settlements: Some(state.staging_settlements),
                    publication_counters: Some(publication_counters),
                    publication_settlements: Some(publication_settlement),
                    checkpoint_residue_denial: Some(denial),
                    integrity_trace: state.integrity_trace,
                    ..PhysicalRecoveryBlockEvidence::default()
                },
                recovery_effects,
            ))
        }
        RecoveryCheckpointResidueOutcome::DeniedBeforeEffect(_)
        | RecoveryCheckpointResidueOutcome::Indeterminate => {
            session.publication_indeterminate();
            PhysicalRecoveryOutcome::PublicationIndeterminate(
                PhysicalRecoveryPublicationIndeterminate::new(
                    store,
                    session_identity,
                    publication_counters,
                    publication_settlement,
                    state.root_protocol_denials,
                    state.root_protocol_counters,
                    recovery_effects,
                )
                .with_integrity_trace(state.integrity_trace)
                .with_integrity_observations(state.integrity.into_observations())
                .with_checkpoint_residue_indeterminate(),
            )
        }
        RecoveryCheckpointResidueOutcome::Absent
        | RecoveryCheckpointResidueOutcome::Removed { .. } => {
            unreachable!("settled checkpoint residue does not fail recovery")
        }
    }
}
