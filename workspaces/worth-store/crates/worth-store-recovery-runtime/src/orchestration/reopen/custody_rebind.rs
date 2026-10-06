//! Published-root custody rebind and the sole planning-to-final state conversion.

use worth_store::physical_runtime::{ArtifactCeiling, PageAddress, ReadGrant, UnchargedRead};

use crate::entry::PhysicalRecoveryPublicationSettlement;
use crate::progression::{CustodyState, PlanningCustody};

pub(super) fn rebind_selected_custody(
    mut state: crate::progression::NamespaceDurableState,
    expectation: &mut crate::progression::RecoveryPublicationExpectation,
    settlement: &crate::entry::PhysicalRecoveryPublicationSettlementLedger,
    counters: &crate::entry::PhysicalRecoveryPublicationCounters,
    completed: &worth_store::physical_runtime::CompletedPhysicalRecoveryFreshReopen,
) -> Result<
    crate::progression::NamespaceDurableState<CustodyState>,
    crate::progression::NamespaceDurableState,
> {
    // Store's generation-zero root and SHA check in construction guards this early return.
    if matches!(&state.custody, PlanningCustody::NoCheckpoint)
        && state.verified_selected_tier_custody.is_none()
    {
        return Ok(into_reopened_state(state, CustodyState::NoCheckpoint));
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
        return Err(state);
    }
    if published != source
        && (!matches!(
            settlement.settlement(),
            PhysicalRecoveryPublicationSettlement::Completed(_)
        ) || counters.root_protocol_replacements_performed != 1
            || expectation.staging_generation() != published.generation()
            || published.generation() <= source.generation())
    {
        return Err(state);
    }
    match &mut state.custody {
        PlanningCustody::SourceHeads(claim) => {
            if claim
                .rebind_published_root(&state.selection, published, completed.format())
                .is_err()
            {
                return Err(state);
            }
        }
        PlanningCustody::NoRelease(claim) => {
            if claim
                .rebind_published_root(&state.selection, published, completed.format())
                .is_err()
            {
                return Err(state);
            }
        }
        PlanningCustody::PendingPrepared { claim, .. } => {
            if claim
                .rebind_published_root(&state.selection, published, completed.format())
                .is_err()
            {
                return Err(state);
            }
            let Some(topology) = expectation.take_release_topology() else {
                return Err(state);
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
                return Err(state);
            }
        }
        PlanningCustody::OrderedCompleted { claim, .. } => {
            if claim.selected_root() != published || published != source {
                return Err(state);
            }
        }
        PlanningCustody::NoCheckpoint => {}
        PlanningCustody::Unresolved => return Err(state),
    }
    let pending_effective = if let PlanningCustody::PendingPrepared { claim, .. } =
        &mut state.custody
    {
        let source_count = claim
            .selected_head_v2()
            .map_or(0usize, |base| base.selected_heads().len());
        let Some(maximum_entries) = (source_count as u64)
            .checked_add(claim.ordered_released_batches().len() as u64)
            .and_then(|entries| entries.checked_add(1))
        else {
            return Err(state);
        };
        let Some(maximum_retained_bytes) = state
            .coordination
            .owner()
            .recovery_allocation_admission()
            .map(|allocation| allocation.byte_limit())
        else {
            return Err(state);
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
        match admitted {
            Ok(effective) => Some(effective),
            Err(_) => return Err(state),
        }
    } else {
        None
    };
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
            None => return Err(state),
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
            return Err(state);
        }
    }
    let planning = std::mem::replace(&mut state.custody, PlanningCustody::Unresolved);
    let custody = match (planning, pending_effective) {
        (PlanningCustody::NoCheckpoint, None) => CustodyState::NoCheckpoint,
        (PlanningCustody::NoRelease(claim), None) => CustodyState::NoRelease(claim),
        (PlanningCustody::SourceHeads(claim), None) => CustodyState::SourceHeads(claim),
        (PlanningCustody::PendingPrepared { claim, replay }, Some(effective_heads)) => {
            CustodyState::Pending {
                claim,
                replay: replay.into_head(),
                effective_heads,
            }
        }
        (
            PlanningCustody::OrderedCompleted {
                claim,
                effective_heads,
            },
            None,
        ) => CustodyState::OrderedCompleted {
            claim,
            effective_heads,
        },
        (planning, _) => {
            state.custody = planning;
            return Err(state);
        }
    };
    Ok(into_reopened_state(state, custody))
}

/// This phase conversion is private to the successful selected-root rebind.
fn into_reopened_state(
    state: crate::progression::NamespaceDurableState,
    custody: CustodyState,
) -> crate::progression::NamespaceDurableState<CustodyState> {
    crate::progression::NamespaceDurableState {
        authority: state.authority,
        coordination: state.coordination,
        selection: state.selection,
        custody,
        verified_selected_tier_custody: state.verified_selected_tier_custody,
        discovery_counters: state.discovery_counters,
        root_protocol_denials: state.root_protocol_denials,
        integrity: state.integrity,
        freshness: state.freshness,
        fates: state.fates,
        planning_counters: state.planning_counters,
        root_protocol_counters: state.root_protocol_counters,
        base: state.base,
        quiescence: state.quiescence,
        closed: state.closed,
        staging_counters: state.staging_counters,
        staging_settlements: state.staging_settlements,
        integrity_trace: state.integrity_trace,
    }
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
        // The page ceiling bounds the file; the discovery's byte bound
        // holds it to the header's exact encoding.
        let address = PageAddress::FreeSpaceManifest {
            generation: root.generation(),
        };
        let source = discovery
            .read(
                ArtifactCeiling::page(format, address),
                ReadGrant::ceiling_only(),
            )
            .observed()
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
