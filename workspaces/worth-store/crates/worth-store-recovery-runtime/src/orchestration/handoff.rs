use crate::entry::{PhysicalRecoveryOutcome, PhysicalRecoveryPublicationIndeterminate};
use crate::handoff::{RecoveredPhysicalRuntimeHandoff, RecoveredPhysicalRuntimeHandoffEvidence};
use crate::progression::ReopenedPhysicalRecovery;
mod construction;
#[path = "handoff/resident_memory.rs"]
mod resident_memory;

pub(crate) fn finish_recovery_after_cleanup(
    mut reopened: ReopenedPhysicalRecovery,
    closed_cleanup: worth_store::physical_runtime::ClosedPhysicalRecoveryCleanup,
    cleanup: crate::handoff::RecoveryCleanupPosture,
) -> PhysicalRecoveryOutcome {
    reopened.release_admitted_wal();
    let retained = resident_memory::retained_bytes(&reopened, &cleanup)
        .and_then(|heap| heap.checked_add(resident_memory::inline_bytes(&reopened, &cleanup)?))
        .and_then(|bytes| bytes.checked_add(closed_cleanup.owned_heap_bytes()?))
        .and_then(|bytes| bytes.checked_add(std::mem::size_of_val(&closed_cleanup) as u64));
    let crate::progression::ReopenedPayload {
        state,
        expectation,
        publication_counters,
        publication_settlement,
        reopened: pending_reopen,
        reopen_counters,
    } = reopened.into_payload();
    debug_assert!(pending_reopen.is_none());
    let store = state.authority.media.store_identity();
    let session_identity = state.authority.session.identity();
    let recovery_effects = state.authority.media.recovery_effect_count();
    let crate::entry::AdmittedPlatformAuthority {
        media,
        session,
        durability,
        ..
    } = state.authority;
    let (mut coordination, absent_checkpoint) = state.coordination.into_parts();
    let resident_admission = retained.ok_or(
        worth_store::physical_runtime::PhysicalRecoveryRejoinResidentAdmissionDenial::SizeOverflow,
    ).and_then(|bytes| coordination.admit_rejoin_resident_bytes(bytes));
    let recovery_allocation = coordination.recovery_allocation_admission();
    let requires_allocation = matches!(
        &state.custody,
        crate::progression::CustodyState::SourceHeads(_)
            | crate::progression::CustodyState::Pending { .. }
            | crate::progression::CustodyState::OrderedCompleted { .. }
    );
    let construction = if let Err(denial) = resident_admission {
        Err(worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::ResidentAdmission(denial))
    } else if requires_allocation && recovery_allocation.is_none() {
        Err(worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch)
    } else {
        construction::RejoinConstruction {
            coordination,
            media,
            closed_cleanup,
        }
        .complete(
            state.custody,
            state.verified_selected_tier_custody,
            recovery_allocation,
            absent_checkpoint,
            durability,
            &state.selection,
        )
    };
    match construction {
        Ok(core) => {
            let session = session.recovered();
            PhysicalRecoveryOutcome::Recovered(Box::new(RecoveredPhysicalRuntimeHandoff::new(
                core,
                RecoveredPhysicalRuntimeHandoffEvidence {
                    store_rejoin_retained_bytes: retained,
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
            )))
        }
        Err(denial) => {
            session.publication_indeterminate();
            PhysicalRecoveryOutcome::PublicationIndeterminate(Box::new(
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
                .with_handoff_failure(denial)
                .with_cleanup_posture(cleanup),
            ))
        }
    }
}
