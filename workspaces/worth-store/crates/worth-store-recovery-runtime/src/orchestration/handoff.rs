//! Prepaid handoff staging, construction and final publication occupy distinct frames.
use crate::entry::PhysicalRecoveryOutcome;
use crate::progression::ReopenedPhysicalRecovery;
mod backing;
mod completion;
mod construction;
mod resident_memory;
pub(crate) use backing::RecoveryHandoffBacking;

pub(crate) fn finish_recovery_after_cleanup(
    mut reopened: ReopenedPhysicalRecovery,
    closed_cleanup: worth_store::physical_runtime::ClosedPhysicalRecoveryCleanup,
    cleanup: crate::handoff::RecoveryCleanupPosture,
) -> PhysicalRecoveryOutcome {
    reopened.release_admitted_wal();
    let prepared = prepare(reopened, closed_cleanup, cleanup);
    let constructed = construct(prepared);
    publish(constructed)
}
struct PreparedHandoff {
    input: Result<
        construction::RejoinConstructionInput,
        worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial,
    >,
    backing: RecoveryHandoffBacking,
}
struct ConstructedHandoff {
    result: Result<(), worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial>,
    backing: RecoveryHandoffBacking,
}
#[inline(never)]
fn prepare(
    reopened: ReopenedPhysicalRecovery,
    closed_cleanup: worth_store::physical_runtime::ClosedPhysicalRecoveryCleanup,
    cleanup: crate::handoff::RecoveryCleanupPosture,
) -> PreparedHandoff {
    let retained = resident_memory::retained_bytes(&reopened, &cleanup)
        .and_then(|heap| heap.checked_sub(reopened.payload_backing_bytes()?))
        .and_then(|heap| heap.checked_add(std::mem::size_of::<PreparedHandoff>() as u64))
        .and_then(|bytes| bytes.checked_add(closed_cleanup.owned_heap_bytes()?));
    let (payload, mut backing) = reopened.into_handoff_parts();
    let crate::progression::ReopenedPayload {
        state,
        expectation,
        publication_counters,
        publication_settlement,
        reopened: pending_reopen,
        reopen_counters,
    } = payload;
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
    let requires_allocation = match &state.custody {
        crate::progression::CustodyState::SourceHeads(_)
        | crate::progression::CustodyState::Pending { .. }
        | crate::progression::CustodyState::OrderedCompleted { .. } => true,
        crate::progression::CustodyState::NoCheckpoint
        | crate::progression::CustodyState::NoRelease(_) => false,
    };
    let input = if let Err(denial) = resident_admission {
        Err(worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::ResidentAdmission(denial))
    } else if requires_allocation && recovery_allocation.is_none() {
        Err(worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch)
    } else {
        Ok(construction::RejoinConstructionInput {
            construction: construction::RejoinConstruction {
                coordination,
                media,
                closed_cleanup,
            },
            custody: state.custody,
            tier: state.verified_selected_tier_custody,
            allocation: recovery_allocation,
            absent: absent_checkpoint,
            durability,
        })
    };
    backing.pending.push(completion::PendingCompletion {
        store,
        session_identity,
        recovery_effects,
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
    });
    PreparedHandoff { input, backing }
}
#[inline(never)]
fn construct(mut prepared: PreparedHandoff) -> ConstructedHandoff {
    let result = match prepared.input {
        Ok(input) => {
            let construction::RejoinConstructionInput {
                construction,
                custody,
                tier,
                allocation,
                absent,
                durability,
            } = input;
            construction
                .complete(
                    custody,
                    tier,
                    allocation,
                    absent,
                    durability,
                    &prepared.backing.pending[0].selection,
                )
                .map(|core| prepared.backing.core.push(core))
        }
        Err(denial) => Err(denial),
    };
    ConstructedHandoff {
        result,
        backing: prepared.backing,
    }
}
#[inline(never)]
fn publish(mut constructed: ConstructedHandoff) -> PhysicalRecoveryOutcome {
    let completion = constructed
        .backing
        .pending
        .pop()
        .expect("one original handoff completion");
    match constructed.result {
        Ok(()) => completion.recovered(
            constructed
                .backing
                .core
                .pop()
                .expect("actual Store core constructed"),
        ),
        Err(denial) => completion.refused(denial),
    }
}
#[cfg(test)]
#[path = "handoff_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
