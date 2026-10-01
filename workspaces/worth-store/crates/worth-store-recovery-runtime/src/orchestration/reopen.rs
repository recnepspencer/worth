use worth_store::physical_runtime::{
    PhysicalRecoveryFreshReopenCommand, PhysicalRecoveryFreshReopenOutcome,
};

use crate::entry::{
    PhysicalRecoveryOutcome, PhysicalRecoveryPublicationIndeterminate,
    PhysicalRecoveryReopenCounters, PhysicalRecoveryReopenFailure,
};
use crate::progression::{NamespaceDurablePhysicalRecovery, ReopenedPhysicalRecovery};

#[path = "reopen/custody_rebind.rs"]
mod custody_rebind;

pub(crate) fn reopen_recovery(
    durable: NamespaceDurablePhysicalRecovery,
) -> Result<ReopenedPhysicalRecovery, PhysicalRecoveryOutcome> {
    let NamespaceDurablePhysicalRecovery {
        state,
        mut expectation,
        publication_counters,
        publication_settlement,
    } = durable;
    let format = state.selection.root().selected().selector().format();
    let command = PhysicalRecoveryFreshReopenCommand::new(
        expectation.plan_identity(),
        expectation.recovered_root().clone(),
        expectation.current_selector(),
        format,
    )
    .expect("a sealed publication expectation has a nonzero recovered root");
    match state
        .coordination
        .owner()
        .execute_fresh_reopen(&state.authority.media, command)
    {
        PhysicalRecoveryFreshReopenOutcome::Completed(completed) => {
            let counters = completed_counters(&completed);
            let rebound_state = custody_rebind::rebind_selected_custody(
                state,
                &mut expectation,
                &publication_settlement,
                &publication_counters,
                &completed,
            );
            let state = match rebound_state {
                Ok(state) => state,
                Err(state) => {
                    return Err(custody_indeterminate(
                        state,
                        publication_counters,
                        publication_settlement,
                    ))
                }
            };
            Ok(ReopenedPhysicalRecovery::new(
                state,
                expectation,
                publication_counters,
                publication_settlement,
                completed,
                counters,
            ))
        }
        PhysicalRecoveryFreshReopenOutcome::Denied(denial) => {
            let counters = denial_counters(&denial);
            let failure = PhysicalRecoveryReopenFailure::new(counters, denial);
            Err(indeterminate(
                state,
                publication_counters,
                publication_settlement,
                failure,
            ))
        }
    }
}

fn custody_indeterminate(
    state: crate::progression::NamespaceDurableState,
    publication_counters: crate::entry::PhysicalRecoveryPublicationCounters,
    publication_settlement: crate::entry::PhysicalRecoveryPublicationSettlementLedger,
) -> PhysicalRecoveryOutcome {
    let store = state.authority.media.store_identity();
    let session = state.authority.session.identity();
    let recovery_effects = state.authority.media.recovery_effect_count();
    let crate::entry::AdmittedPlatformAuthority {
        media,
        session: session_authority,
        ..
    } = state.authority;
    drop(media);
    session_authority.publication_indeterminate();
    PhysicalRecoveryOutcome::PublicationIndeterminate(
        PhysicalRecoveryPublicationIndeterminate::new(
            store,
            session,
            publication_counters,
            publication_settlement,
            state.root_protocol_denials,
            state.root_protocol_counters,
            recovery_effects,
        )
        .with_integrity_observations(state.integrity.into_observations())
        .with_integrity_trace(state.integrity_trace)
        .with_handoff_failure(
            worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch,
        ),
    )
}

fn completed_counters(
    completed: &worth_store::physical_runtime::CompletedPhysicalRecoveryFreshReopen,
) -> PhysicalRecoveryReopenCounters {
    let occurrence = completed.fresh_reopen_occurrence();
    PhysicalRecoveryReopenCounters {
        selector_reads_completed: 1,
        root_reads_completed: 1,
        bytes_read: occurrence.selector().bytes().len() as u64
            + occurrence.root().bytes().len() as u64,
    }
}

fn denial_counters(
    denial: &worth_store::physical_runtime::PhysicalRecoveryFreshReopenDenial,
) -> PhysicalRecoveryReopenCounters {
    PhysicalRecoveryReopenCounters {
        selector_reads_completed: u64::from(denial.selector().is_some()),
        root_reads_completed: u64::from(denial.root().is_some()),
        bytes_read: denial
            .selector()
            .map_or(0, |read| read.bytes().len() as u64)
            + denial.root().map_or(0, |read| read.bytes().len() as u64),
    }
}

fn indeterminate(
    state: crate::progression::NamespaceDurableState,
    publication_counters: crate::entry::PhysicalRecoveryPublicationCounters,
    publication_settlement: crate::entry::PhysicalRecoveryPublicationSettlementLedger,
    failure: PhysicalRecoveryReopenFailure,
) -> PhysicalRecoveryOutcome {
    assert!(state.coordination.shutdown_is_quiescent());
    let store = state.authority.media.store_identity();
    let session = state.authority.session.identity();
    let recovery_effects = state.authority.media.recovery_effect_count();
    let crate::entry::AdmittedPlatformAuthority {
        media,
        session: session_authority,
        ..
    } = state.authority;
    drop(media);
    session_authority.publication_indeterminate();
    PhysicalRecoveryOutcome::PublicationIndeterminate(
        PhysicalRecoveryPublicationIndeterminate::new(
            store,
            session,
            publication_counters,
            publication_settlement,
            state.root_protocol_denials,
            state.root_protocol_counters,
            recovery_effects,
        )
        .with_integrity_observations(state.integrity.into_observations())
        .with_reopen_failure(failure),
    )
}
