//! Bounded addressed-root and targeted routing evidence for historical
//! publications. A later selected generation alone is never publication proof.

use worth_store::physical_runtime::{RecoveryDiscoveryByteLimitScope, RecoveryDiscoveryFailure};
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, ManifestBlockReference,
    PhysicalTreeIdentity,
};

#[path = "historical_publication/route_inventory.rs"]
mod route_inventory;
pub(super) use route_inventory::observe_all_routes;

use crate::integrity_ingress::{admit_addressed_root, RecoveryArtifactNamespaceJoin};
use crate::orchestration::planning::{
    context::PlanningContext, manifest_entry_budget::ManifestEntryBudget,
    resolved_basis::ResolvedPlanningBasis,
};

pub(in crate::orchestration::planning) enum HistoricalFailure {
    Invalid,
    ManifestEntries,
    ObservationBytes(u64),
}

pub(super) fn discovery_failure(failure: RecoveryDiscoveryFailure) -> HistoricalFailure {
    match failure {
        RecoveryDiscoveryFailure::EntryLimitExceeded { .. } => HistoricalFailure::ManifestEntries,
        RecoveryDiscoveryFailure::ByteLimitExceeded {
            observed,
            scope: RecoveryDiscoveryByteLimitScope::Observation,
            ..
        } => HistoricalFailure::ObservationBytes(observed),
        _ => HistoricalFailure::Invalid,
    }
}

pub(super) fn observe<R>(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    generation: u64,
    record: worth_store_physical_format::PersistedRecordIdentity,
    validate: impl FnOnce(
        &mut worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery,
        &DurablePhysicalRootManifest,
        Option<CurrentPhysicalRecordPlacement>,
        &mut ManifestEntryBudget,
        &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
        &mut u64,
    ) -> Result<R, HistoricalFailure>,
) -> Result<(PlanningContext, R), crate::entry::PhysicalRecoveryOutcome> {
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
        let limit = crate::entry::PhysicalRecoveryLimitFailure {
            dimension: if remaining_entries == 0 {
                crate::entry::PhysicalRecoveryLimitDimension::ManifestEntries
            } else {
                crate::entry::PhysicalRecoveryLimitDimension::ObservationBytes
            },
            observed: if remaining_entries == 0 {
                context.limits.manifest_entries.saturating_add(1)
            } else {
                context.limits.observation_bytes.saturating_add(1)
            },
            admitted: if remaining_entries == 0 {
                context.limits.manifest_entries
            } else {
                context.limits.observation_bytes
            },
        };
        return Err(context.redo_block(basis.planning_counters(), Some(limit)));
    }
    let format = context.authority.record_format;
    let store = context.authority.media.store_identity();
    let media = context.authority.media;
    let mut discovery = media
        .bounded_discovery(remaining_entries, remaining_bytes)
        .expect("admitted nonzero historical publication observation limits");
    let mut callback_scratch = 0;
    let result = (|| {
        let root_source = discovery
            .read_root_manifest(generation, u64::from(format.page_size().bytes()))
            .map_err(discovery_failure)?;
        basis
            .observed_pages
            .manifest_budget
            .consume(1)
            .map_err(|_| HistoricalFailure::ManifestEntries)?;
        let admitted = admit_addressed_root(
            RecoveryArtifactNamespaceJoin::from_canonical(&root_source),
            store,
            format,
            generation,
        )
        .map_err(|_| HistoricalFailure::Invalid)?;
        let (root, observed_format) = admitted.project();
        if observed_format != format || root.generation() != generation {
            return Err(HistoricalFailure::Invalid);
        }
        drop(root_source);
        let route = find_route(
            &mut discovery,
            &root,
            record,
            format,
            &mut basis.observed_pages.manifest_budget,
            &mut context.integrity_trace,
            &mut callback_scratch,
        )?;
        validate(
            &mut discovery,
            &root,
            route,
            &mut basis.observed_pages.manifest_budget,
            &mut context.integrity_trace,
            &mut callback_scratch,
        )
    })();
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
        .max(u64::from(format.page_size().bytes()) * 2 + callback_scratch);
    let value = match result {
        Ok(value) => value,
        Err(failure) => {
            let limit = match failure {
                HistoricalFailure::Invalid => None,
                HistoricalFailure::ManifestEntries => {
                    Some(crate::entry::PhysicalRecoveryLimitFailure {
                        dimension: crate::entry::PhysicalRecoveryLimitDimension::ManifestEntries,
                        observed: context.limits.manifest_entries.saturating_add(1),
                        admitted: context.limits.manifest_entries,
                    })
                }
                HistoricalFailure::ObservationBytes(observed) => {
                    let admitted = context.limits.observation_bytes;
                    Some(crate::entry::PhysicalRecoveryLimitFailure {
                        dimension: crate::entry::PhysicalRecoveryLimitDimension::ObservationBytes,
                        observed: admitted
                            .saturating_sub(remaining_bytes)
                            .saturating_add(observed),
                        admitted,
                    })
                }
            };
            return Err(context.redo_block(basis.planning_counters(), limit));
        }
    };
    Ok((context, value))
}

pub(super) fn find_route(
    discovery: &mut worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    record: worth_store_physical_format::PersistedRecordIdentity,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
) -> Result<Option<CurrentPhysicalRecordPlacement>, HistoricalFailure> {
    let mut reference = root
        .routing_root()
        .filter(|reference| reference.contains(record));
    let mut previous_level = None;
    while let Some(current) = reference {
        if previous_level.is_some_and(|level| current.level() >= level) {
            return Err(HistoricalFailure::Invalid);
        }
        previous_level = Some(current.level());
        budget
            .consume(1)
            .map_err(|_| HistoricalFailure::ManifestEntries)?;
        let observed = discovery
            .read_root_routing_block(
                current.generation(),
                current.block(),
                u64::from(format.page_size().bytes()),
            )
            .map_err(discovery_failure)?;
        let tree =
            PhysicalTreeIdentity::new(root.tree_identity()).ok_or(HistoricalFailure::Invalid)?;
        let projected = crate::integrity_ingress::projection::root_routing_block(
            &observed,
            discovery.store_identity(),
            format,
            tree,
            current,
            root.node_capacity(),
            trace,
        )
        .map_err(|_| HistoricalFailure::Invalid)?;
        if let Some(entries) = projected.block.entries() {
            *scratch = (*scratch).max(
                (entries.len() * std::mem::size_of::<CurrentPhysicalRecordPlacement>()) as u64,
            );
            budget
                .consume(entries.len())
                .map_err(|_| HistoricalFailure::ManifestEntries)?;
            return Ok(entries
                .iter()
                .copied()
                .find(|entry| entry.record() == record));
        }
        let children = projected
            .block
            .children()
            .ok_or(HistoricalFailure::Invalid)?;
        *scratch =
            (*scratch).max((children.len() * std::mem::size_of::<ManifestBlockReference>()) as u64);
        budget
            .consume(children.len())
            .map_err(|_| HistoricalFailure::ManifestEntries)?;
        reference = unique_child(children, record)?;
    }
    Ok(None)
}

fn unique_child(
    children: &[ManifestBlockReference],
    record: worth_store_physical_format::PersistedRecordIdentity,
) -> Result<Option<ManifestBlockReference>, HistoricalFailure> {
    let mut matching = children
        .iter()
        .copied()
        .filter(|child| child.contains(record));
    let first = matching.next();
    if matching.next().is_some() {
        return Err(HistoricalFailure::Invalid);
    }
    Ok(first)
}
