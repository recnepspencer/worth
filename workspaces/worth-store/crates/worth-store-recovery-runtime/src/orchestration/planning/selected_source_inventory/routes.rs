//! Complete, integrity-witnessed routing traversal for an addressed root.
//! Shared by historical completion and preplanning root-chain admission.

use std::collections::{BTreeSet, VecDeque};
use std::convert::Infallible;

use worth_store::physical_runtime::{BoundedRecoveryFilesystemDiscovery, PageAddress};
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    PhysicalTreeIdentity, RecordArtifactFile,
};

use super::{invalid, PageObservationFailure, ResidentAllowance, ResidentTraceDenial};
use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::orchestration::planning::manifest_entry_budget::{
    spend, ChargeToken, EntriesStopped, EntryAdmission,
};

/// What a routes traversal keeps in memory, told to whoever bounds that.
pub(in crate::orchestration::planning) trait RoutesHold {
    type Refused;
    fn block(&mut self, format: PhysicalRecordFormatDeclaration) -> Result<(), Self::Refused>;
    fn trace_slot(
        &mut self,
        trace: &mut RecoveryIntegrityIngressTrace,
    ) -> Result<(), Self::Refused>;
    fn entries(&mut self, count: usize, entry_bytes: usize) -> Result<(), Self::Refused>;
}

/// A traversal bounded by its entries and observation bytes alone.
struct Unheld;

impl RoutesHold for Unheld {
    type Refused = Infallible;

    fn block(&mut self, _: PhysicalRecordFormatDeclaration) -> Result<(), Infallible> {
        Ok(())
    }

    fn trace_slot(&mut self, _: &mut RecoveryIntegrityIngressTrace) -> Result<(), Infallible> {
        Ok(())
    }

    fn entries(&mut self, _: usize, _: usize) -> Result<(), Infallible> {
        Ok(())
    }
}

impl RoutesHold for ResidentAllowance {
    type Refused = ResidentTraceDenial;

    fn block(&mut self, format: PhysicalRecordFormatDeclaration) -> Result<(), Self::Refused> {
        ResidentAllowance::block(self, format).map_err(ResidentTraceDenial::from)
    }

    fn trace_slot(
        &mut self,
        trace: &mut RecoveryIntegrityIngressTrace,
    ) -> Result<(), Self::Refused> {
        self.trace_slots(trace, 1)
    }

    fn entries(&mut self, count: usize, entry_bytes: usize) -> Result<(), Self::Refused> {
        ResidentAllowance::entries(self, count, entry_bytes).map_err(ResidentTraceDenial::from)
    }
}

/// Why a routes traversal stopped: what it observed, or what it was refused
/// room to hold. The second is never an entry or byte limit.
pub(in crate::orchestration::planning) enum RoutesFailure<R> {
    Observation(PageObservationFailure),
    Held(R),
}

impl<R> From<PageObservationFailure> for RoutesFailure<R> {
    fn from(failure: PageObservationFailure) -> Self {
        Self::Observation(failure)
    }
}

impl<R> From<EntriesStopped> for RoutesFailure<R> {
    fn from(stopped: EntriesStopped) -> Self {
        Self::Observation(stopped.into())
    }
}

pub(in crate::orchestration::planning) fn observe_routes_with_budget(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    charge: ChargeToken,
    entries: &mut impl EntryAdmission,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> Result<Vec<CurrentPhysicalRecordPlacement>, PageObservationFailure> {
    observe_routes_held(discovery, root, format, charge, entries, trace, &mut Unheld).map_err(
        |failure| match failure {
            RoutesFailure::Observation(failure) => failure,
            RoutesFailure::Held(never) => match never {},
        },
    )
}

/// Every route under `root`, the last read `charge` pays for, each leaf
/// entry admitted to `budget`.
pub(in crate::orchestration::planning) fn observe_routes_held<H: RoutesHold>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    charge: ChargeToken,
    budget: &mut impl EntryAdmission,
    trace: &mut RecoveryIntegrityIngressTrace,
    hold: &mut H,
) -> Result<Vec<CurrentPhysicalRecordPlacement>, RoutesFailure<H::Refused>> {
    spend(charge, root.generation());
    let root_artifact = RecordArtifactFile::RootManifest {
        generation: root.generation(),
    };
    let tree =
        PhysicalTreeIdentity::new(root.tree_identity()).ok_or_else(|| invalid(root_artifact))?;
    let mut pending = root.routing_root().into_iter().collect::<VecDeque<_>>();
    let mut visited = BTreeSet::new();
    let mut entries = Vec::new();
    while let Some(reference) = pending.pop_front() {
        hold.block(format).map_err(RoutesFailure::Held)?;
        let artifact = RecordArtifactFile::RootRoutingBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        // Two references that reach one block contradict the tree.
        if !visited.insert((reference.generation(), reference.block())) {
            return Err(invalid(artifact).into());
        }
        hold.trace_slot(trace).map_err(RoutesFailure::Held)?;
        let address = PageAddress::RootRoutingBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        let observed = super::required_source(super::read_page(discovery, format, address), None)?;
        let projected = crate::integrity_ingress::projection::root_routing_block(
            &observed,
            discovery.store_identity(),
            format,
            tree,
            reference,
            root.node_capacity(),
            trace,
        )
        .map_err(|denial| PageObservationFailure::Integrity {
            artifact,
            denial: denial.diagnostic(),
        })?;
        if let Some(found) = projected.block.entries() {
            budget.admit(found.len())?;
            hold.entries(
                found.len(),
                std::mem::size_of::<CurrentPhysicalRecordPlacement>(),
            )
            .map_err(RoutesFailure::Held)?;
            entries.extend_from_slice(found);
        } else if let Some(children) = projected.block.children() {
            hold.entries(
                children.len(),
                std::mem::size_of::<worth_store_physical_format::ManifestBlockReference>(),
            )
            .map_err(RoutesFailure::Held)?;
            pending.extend(children.iter().copied());
        } else {
            return Err(invalid(artifact).into());
        }
    }
    entries.sort_unstable_by_key(|route| route.record());
    if entries.len() as u64 != root.record_count()
        || entries
            .windows(2)
            .any(|pair| pair[0].record() == pair[1].record())
    {
        // The root counts records its tree does not route, or routes one twice.
        return Err(invalid(root_artifact).into());
    }
    Ok(entries)
}

#[cfg(test)]
#[path = "routes/tests.rs"]
mod tests;
