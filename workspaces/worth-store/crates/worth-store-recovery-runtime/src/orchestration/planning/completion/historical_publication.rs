//! Bounded addressed-root and targeted routing evidence for historical
//! publications. A later selected generation alone is never publication proof.

use worth_store::physical_runtime::{
    ArtifactCeiling, BoundedRecoveryFilesystemDiscovery, PageAddress, ReadGrant,
    RecoveryDiscoveryFailure, UnchargedRead,
};
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, ManifestBlockReference,
    PhysicalRecordFormatDeclaration, PhysicalTreeIdentity,
};

#[cfg(test)]
#[path = "historical_publication/failure_tests.rs"]
mod failure_tests;
#[cfg(test)]
#[path = "historical_publication/root_charge_tests.rs"]
mod root_charge_tests;
#[path = "historical_publication/route_inventory.rs"]
mod route_inventory;
pub(super) use route_inventory::{
    observe_all_routes, observe_all_routes_of_root, RootRouteInventory,
};

use crate::entry::{PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitFailure};
use crate::integrity_ingress::{admit_addressed_root, RecoveryArtifactNamespaceJoin};
use crate::orchestration::planning::{
    context::PlanningContext,
    manifest_entry_budget::{
        pays_for, spend, ChargeTarget, ChargeToken, EntriesStopped, EntryAdmission,
        ManifestEntryBudget, ROOT_ENTRY,
    },
    page_observation::{PageLimit, PageObservationFailure},
    resolved_basis::ResolvedPlanningBasis,
};
use crate::orchestration::reader_limit::{ReaderBytes, UNCOUNTED_READS};

/// Why a completion phase could not observe history: failed verification,
/// or one of recovery's limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::orchestration::planning) enum HistoricalFailure {
    Invalid,
    /// A limit with its own counts.
    Limit(PageLimit),
    /// A public denial that names the manifest entry limit without counts:
    /// the phase's budget refused, and holds them.
    ManifestEntries,
    /// A count the phase kept passed every count, so no limit can state it.
    CountOverflow,
}

impl HistoricalFailure {
    /// The limit this failure names, in recovery's own counts. `budget` is
    /// the manifest entry budget the phase charged.
    pub(in crate::orchestration::planning) fn limit(
        self,
        limits: &PhysicalRecoveryLimitDeclaration,
        budget: &ManifestEntryBudget,
    ) -> Option<PhysicalRecoveryLimitFailure> {
        match self {
            Self::Invalid | Self::CountOverflow => None,
            Self::Limit(limit) => limit.in_recovery(limits),
            Self::ManifestEntries => budget.refused().map(Into::into),
        }
    }
}

impl From<PageObservationFailure> for HistoricalFailure {
    fn from(failure: PageObservationFailure) -> Self {
        use PageObservationFailure as Page;
        match failure {
            Page::Limit(limit) => Self::Limit(limit),
            Page::CountOverflow => Self::CountOverflow,
            Page::Media { .. }
            | Page::MissingArtifact { .. }
            | Page::InvalidManifest { .. }
            | Page::Integrity { .. }
            | Page::InvalidTarget(_)
            | Page::HistoricalDrop { .. }
            | Page::AbsentExtentBelowFrontier { .. }
            | Page::MaterializedExtentChunkCount { .. }
            | Page::MaterializedExtentCoordinate(_)
            | Page::InvalidPage(_) => Self::Invalid,
        }
    }
}

impl From<EntriesStopped> for HistoricalFailure {
    fn from(stopped: EntriesStopped) -> Self {
        match stopped {
            EntriesStopped::Limit(limit) => Self::Limit(PageLimit::Entries(limit)),
            EntriesStopped::CountOverflow => Self::CountOverflow,
        }
    }
}

/// Only a reader out of its observation bytes is a limit: a phase's reader
/// counts no reads, and any other failed read is damage.
pub(in crate::orchestration::planning) fn discovery_failure(
    failure: RecoveryDiscoveryFailure,
) -> HistoricalFailure {
    ReaderBytes::of(&failure).map_or(HistoricalFailure::Invalid, |bytes| {
        HistoricalFailure::Limit(PageLimit::Reader(bytes))
    })
}

