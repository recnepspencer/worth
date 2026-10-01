use crate::entry::{PhysicalRecoveryOutcome, PhysicalRecoveryPublicationIndeterminate};
use crate::handoff::{RecoveredPhysicalRuntimeHandoff, RecoveredPhysicalRuntimeHandoffEvidence};
use crate::progression::ReopenedPhysicalRecovery;
#[path = "handoff/resident_memory.rs"]
mod resident_memory;
#[cfg(feature = "certification-test-authority")]
use worth_store_physical_format::{BlobRecordKind, SelectedRecordContentClass};

pub(crate) fn finish_recovery_after_cleanup(
    reopened: ReopenedPhysicalRecovery,
    closed_cleanup: worth_store::physical_runtime::ClosedPhysicalRecoveryCleanup,
    cleanup: crate::handoff::RecoveryCleanupPosture,
) -> PhysicalRecoveryOutcome {
    let retained = resident_memory::retained_bytes(&reopened, &cleanup)
        .and_then(|heap| heap.checked_add(resident_memory::inline_bytes(&reopened, &cleanup)?))
        .and_then(|bytes| bytes.checked_add(closed_cleanup.owned_heap_bytes()?))
        .and_then(|bytes| bytes.checked_add(std::mem::size_of_val(&closed_cleanup) as u64));
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
    let mut coordination = state.coordination.into_owner();
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
        match (state.custody, state.verified_selected_tier_custody) {
            (crate::progression::CustodyState::SourceHeads(claim), tier) => {
                if let Some(allocation) = recovery_allocation {
            #[cfg(feature = "certification-test-authority")]
            if crate::certification::enabled() {
                let descriptor = claim.selected_heads().iter().find_map(|head| {
                    state
                        .selection
                        .page_facts()
                        .placements()
                        .iter()
                        .copied()
                        .find(|route| {
                            route.record() == head.descriptor_record()
                                && route.content_class()
                                    == SelectedRecordContentClass::Blob(
                                        BlobRecordKind::ReclaimDescriptorV3,
                                    )
                        })
                });
                if let Some(descriptor) = descriptor {
                    crate::certification::pause_after_claim(descriptor);
                }
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::certification_construct_with_verified_head_custody_v2_and_rejoin_pause(
                    coordination,
                    media,
                    closed_cleanup,
                    allocation,
                    claim,
                    tier,
                    crate::certification::take_rejoin_pause(),
                )
            } else {
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_head_custody_v2(
                    coordination, media, closed_cleanup, allocation, claim, tier,
                )
            }
            #[cfg(not(feature = "certification-test-authority"))]
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_head_custody_v2(
                coordination, media, closed_cleanup, allocation, claim, tier,
            )
        } else {
                    Err(worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch)
                }
            }
        (crate::progression::CustodyState::NoRelease(no_release), Some(tier)) => {
            #[cfg(feature = "certification-test-authority")]
            if crate::certification::enabled() {
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::certification_construct_with_verified_tier_no_release(
                    coordination,
                    media,
                    closed_cleanup,
                    tier,
                    no_release,
                    crate::certification::take_rejoin_pause(),
                )
            } else {
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_tier_no_release(
                    coordination, media, closed_cleanup, tier, no_release,
                )
            }
            #[cfg(not(feature = "certification-test-authority"))]
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_tier_no_release(
                coordination, media, closed_cleanup, tier, no_release,
            )
        }
        (crate::progression::CustodyState::NoRelease(no_release), None) => {
            #[cfg(feature = "certification-test-authority")]
            if crate::certification::enabled() {
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::certification_construct_with_verified_no_release(
                    coordination,
                    media,
                    closed_cleanup,
                    no_release,
                    crate::certification::take_rejoin_pause(),
                )
            } else {
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_no_release(
                    coordination, media, closed_cleanup, no_release,
                )
            }
            #[cfg(not(feature = "certification-test-authority"))]
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_no_release(
                coordination, media, closed_cleanup, no_release,
            )
        }
        (crate::progression::CustodyState::Pending { claim: pending, replay, effective_heads: effective }, None) => {
            let allocation = recovery_allocation.expect("pending recovery allocation was checked");
            drop(replay);
            #[cfg(feature = "certification-test-authority")]
            if crate::certification::enabled() {
                let reservation = state.selection.page_facts().placements().iter().copied()
                    .find(|route| route.record() == pending.reservation_record())
                    .expect("pending-WAL reservation was already joined");
                crate::certification::pause_after_claim(reservation);
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::certification_construct_with_verified_pending_wal_release_and_rejoin_pause(
                    coordination, media, closed_cleanup, allocation, pending, effective,
                    crate::certification::take_rejoin_pause(),
                )
            } else {
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_pending_wal_release(
                    coordination, media, closed_cleanup, allocation, pending, effective,
                )
            }
            #[cfg(not(feature = "certification-test-authority"))]
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_pending_wal_release(
                coordination, media, closed_cleanup, allocation, pending, effective,
            )
        }
        (crate::progression::CustodyState::Pending { claim: pending, replay, effective_heads: effective }, Some(tier)) => {
            let allocation = recovery_allocation.expect("pending recovery allocation was checked");
            drop(replay);
            #[cfg(feature = "certification-test-authority")]
            if crate::certification::enabled() {
                let reservation = state.selection.page_facts().placements().iter().copied()
                    .find(|route| route.record() == pending.reservation_record())
                    .expect("pending-WAL reservation was already joined");
                crate::certification::pause_after_claim(reservation);
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::certification_construct_with_verified_tier_and_pending_wal_release_and_rejoin_pause(
                    coordination, media, closed_cleanup, allocation, pending, effective, tier,
                    crate::certification::take_rejoin_pause(),
                )
            } else {
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_tier_and_pending_wal_release(
                    coordination, media, closed_cleanup, allocation, pending, effective, tier,
                )
            }
            #[cfg(not(feature = "certification-test-authority"))]
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_tier_and_pending_wal_release(
                coordination, media, closed_cleanup, allocation, pending, effective, tier,
            )
        }
        (crate::progression::CustodyState::OrderedCompleted { claim: historical, effective_heads: effective }, tier) => {
            let allocation = recovery_allocation.expect("historical recovery allocation was checked");

            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_ordered_historical_release(
                coordination,
                media,
                closed_cleanup,
                allocation,
                historical,
                effective,
                tier,
            )
        }
        (crate::progression::CustodyState::NoCheckpoint, None) => worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct(
            coordination,
            media,
            closed_cleanup,
        ),
        _ => Err(
            worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch,
        ),
        }
    };
    match construction {
        Ok(core) => {
            let session = session.recovered();
            PhysicalRecoveryOutcome::Recovered(RecoveredPhysicalRuntimeHandoff::new(
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
