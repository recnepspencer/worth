//! Authenticated historical V3 result against its own addressed candidate.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{BlobReclaimSourceBasisV1, BlobRecordKind, BlobRecordV1};

use super::super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use super::super::historical_publication::{self, HistoricalFailure};
use super::{binding_matches, historical, historical_predecessors, selected, selected_blob_record};
use crate::orchestration::planning::{
    page_observation::HistoricalDropEvidence, selected_source_inventory,
};
use crate::progression::verified_historical_release_transition;

pub(in crate::orchestration::planning::completion) fn verify_historical(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    evidence: &HistoricalDropEvidence,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let descriptor = evidence.descriptor;
    let base = descriptor.base();
    let custody = descriptor.custody();
    let selected_generation = context
        .selection
        .root()
        .selected()
        .selector()
        .root_generation();
    let advanced_chain = evidence.chain.as_ref();
    let ordered_history = evidence.ordered_history.as_deref();
    let ordered_edge = ordered_history.and_then(|history| {
        let mut matching = history.edges().iter().filter_map(|edge| match edge {
            worth_store_recovery_physics::VerifiedOrderedRootEdge::Released(edge)
                if edge.operation() == evidence.operation =>
            {
                Some(edge)
            }
            _ => None,
        });
        let first = matching.next()?;
        matching.next().is_none().then_some(first)
    });
    if base.store() != context.authority.media.store_identity().bytes()
        || base.source_root_generation().checked_add(1) != Some(base.candidate_root_generation())
        || advanced_chain.map_or_else(
            || {
                ordered_history.map_or_else(
                    || base.candidate_root_generation() != selected_generation,
                    |history| {
                        ordered_edge.is_none_or(|edge| {
                            edge.candidate_root_generation() != base.candidate_root_generation()
                                || edge.descriptor_record() != evidence.descriptor_record
                        }) || history.selected_root_frame_sha256()
                            != <[u8; 32]>::from(Sha256::digest(
                                context
                                    .selection
                                    .root()
                                    .selected()
                                    .manifest()
                                    .encode(context.authority.record_format),
                            ))
                    },
                )
            },
            |chain| {
                chain.descriptor_operation() != evidence.operation
                    || chain.source_root_generation() != base.source_root_generation()
                    || chain.first_result_generation() != base.candidate_root_generation()
                    || chain.checkpoint_root_frame_sha256().is_none()
                    || chain.selected_root_frame_sha256()
                        != <[u8; 32]>::from(Sha256::digest(
                            context
                                .selection
                                .root()
                                .selected()
                                .manifest()
                                .encode(context.authority.record_format),
                        ))
            },
        )
        || custody.request().idempotency() != evidence.operation
        || base.manifest_record() == evidence.descriptor_record
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let format = context.authority.record_format;
    let (next, historical_source) = historical_publication::observe(
        context,
        basis,
        base.source_root_generation(),
        base.manifest_record(),
        |discovery, root, route, budget, trace, _| {
            if route.is_none()
                || Sha256::digest(root.encode(format)).as_slice()
                    != custody.source_root_frame_sha256()
            {
                return Err(HistoricalFailure::Invalid);
            }
            let inventory = selected_source_inventory::observe_with_budget(
                discovery,
                root,
                format,
                budget,
                u64::from(format.page_size().bytes()),
                trace,
            )
            .map_err(|_| HistoricalFailure::Invalid)?;
            if <[u8; 32]>::from(Sha256::digest(inventory.free_space.encode(format)))
                != custody.source_free_space_frame_sha256()
            {
                return Err(HistoricalFailure::Invalid);
            }
            Ok((root.clone(), inventory))
        },
    )?;
    context = next;
    let (next, manifest_bytes) = selected_blob_record(
        context,
        basis,
        base.source_root_generation(),
        base.manifest_record(),
        BlobRecordKind::DropSetManifestV3,
    )?;
    context = next;
    let Ok(BlobRecordV1::DropSetManifestV3(manifest)) =
        worth_store_physical_format::decode_blob_record(&manifest_bytes)
    else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if manifest != evidence.manifest
        || !binding_matches(base, &manifest, &manifest_bytes, evidence.descriptor_record)
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis() else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let historical_dropped = if base.predecessor().is_some() {
        let Some(releases) = basis.observed_pages.ordered_releases.as_deref() else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        let Some(dropped) = historical_predecessors::authenticate_no_release_chain(
            base,
            source,
            manifest.count(),
            releases,
            context.limits.manifest_entries,
        ) else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        dropped
    } else {
        let (next, publication_bytes) = selected_blob_record(
            context,
            basis,
            base.source_root_generation(),
            source.publication_record(),
            BlobRecordKind::GenerationPublished,
        )?;
        context = next;
        if publication_bytes != source.publication().encode()
            || <[u8; 32]>::from(Sha256::digest(&publication_bytes))
                != source.publication_frame_sha256()
        {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        Vec::new()
    };
    let (next, source_routes) = historical_publication::observe_all_routes(
        context,
        basis,
        base.source_root_generation(),
        base.manifest_record(),
    )?;
    if historical_dropped.iter().any(|record| {
        source_routes
            .binary_search_by_key(record, |route| route.record())
            .is_ok()
    }) {
        return Err(next.redo_block(basis.planning_counters(), None));
    }
    context = selected::verify_initial(
        next,
        basis,
        base,
        &manifest,
        evidence.operation,
        &source_routes,
        Some(custody),
        Some(evidence.descriptor_record),
        super::closure_evidence::ReleasedClosureEvidence::RetainedHistory(&historical_dropped),
    )?;
    let mut removed = manifest.dropped().to_vec();
    let Some(projection) = basis
        .redo
        .projections()
        .iter()
        .find(|projection| projection.operation() == evidence.operation)
    else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if let worth_store_physical_format::PersistedPhysicalRecoveryOperation::DerivedDirectory {
        retirement: Some(retirement),
        ..
    } = projection.materialization().operation()
    {
        removed.extend_from_slice(retirement.dropped_records());
    }
    let projected = projection.materialization().placements().to_vec();
    removed.sort_unstable();
    let route_count = source_routes.len() as u64;
    let free_count = historical_source.1.free_entries.len() as u64;
    let segment_count = historical_source.1.segment_pages.len() as u64;
    let projected_count = projected.len() as u64;
    let scratch_bytes = route_count
        .checked_mul(
            (std::mem::size_of::<worth_store_physical_format::CurrentPhysicalRecordPlacement>()
                + 2 * std::mem::size_of::<crate::progression::RecoveryBaseImageAction>())
                as u64,
        )
        .and_then(|bytes| {
            free_count
                .checked_mul(
                    (std::mem::size_of::<worth_store_physical_format::RecordFreeSpaceManifestEntry>(
                    ) + 4 * std::mem::size_of::<usize>()) as u64,
                )
                .and_then(|extra| bytes.checked_add(extra))
        })
        .and_then(|bytes| {
            segment_count.checked_mul(
                (std::mem::size_of::<worth_store_physical_format::RecordSegmentPageManifestEntry>()
                    + 4 * std::mem::size_of::<usize>()) as u64,
            ).and_then(|extra| bytes.checked_add(extra))
        })
        .and_then(|bytes| {
            projected_count
                .checked_mul(std::mem::size_of::<
                    worth_store_physical_format::CurrentPhysicalRecordPlacement,
                >() as u64)
                .and_then(|extra| bytes.checked_add(extra))
        });
    let Some(scratch_bytes) = scratch_bytes else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    basis
        .observed_pages
        .historical_publication_peak_scratch_bytes = basis
        .observed_pages
        .historical_publication_peak_scratch_bytes
        .max(scratch_bytes);
    let candidate = if advanced_chain.is_some() || ordered_edge.is_some() {
        let (next, candidate) = historical_publication::observe(
            context,
            basis,
            base.candidate_root_generation(),
            evidence.descriptor_record,
            |discovery, root, route, budget, trace, _| {
                if route.is_none() {
                    return Err(HistoricalFailure::Invalid);
                }
                let inventory = selected_source_inventory::observe_with_budget(
                    discovery,
                    root,
                    format,
                    budget,
                    u64::from(format.page_size().bytes()),
                    trace,
                )
                .map_err(|_| HistoricalFailure::Invalid)?;
                Ok((root.clone(), inventory))
            },
        )?;
        context = next;
        let (next, routes) = historical_publication::observe_all_routes(
            context,
            basis,
            base.candidate_root_generation(),
            evidence.descriptor_record,
        )?;
        context = next;
        Some((candidate.0, candidate.1, routes))
    } else {
        None
    };
    let head_replay = if let Some(edge) = ordered_edge {
        let mut matching = basis
            .observed_pages
            .ordered_releases
            .iter()
            .flatten()
            .filter(|release| release.operation == evidence.operation);
        let Some(release) = matching.next() else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        if matching.next().is_some()
            || release.descriptor != descriptor
            || release.head_replay.source_root_frame_sha256() != edge.source_root_frame_sha256()
            || release.head_replay.result_root_frame_sha256() != edge.result_root_frame_sha256()
        {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        Some(release.head_replay.replay())
    } else {
        None
    };
    let transition = if let Some((root, inventory, routes)) = candidate.as_ref() {
        verified_historical_release_transition(
            &historical_source.0,
            &historical_source.1,
            &source_routes,
            root,
            inventory,
            routes,
            &removed,
            &projected,
            head_replay,
            format,
            context.limits.manifest_entries,
            context.limits.staging_bytes,
        )
    } else {
        verified_historical_release_transition(
            &historical_source.0,
            &historical_source.1,
            &source_routes,
            context.selection.root().selected().manifest(),
            &basis.observed_pages.selected_source,
            context.selection.page_facts().placements(),
            &removed,
            &projected,
            head_replay,
            format,
            context.limits.manifest_entries,
            context.limits.staging_bytes,
        )
    };
    let Some((transition, input_scratch)) = transition else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if advanced_chain.is_some_and(|chain| {
        chain.first_transition() != &transition
            || !chain.selected_topology().matches_headers(
                context.selection.root().selected().manifest(),
                &basis.observed_pages.selected_source.free_space,
                format,
            )
    }) || ordered_edge.is_some_and(|edge| {
        edge.transition() != &transition
            || ordered_history.is_none_or(|history| {
                !history.selected_topology().matches_headers(
                    context.selection.root().selected().manifest(),
                    &basis.observed_pages.selected_source.free_space,
                    format,
                )
            })
    }) {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    basis
        .observed_pages
        .historical_publication_peak_scratch_bytes = basis
        .observed_pages
        .historical_publication_peak_scratch_bytes
        .max(
            scratch_bytes
                .saturating_add(input_scratch)
                .saturating_add(transition.scratch_bytes()),
        );
    context = historical::verify_v3_result(
        context,
        basis,
        descriptor,
        evidence.descriptor_record,
        &manifest,
    )?;
    basis
        .verified_historical_release_operations
        .push(evidence.operation);
    basis.verified_historical_release_sources.push((
        evidence.operation,
        historical_source.0,
        historical_source.1.free_space,
    ));
    Ok(context)
}
