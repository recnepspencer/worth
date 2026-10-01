use worth_store::physical_runtime::{
    PhysicalRecoveryFreshReopenCommand, PhysicalRecoveryFreshReopenOutcome,
};

use crate::entry::{
    PhysicalRecoveryOutcome, PhysicalRecoveryPublicationIndeterminate,
    PhysicalRecoveryPublicationSettlement, PhysicalRecoveryReopenCounters,
    PhysicalRecoveryReopenFailure,
};
use crate::progression::{NamespaceDurablePhysicalRecovery, ReopenedPhysicalRecovery};

pub(crate) fn reopen_recovery(
    durable: NamespaceDurablePhysicalRecovery,
) -> Result<ReopenedPhysicalRecovery, PhysicalRecoveryOutcome> {
    let NamespaceDurablePhysicalRecovery {
        mut state,
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
            let (rebound_state, custody_valid) = rebind_selected_custody(
                state,
                &mut expectation,
                &publication_settlement,
                &publication_counters,
                &completed,
            );
            state = rebound_state;
            if !custody_valid {
                return Err(custody_indeterminate(
                    state,
                    publication_counters,
                    publication_settlement,
                ));
            }
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

fn rebind_selected_custody(
    mut state: crate::progression::NamespaceDurableState,
    expectation: &mut crate::progression::RecoveryPublicationExpectation,
    settlement: &crate::entry::PhysicalRecoveryPublicationSettlementLedger,
    counters: &crate::entry::PhysicalRecoveryPublicationCounters,
    completed: &worth_store::physical_runtime::CompletedPhysicalRecoveryFreshReopen,
) -> (crate::progression::NamespaceDurableState, bool) {
    if state.verified_selected_checkpoint_custody.is_none()
        && state.verified_selected_head_custody_v2.is_none()
        && state.verified_selected_no_release_custody.is_none()
        && state.verified_pending_wal_release_custody.is_none()
        && state.verified_ordered_historical_release_custody.is_none()
        && state.verified_selected_tier_custody.is_none()
    {
        return (state, true);
    }
    let selected = state.selection.root().selected();
    let source = selected.manifest();
    let published = completed.root();
    let occurrence = completed.fresh_reopen_occurrence();
    if published != expectation.recovered_root()
        || completed.format() != selected.selector().format()
        || occurrence.plan() != expectation.plan_identity()
        || occurrence.generation() != published.generation()
        || expectation.store_identity() != selected.selector().store_identity()
        || expectation.source_generation() != source.generation()
        || expectation.current_selector().root_generation() != published.generation()
    {
        return (state, false);
    }
    if published != source
        && (!matches!(
            settlement.settlement(),
            PhysicalRecoveryPublicationSettlement::Completed(_)
        ) || counters.root_protocol_replacements_performed != 1
            || expectation.staging_generation() != published.generation()
            || published.generation() <= source.generation())
    {
        return (state, false);
    }
    if let Some(claim) = &mut state.verified_selected_head_custody_v2 {
        if claim
            .rebind_published_root(&state.selection, published, completed.format())
            .is_err()
        {
            return (state, false);
        }
    }
    if let Some(claim) = &mut state.verified_selected_checkpoint_custody {
        if claim
            .rebind_published_root(&state.selection, published, completed.format())
            .is_err()
        {
            return (state, false);
        }
    }
    if let Some(claim) = &mut state.verified_selected_no_release_custody {
        if claim
            .rebind_published_root(&state.selection, published, completed.format())
            .is_err()
        {
            return (state, false);
        }
    }
    if let Some(claim) = &mut state.verified_pending_wal_release_custody {
        if claim
            .rebind_published_root(&state.selection, published, completed.format())
            .is_err()
        {
            return (state, false);
        }
        let Some(topology) = expectation.take_release_topology() else {
            return (state, false);
        };
        if claim
            .rebind_verified_topology(
                &topology.source_free,
                &topology.published_free,
                topology.transition,
                completed.format(),
            )
            .is_err()
        {
            return (state, false);
        }
    }
    if let Some(claim) = state.verified_pending_wal_release_custody.as_mut() {
        if state.verified_effective_release_heads_v14.is_none() {
            let source_count = claim
                .selected_head_v2()
                .map_or(0usize, |base| base.selected_heads().len());
            let Some(maximum_entries) = (source_count as u64)
                .checked_add(claim.ordered_released_batches().len() as u64)
                .and_then(|entries| entries.checked_add(1))
            else {
                return (state, false);
            };
            let Some(maximum_retained_bytes) = state
                .coordination
                .owner()
                .recovery_allocation_admission()
                .map(|allocation| allocation.byte_limit())
            else {
                return (state, false);
            };
            let admitted = if claim.ordered_history().is_some() {
                worth_store_recovery_physics::VerifiedEffectiveReleaseHeadRosterV14::admit_ordered_pending(
                    claim,
                    maximum_entries,
                    maximum_retained_bytes,
                )
            } else {
                worth_store_recovery_physics::VerifiedEffectiveReleaseHeadRosterV14::admit_pending(
                    claim,
                    maximum_entries,
                    maximum_retained_bytes,
                )
            };
            let Ok(effective) = admitted else {
                return (state, false);
            };
            state.verified_effective_release_heads_v14 = Some(effective);
        }
    }
    if state.verified_ordered_historical_release_custody.is_some()
        && state.verified_effective_release_heads_v14.is_none()
    {
        return (state, false);
    }
    if state
        .verified_ordered_historical_release_custody
        .as_ref()
        .is_some_and(|claim| claim.selected_root() != published || published != source)
    {
        return (state, false);
    }
    let selected_free = if let Some(claim) = &state.verified_selected_tier_custody {
        let byte_limit = claim.free_header().encode(completed.format()).len() as u64;
        let (media, observed) = observed_published_free_header(
            state.authority.media,
            &mut state.integrity_trace,
            published,
            completed.format(),
            byte_limit,
        );
        state.authority.media = media;
        match observed {
            Some(header) => Some(header),
            None => return (state, false),
        }
    } else {
        None
    };
    if let Some(claim) = &mut state.verified_selected_tier_custody {
        if claim
            .rebind_published_root_and_free_header(
                &state.selection,
                published,
                selected_free
                    .as_ref()
                    .expect("tier claim requires observed free header"),
                completed.format(),
            )
            .is_err()
        {
            return (state, false);
        }
    }
    (state, true)
}

fn observed_published_free_header(
    media: worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia,
    trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    root: &worth_store_physical_format::DurablePhysicalRootManifest,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    byte_limit: u64,
) -> (
    worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia,
    Option<worth_store_physical_format::DurableFreeSpaceManifestHeader>,
) {
    let mut discovery = media
        .bounded_discovery(1, byte_limit)
        .expect("the admitted tier header has a positive exact read bound");
    let header = (|| {
        let source = discovery
            .read_free_space_manifest(root.generation(), byte_limit)
            .ok()?;
        let header = crate::integrity_ingress::projection::free_space_header(
            &source,
            discovery.store_identity(),
            format,
            root,
            trace,
        )
        .ok()?;
        (source.bytes()? == header.encode(format)).then_some(header)
    })();
    (discovery.finish(), header)
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
