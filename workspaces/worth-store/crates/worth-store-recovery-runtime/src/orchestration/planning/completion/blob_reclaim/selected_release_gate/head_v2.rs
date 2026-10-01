//! Observes checkpoint-source custody independently of the newer selected
//! post-WAL root. Only the joined controls claim leaves this boundary.

use worth_store_physical_integrity::ReleaseCustodyHeadWalkLimitsV1;
use worth_store_recovery_physics::{
    VerifiedCheckpointReleaseHeadRosterV2, VerifiedSelectedReleaseHeadCustodyV2,
};

use super::super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use super::resident_basis;
use crate::integrity_ingress::{admit_addressed_root, RecoveryArtifactNamespaceJoin};
use crate::orchestration::planning::selected_source_inventory::{self, ResidentAllowance};

pub(super) fn admit(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let remaining_entries = basis.observed_pages.manifest_budget.remaining();
    let remaining_bytes = context
        .limits
        .observation_bytes
        .saturating_sub(context.counters.bytes_observed)
        .saturating_sub(basis.observed_pages.bytes_read)
        .saturating_sub(basis.observed_pages.candidate_bytes_read)
        .saturating_sub(basis.observed_pages.source_copy_bytes_read)
        .saturating_sub(basis.observed_pages.historical_publication_bytes_read);
    if remaining_entries == 0 || remaining_bytes == 0 {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let mut resident = match resident_basis::seed(&context, basis) {
        Ok(resident) => resident,
        Err(limit) => return Err(context.redo_block(basis.planning_counters(), limit)),
    };
    let mut discovery = context
        .authority
        .media
        .bounded_discovery(remaining_entries, remaining_bytes)
        .expect("positive V2 checkpoint-source bounds");
    let mut scratch = 0;
    let claim = observe(
        &mut discovery,
        &context.selection,
        context.authority.record_format,
        remaining_bytes,
        &mut basis.observed_pages.manifest_budget,
        &mut context.integrity_trace,
        &mut scratch,
        &mut resident,
    );
    let counters = discovery.counters();
    context.authority.media = discovery.finish();
    basis.observed_pages.historical_publication_reads = basis
        .observed_pages
        .historical_publication_reads
        .saturating_add(counters.addressed_artifacts_read);
    basis.observed_pages.historical_publication_bytes_read = basis
        .observed_pages
        .historical_publication_bytes_read
        .saturating_add(counters.bytes_read);
    basis
        .observed_pages
        .historical_publication_peak_scratch_bytes = basis
        .observed_pages
        .historical_publication_peak_scratch_bytes
        .max(resident.peak());
    let Some(claim) = claim else {
        let limit = resident_basis::limit_failure(&context, &resident);
        return Err(context.redo_block(basis.planning_counters(), limit));
    };
    if basis
        .verified_selected_head_custody_v2
        .replace(claim)
        .is_some()
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    Ok(context)
}

#[allow(clippy::too_many_arguments)]
fn observe(
    discovery: &mut worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery,
    selection: &worth_store_recovery_physics::PhysicalSourceSelection,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    byte_limit: u64,
    budget: &mut crate::orchestration::planning::manifest_entry_budget::ManifestEntryBudget,
    trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
    resident: &mut ResidentAllowance,
) -> Option<VerifiedSelectedReleaseHeadCustodyV2> {
    let checkpoint = selection.checkpoint()?;
    let generation = checkpoint.checkpoint().source().root().generation();
    resident
        .transient(u64::from(format.page_size().bytes()).checked_mul(3)?)
        .ok()?;
    budget.consume(1).ok()?;
    let observed = discovery
        .read_root_manifest(generation, u64::from(format.page_size().bytes()))
        .ok()?;
    let admitted = admit_addressed_root(
        RecoveryArtifactNamespaceJoin::from_canonical(&observed),
        discovery.store_identity(),
        format,
        generation,
    )
    .ok()?;
    let (root, observed_format) = admitted.project();
    drop(observed);
    if observed_format != format {
        return None;
    }
    let routes = selected_source_inventory::observe_routes_with_resident_budget(
        discovery, &root, format, budget, trace, resident,
    )
    .ok()?;
    let entries = budget.remaining();
    // Even an empty remainder must produce the same typed resident denial,
    // before constructing walker metadata or encoding the source root.
    resident
        .transient(u64::from(format.page_size().bytes()).checked_mul(2)?)
        .ok()?;
    let memory = resident.remaining();
    // Entry cardinality and resident storage are separate denial dimensions.
    // The physics owner preflights the declared head backing against memory.
    let maximum_heads = entries;
    let limits =
        ReleaseCustodyHeadWalkLimitsV1::new(entries.max(1), maximum_heads, byte_limit, memory, 16)?;
    let roster = match VerifiedCheckpointReleaseHeadRosterV2::admit_checkpoint_source(
        selection,
        &root,
        format,
        limits,
        maximum_heads,
        memory,
        |reference, maximum| {
            budget.consume(1).map_err(|_| ())?;
            discovery
                .read_release_custody_head_block(reference.generation(), reference.block(), maximum)
                .map_err(|_| ())?
                .bytes()
                .map(<[u8]>::to_vec)
                .ok_or(())
        },
    ) {
        Ok(roster) => roster,
        Err(worth_store_recovery_physics::SelectedCustodyDenial::ResidentBoundExceeded {
            required,
            ..
        }) => {
            // Preserve the owner's requested local window alongside the
            // already-live aggregate seed; do not relabel authority failures.
            let _ = resident.transient(required);
            return None;
        }
        Err(_) => return None,
    };
    resident
        .transient(roster.admission_peak_resident_bytes())
        .ok()?;
    resident.bytes(roster.owned_heap_bytes()?).ok()?;
    let controls = super::head_v2_controls::read_checkpoint_source_controls(
        discovery, &routes, &roster, format, budget, trace, scratch, resident,
    )?;
    let decode_peak = controls
        .iter()
        .map(|control| control.manifest().bytes().len())
        .max()
        .unwrap_or(0);
    resident
        .transient(
            u64::try_from(decode_peak)
                .ok()?
                .checked_mul(3)?
                .checked_add(u64::from(format.page_size().bytes()))?,
        )
        .ok()?;
    roster.join_controls(&routes, controls).ok()
}
