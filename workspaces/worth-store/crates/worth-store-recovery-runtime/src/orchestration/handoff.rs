use crate::entry::{PhysicalRecoveryOutcome, PhysicalRecoveryPublicationIndeterminate};
use crate::handoff::{RecoveredPhysicalRuntimeHandoff, RecoveredPhysicalRuntimeHandoffEvidence};
use crate::progression::ReopenedPhysicalRecovery;

pub(crate) fn finish_recovery_after_cleanup(
    reopened: ReopenedPhysicalRecovery,
    closed_cleanup: worth_store::physical_runtime::ClosedPhysicalRecoveryCleanup,
    cleanup: crate::handoff::RecoveryCleanupPosture,
) -> PhysicalRecoveryOutcome {
    let ReopenedPhysicalRecovery {
        state,
        expectation,
        publication_counters,
        publication_settlement,
        reopened: pending_reopen,
        reopen_counters,
    } = reopened;
    debug_assert!(pending_reopen.is_none());
    let store = state.authority.media.store_identity();
    let session_identity = state.authority.session.identity();
    let recovery_effects = state.authority.media.recovery_effect_count();
    let crate::entry::AdmittedPlatformAuthority { media, session, .. } = state.authority;
    let coordination = state.coordination.into_owner();
    let construction = worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct(
        coordination,
        media,
        closed_cleanup,
    );
    match construction {
        Ok(core) => {
            let session = session.recovered();
            PhysicalRecoveryOutcome::Recovered(RecoveredPhysicalRuntimeHandoff::new(
                core,
                RecoveredPhysicalRuntimeHandoffEvidence {
                    session,
                    selection: state.selection,
                    discovery: state.discovery_counters,
                    root_protocol_denials: state.root_protocol_denials,
                    integrity_observations: state.integrity.into_observations(),
                    freshness: state.freshness,
                    fates: state.fates,
                    planning: state.planning_counters,
                    root_protocol_counters: state.root_protocol_counters,
                    base: state.base,
                    quiescence: state.quiescence,
                    closed: state.closed,
                    staging: state.staging_counters,
                    staging_settlements: state.staging_settlements,
                    publication_expectation: expectation,
                    publication: publication_counters,
                    publication_settlement,
                    reopen: reopen_counters,
                    cleanup,
                    integrity_trace: state.integrity_trace,
                },
            ))
        }
        Err(denial) => {
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
                .with_handoff_failure(denial),
            )
        }
    }
}
