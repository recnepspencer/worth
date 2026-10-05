//! Bounded addressed-root and targeted routing evidence for historical
//! publications. A later selected generation alone is never publication proof.

use worth_store::physical_runtime::RecoveryDiscoveryFailure;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, ManifestBlockReference,
    PhysicalTreeIdentity,
};

#[cfg(test)]
#[path = "historical_publication/failure_tests.rs"]
mod failure_tests;
#[path = "historical_publication/route_inventory.rs"]
mod route_inventory;
pub(super) use route_inventory::{
    observe_all_routes, observe_all_routes_of_root, RootRouteInventory,
};

use crate::entry::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension, PhysicalRecoveryLimitFailure,
};
use crate::integrity_ingress::{admit_addressed_root, RecoveryArtifactNamespaceJoin};
use crate::orchestration::planning::{
    context::PlanningContext,
    manifest_entry_budget::ManifestEntryBudget,
    page_observation::{PageObservationFailure, ReaderLimit},
    resolved_basis::ResolvedPlanningBasis,
};
use crate::orchestration::reader_limit::UNCOUNTED_READS;

/// Why a completion phase could not observe history: failed verification,
/// or one of recovery's limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::orchestration::planning) enum HistoricalFailure {
    Invalid,
    ManifestEntries,
    /// The bytes the phase's reader had observed at the crossing, where the
    /// reader counted them.
    ObservationBytes(Option<u64>),
    /// The staging bytes the phase needed.
    StagingBytes(u64),
}

impl HistoricalFailure {
    /// The limit this failure names, with the value recovery admitted.
    /// `remaining_bytes` is what the phase's reader started with.
    pub(in crate::orchestration::planning) fn limit(
        self,
        limits: &PhysicalRecoveryLimitDeclaration,
        remaining_bytes: u64,
    ) -> Option<PhysicalRecoveryLimitFailure> {
        match self {
            Self::Invalid => None,
            Self::ManifestEntries => Some(PhysicalRecoveryLimitFailure {
                dimension: PhysicalRecoveryLimitDimension::ManifestEntries,
                observed: limits.manifest_entries.saturating_add(1),
                admitted: limits.manifest_entries,
            }),
            Self::ObservationBytes(observed) => {
                let admitted = limits.observation_bytes;
                Some(PhysicalRecoveryLimitFailure {
                    dimension: PhysicalRecoveryLimitDimension::ObservationBytes,
                    observed: observed.map_or(admitted.saturating_add(1), |observed| {
                        admitted
                            .saturating_sub(remaining_bytes)
                            .saturating_add(observed)
                    }),
                    admitted,
                })
            }
            Self::StagingBytes(observed) => Some(PhysicalRecoveryLimitFailure {
                dimension: PhysicalRecoveryLimitDimension::StagingBytes,
                observed,
                admitted: limits.staging_bytes,
            }),
        }
    }
}

impl From<PageObservationFailure> for HistoricalFailure {
    fn from(failure: PageObservationFailure) -> Self {
        match failure {
            PageObservationFailure::ManifestEntryLimit => Self::ManifestEntries,
            PageObservationFailure::ByteLimit => Self::ObservationBytes(None),
            _ => Self::Invalid,
        }
    }
}

pub(in crate::orchestration::planning) fn discovery_failure(
    failure: RecoveryDiscoveryFailure,
) -> HistoricalFailure {
    match ReaderLimit::of(&failure) {
        Some(ReaderLimit::Reads { .. }) => HistoricalFailure::ManifestEntries,
        Some(ReaderLimit::ObservationBytes { observed, .. }) => {
            HistoricalFailure::ObservationBytes(Some(observed))
        }
        None => HistoricalFailure::Invalid,
    }
}

/// A completion phase stopped before it could verify: the block names the
/// limit that ran out, and none where verification failed.
/// `remaining_bytes` is what the phase's reader started with.
pub(in crate::orchestration::planning) fn unobserved(
    context: PlanningContext,
    basis: &ResolvedPlanningBasis,
    failure: HistoricalFailure,
    remaining_bytes: u64,
) -> crate::entry::PhysicalRecoveryOutcome {
    let limit = failure.limit(&context.limits, remaining_bytes);
    context.redo_block(basis.planning_counters(), limit)
}

/// The observation bytes left to a completion phase, or the limit that has
/// already run out.
pub(in crate::orchestration::planning) fn remaining_observation(
    context: &PlanningContext,
    basis: &ResolvedPlanningBasis,
) -> Result<u64, HistoricalFailure> {
    let remaining_bytes = context
        .limits
        .observation_bytes
        .saturating_sub(context.counters.bytes_observed)
        .saturating_sub(basis.observed_pages.bytes_read)
        .saturating_sub(basis.observed_pages.candidate_bytes_read)
        .saturating_sub(basis.observed_pages.source_copy_bytes_read)
        .saturating_sub(basis.observed_pages.historical_publication_bytes_read);
    left_to_observe(remaining_bytes)
}

/// Charges what a completion phase's reader read, once it has finished, to
/// the observation bytes every later phase is left with.
pub(in crate::orchestration::planning) fn charge_reader(
    basis: &mut ResolvedPlanningBasis,
    counters: worth_store::physical_runtime::RecoveryDiscoveryCounters,
) {
    let observed = &mut basis.observed_pages;
    observed.historical_publication_reads = observed
        .historical_publication_reads
        .saturating_add(counters.addressed_artifacts_read);
    observed.historical_publication_bytes_read = observed
        .historical_publication_bytes_read
        .saturating_add(counters.bytes_read);
}

/// A phase cannot open its reader with no observation bytes left, so it has
/// met that limit. Entries are refused where they are charged: a phase that
/// charges none is refused none.
fn left_to_observe(bytes: u64) -> Result<u64, HistoricalFailure> {
    if bytes == 0 {
        Err(HistoricalFailure::ObservationBytes(None))
    } else {
        Ok(bytes)
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
    let remaining_bytes = match remaining_observation(&context, basis) {
        Ok(remaining_bytes) => remaining_bytes,
        Err(failure) => return Err(unobserved(context, basis, failure, 0)),
    };
    let format = context.authority.record_format;
    let store = context.authority.media.store_identity();
    let media = context.authority.media;
    let mut discovery = media
        .bounded_discovery(UNCOUNTED_READS, remaining_bytes)
        .expect("admitted nonzero historical publication observation limits");
    let mut callback_scratch = 0;
    let result = (|| {
        let root_source = discovery
            .read_root_manifest(generation, u64::from(format.page_size().bytes()))
            .map_err(discovery_failure)?;
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
    charge_reader(basis, counters);
    basis
        .observed_pages
        .historical_publication_peak_scratch_bytes = basis
        .observed_pages
        .historical_publication_peak_scratch_bytes
        .max(u64::from(format.page_size().bytes()) * 2 + callback_scratch);
    let value = match result {
        Ok(value) => value,
        Err(failure) => return Err(unobserved(context, basis, failure, remaining_bytes)),
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
    // One lookup charges one entry, however many blocks its path crosses
    // and however many neighbors share its leaf.
    budget.consume(1)?;
    let mut reference = root
        .routing_root()
        .filter(|reference| reference.contains(record));
    let mut previous_level = None;
    while let Some(current) = reference {
        if previous_level.is_some_and(|level| current.level() >= level) {
            return Err(HistoricalFailure::Invalid);
        }
        previous_level = Some(current.level());
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
