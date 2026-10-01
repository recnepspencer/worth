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
    let effective_heads = state.verified_effective_release_heads_v14;
    let has_effective_release = state.verified_pending_wal_release_custody.is_some()
        || state.verified_ordered_historical_release_custody.is_some();
    let construction = if let Err(denial) = resident_admission {
        Err(worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::ResidentAdmission(denial))
    } else if has_effective_release != effective_heads.is_some()
        || (has_effective_release && recovery_allocation.is_none())
    {
        Err(worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch)
    } else if let Some(claim) = state.verified_selected_head_custody_v2 {
        if state.verified_selected_checkpoint_custody.is_some()
            || state.verified_selected_no_release_custody.is_some()
            || state.verified_pending_wal_release_custody.is_some()
            || state.verified_ordered_historical_release_custody.is_some()
        {
            Err(worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch)
        } else if let Some(allocation) = coordination.recovery_allocation_admission() {
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
                    state.verified_selected_tier_custody,
                    crate::certification::take_rejoin_pause(),
                )
            } else {
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_head_custody_v2(
                    coordination, media, closed_cleanup, allocation, claim, state.verified_selected_tier_custody,
                )
            }
            #[cfg(not(feature = "certification-test-authority"))]
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_head_custody_v2(
                coordination, media, closed_cleanup, allocation, claim, state.verified_selected_tier_custody,
            )
        } else {
            Err(worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch)
        }
    } else {
        match (
        state.verified_selected_checkpoint_custody,
        state.verified_selected_no_release_custody,
        state.verified_pending_wal_release_custody,
        state.verified_selected_tier_custody,
        state.verified_ordered_historical_release_custody,
    ) {
        (Some(released), None, None, Some(tier), None) => {
            #[cfg(feature = "certification-test-authority")]
            if crate::certification::enabled() {
                let descriptor = state.selection.page_facts().placements().iter().copied()
                    .find(|route| route.record() == released.accumulator().tip().descriptor_record())
                    .expect("certified selected descriptor was already joined");
                crate::certification::pause_after_claim(descriptor);
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::certification_construct_with_verified_tier_and_release(
                    coordination, media, closed_cleanup, tier, released,
                    crate::certification::take_rejoin_pause(),
                )
            } else {
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_tier_and_release(
                    coordination, media, closed_cleanup, tier, released,
                )
            }
            #[cfg(not(feature = "certification-test-authority"))]
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_tier_and_release(
                coordination, media, closed_cleanup, tier, released,
            )
        }
        (Some(custody), None, None, None, None) => {
            #[cfg(feature = "certification-test-authority")]
            if crate::certification::enabled() {
                let descriptor = state
                    .selection
                    .page_facts()
                    .placements()
                    .iter()
                    .copied()
                    .find(|route| route.record() == custody.accumulator().tip().descriptor_record())
                    .expect("certified selected descriptor was already joined");
                crate::certification::pause_after_claim(descriptor);
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::certification_construct_with_verified_custody_and_rejoin_pause(
                    coordination, media, closed_cleanup, custody,
                    crate::certification::take_rejoin_pause(),
                )
            } else {
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_custody(
                    coordination, media, closed_cleanup, custody,
                )
            }
            #[cfg(not(feature = "certification-test-authority"))]
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_custody(
                coordination, media, closed_cleanup, custody,
            )
        }
        (None, Some(no_release), None, Some(tier), None) => {
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
        (None, Some(no_release), None, None, None) => {
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
        (None, None, Some(pending), None, None) => {
            let allocation = recovery_allocation.expect("pending recovery allocation was checked");
            let effective = effective_heads.expect("pending effective heads were checked");
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
        (None, None, Some(pending), Some(tier), None) => {
            let allocation = recovery_allocation.expect("pending recovery allocation was checked");
            let effective = effective_heads.expect("pending effective heads were checked");
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
        (None, None, None, tier, Some(historical)) => {
            let allocation = recovery_allocation.expect("historical recovery allocation was checked");
            let effective = effective_heads.expect("historical effective heads were checked");
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
        (None, None, None, None, None) => worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct(
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