/// A completion phase stopped before it could verify: the block names the
/// limit that ran out, and none where verification failed.
pub(in crate::orchestration::planning) fn unobserved(
    context: PlanningContext,
    basis: &ResolvedPlanningBasis,
    failure: HistoricalFailure,
) -> crate::entry::PhysicalRecoveryOutcome {
    let limit = failure.limit(&context.limits, &basis.observed_pages.manifest_budget);
    context.redo_block(basis.planning_counters(), limit)
}

/// The observation bytes left to a completion phase. None left refuses
/// nothing yet: the phase's first read past them is refused, with its real
/// length.
pub(in crate::orchestration::planning) fn remaining_observation(
    context: &PlanningContext,
    basis: &ResolvedPlanningBasis,
) -> u64 {
    context
        .limits
        .observation_bytes
        .saturating_sub(context.counters.bytes_observed)
        .saturating_sub(basis.observed_pages.bytes_read)
        .saturating_sub(basis.observed_pages.candidate_bytes_read)
        .saturating_sub(basis.observed_pages.source_copy_bytes_read)
        .saturating_sub(basis.observed_pages.historical_publication_bytes_read)
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

/// The addressed root at `generation` and the route of `record` under it,
/// handed to `validate`. The root's entry is spent once it is read.
pub(super) fn observe<R>(
    context: PlanningContext,
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
    observe_charged(
        context,
        basis,
        generation,
        record,
        |discovery, root, route, charge, budget, trace, scratch| {
            spend(charge, root.generation());
            validate(discovery, root, route, budget, trace, scratch)
        },
    )
}

/// Charges the root at `generation` its one entry before reading it, then
/// hands the token on with the root, for a walk under it that the same entry
/// pays for.
pub(super) fn observe_charged<R>(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    generation: u64,
    record: worth_store_physical_format::PersistedRecordIdentity,
    validate: impl FnOnce(
        &mut worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery,
        &DurablePhysicalRootManifest,
        Option<CurrentPhysicalRecordPlacement>,
        ChargeToken,
        &mut ManifestEntryBudget,
        &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
        &mut u64,
    ) -> Result<R, HistoricalFailure>,
) -> Result<(PlanningContext, R), crate::entry::PhysicalRecoveryOutcome> {
    let remaining_bytes = remaining_observation(&context, basis);
    let format = context.authority.record_format;
    let media = context.authority.media;
    let mut discovery = media
        .bounded_discovery(UNCOUNTED_READS, remaining_bytes)
        .expect("a reader that counts no reads opens on any byte bound");
    let mut callback_scratch = 0;
    let result = (|| {
        let (root, charge) = charged_root(
            &mut discovery,
            &mut basis.observed_pages.manifest_budget,
            format,
            generation,
        )?;
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
            charge,
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
        Err(failure) => return Err(unobserved(context, basis, failure)),
    };
    Ok((context, value))
}

/// The addressed root at `generation`, charged its one entry before it is
/// read. The token goes on with the root, for the reads under it.
fn charged_root(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    budget: &mut ManifestEntryBudget,
    format: PhysicalRecordFormatDeclaration,
    generation: u64,
) -> Result<(DurablePhysicalRootManifest, ChargeToken), HistoricalFailure> {
    let charge = budget.charge(ROOT_ENTRY, ChargeTarget::root(generation))?;
    pays_for(&charge, generation);
    let root_source = discovery
        .read(
            ArtifactCeiling::page(format, PageAddress::RootManifest { generation }),
            ReadGrant::ceiling_only(),
        )
        .observed()
        .map_err(discovery_failure)?;
    let admitted = admit_addressed_root(
        RecoveryArtifactNamespaceJoin::from_canonical(&root_source),
        discovery.store_identity(),
        format,
        generation,
    )
    .map_err(|_| HistoricalFailure::Invalid)?;
    let (root, observed_format) = admitted.project();
    if observed_format != format || root.generation() != generation {
        return Err(HistoricalFailure::Invalid);
    }
    Ok((root, charge))
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
    budget.admit(1)?;
    let mut reference = root
        .routing_root()
        .filter(|reference| reference.contains(record));
    let mut previous_level = None;
    while let Some(current) = reference {
        if previous_level.is_some_and(|level| current.level() >= level) {
            return Err(HistoricalFailure::Invalid);
        }
        previous_level = Some(current.level());
        let address = PageAddress::RootRoutingBlock {
            generation: current.generation(),
            block: current.block(),
        };
        let observed = discovery
            .read(
                ArtifactCeiling::page(format, address),
                ReadGrant::ceiling_only(),
            )
            .observed()
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
