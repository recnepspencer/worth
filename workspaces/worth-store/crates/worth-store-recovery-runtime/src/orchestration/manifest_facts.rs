use std::collections::{BTreeSet, VecDeque};

use worth_store::physical_runtime::{
    ArtifactCeiling, BoundedRecoveryFilesystemDiscovery, GrantedRead, GrantedReadStop, PageAddress,
};
use worth_store_physical_format::{
    ManifestBlockReference, PhysicalRootRoutingBlock, PhysicalTreeIdentity,
};
use worth_store_recovery_physics::{
    PhysicalManifestBlockProjection, PhysicalRootSlotObservation, PhysicalRootSourceCandidate,
};

use crate::entry::{
    PhysicalManifestObservationDenial, PhysicalRecoveryBlockKind as PhysicalRecoveryBlock,
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension,
};
use crate::orchestration::discovery::DiscoveryFailure;
use crate::orchestration::discovery::OversizedArtifact;
use crate::orchestration::recovery_budget::{RecoveryAllowance, RecoveryReadBudget};

pub(crate) enum ManifestFactsState {
    Unavailable,
    Rejected(PhysicalManifestObservationDenial),
    Observed {
        blocks: Vec<PhysicalManifestBlockProjection>,
    },
}

pub(crate) struct ManifestFactsDiscovery {
    state: ManifestFactsState,
    integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

/// What is left of recovery's manifest bytes and entries, shared by the
/// current and previous roots; `limits` holds the whole of each.
pub(super) struct ManifestObservationBudget<'a> {
    pub limits: PhysicalRecoveryLimitDeclaration,
    pub bytes: &'a mut RecoveryReadBudget,
    pub remaining_entries: &'a mut u64,
    pub blocks_read: &'a mut u64,
}

pub(super) fn observe_manifest_facts(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &PhysicalRootSlotObservation,
    mut budget: ManifestObservationBudget<'_>,
) -> Result<ManifestFactsDiscovery, DiscoveryFailure> {
    let PhysicalRootSlotObservation::Candidate(root) = root else {
        return Ok(ManifestFactsDiscovery::unavailable());
    };
    // The verified manifest counts the tree's entries, so the caller's entry
    // budget decides on that count before any routing block is read. The
    // walk is then bounded by the artifact alone: the root's level is the
    // format's height for that count, every child is one level lower, and
    // no block holds more than the node capacity, so no admitted tree has
    // more blocks than a full one of that height. Its leaves can still hold
    // more entries than it counts, and that is damage.
    let record_count = root.manifest().record_count();
    charge_record_count(record_count, &mut budget)?;
    let mut entries = 0_u64;
    let mut pending = root
        .manifest()
        .routing_root()
        .into_iter()
        .collect::<VecDeque<_>>();
    let mut visited = BTreeSet::new();
    let mut candidates = Vec::new();
    let mut integrity_trace = crate::integrity_ingress::RecoveryIntegrityIngressTrace::default();
    while let Some(reference) = pending.pop_front() {
        if !visited.insert((reference.generation(), reference.block())) {
            return Ok(ManifestFactsDiscovery::rejected(
                PhysicalManifestObservationDenial::DuplicateReference { reference },
                integrity_trace,
            ));
        }
        let observed = match observe_manifest_block(
            discovery,
            root,
            reference,
            &mut budget,
            &mut integrity_trace,
        ) {
            Ok(observed) => observed,
            Err(failure) => return Err(failure.with_integrity_trace(integrity_trace)),
        };
        let projected = match observed {
            Ok(observed) => observed,
            Err(denial) => {
                return Ok(ManifestFactsDiscovery::rejected(denial, integrity_trace));
            }
        };
        match &projected.block {
            PhysicalRootRoutingBlock::Branch { children, .. } => {
                pending.extend(children.iter().copied());
            }
            PhysicalRootRoutingBlock::Leaf { entries: leaf, .. } => {
                entries = entries.saturating_add(leaf.len() as u64);
                if entries > record_count {
                    return Ok(ManifestFactsDiscovery::rejected(
                        PhysicalManifestObservationDenial::RecordCountCeiling {
                            observed: entries,
                            admitted: record_count,
                        },
                        integrity_trace,
                    ));
                }
            }
        }
        candidates.push(projected.page_facts);
    }
    Ok(ManifestFactsDiscovery::observed(
        candidates,
        integrity_trace,
    ))
}

fn observe_manifest_block(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &PhysicalRootSourceCandidate,
    reference: ManifestBlockReference,
    budget: &mut ManifestObservationBudget<'_>,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<
    Result<
        crate::integrity_ingress::projection::AdmittedRootRoutingProjection,
        PhysicalManifestObservationDenial,
    >,
    DiscoveryFailure,
> {
    // A routing block is one page of its root's format, and spends
    // recovery's manifest bytes.
    let ceiling = ArtifactCeiling::page(
        root.selector().format(),
        PageAddress::RootRoutingBlock {
            generation: reference.generation(),
            block: reference.block(),
        },
    );
    let artifact = match discovery.read(ceiling, budget.bytes.grant()).granted() {
        Ok(artifact) => artifact,
        Err(GrantedReadStop::PastGrant(overrun)) => {
            return Err(super::discovery::past_grant(budget.bytes, overrun));
        }
        Err(GrantedReadStop::Unread(failure)) => {
            let OversizedArtifact = super::discovery::unread(failure, &budget.limits)?;
            return Ok(Err(PhysicalManifestObservationDenial::Integrity {
                reference,
                denial: crate::entry::PhysicalRecoveryRootProtocolDenial::NonCanonicalEncoding,
            }));
        }
    };
    budget.bytes.charge(&artifact);
    if artifact.bytes().is_some() {
        *budget.blocks_read += 1;
    }
    match admit_manifest_block(
        &artifact,
        discovery.store_identity(),
        root,
        reference,
        integrity_trace,
    ) {
        Ok(observed) => Ok(Ok(observed)),
        Err(ManifestBlockObservationFailure::Format(denial)) => Ok(Err(denial)),
    }
}

fn admit_manifest_block(
    source: &worth_store::physical_runtime::ObservedRecoveryArtifact,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    root: &PhysicalRootSourceCandidate,
    reference: ManifestBlockReference,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<
    crate::integrity_ingress::projection::AdmittedRootRoutingProjection,
    ManifestBlockObservationFailure,
> {
    let tree = PhysicalTreeIdentity::new(root.manifest().tree_identity()).ok_or_else(|| {
        ManifestBlockObservationFailure::Format(PhysicalManifestObservationDenial::Integrity {
            reference,
            denial: crate::entry::PhysicalRecoveryRootProtocolDenial::ScopeMismatch,
        })
    })?;
    crate::integrity_ingress::projection::root_routing_block(
        source,
        store,
        root.selector().format(),
        tree,
        reference,
        root.manifest().node_capacity(),
        integrity_trace,
    )
    .map_err(|rejection| {
        ManifestBlockObservationFailure::Format(PhysicalManifestObservationDenial::Integrity {
            reference,
            denial: rejection.diagnostic(),
        })
    })
}

enum ManifestBlockObservationFailure {
    Format(PhysicalManifestObservationDenial),
}

/// Charges a verified manifest's record count to the caller's entry budget,
/// shared by the current and previous roots.
fn charge_record_count(
    record_count: u64,
    budget: &mut ManifestObservationBudget<'_>,
) -> Result<(), DiscoveryFailure> {
    let Some(remaining) = budget.remaining_entries.checked_sub(record_count) else {
        // Recovery's entries less those this observation was handed were
        // charged before it.
        return Err(super::discovery::refused_beside(
            RecoveryAllowance::declared(
                &budget.limits,
                PhysicalRecoveryLimitDimension::ManifestEntries,
            ),
            record_count,
            *budget.remaining_entries,
            PhysicalRecoveryBlock::RootProtocol,
        ));
    };
    *budget.remaining_entries = remaining;
    Ok(())
}

impl ManifestFactsDiscovery {
    const fn unavailable() -> Self {
        Self {
            state: ManifestFactsState::Unavailable,
            integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace::new(),
        }
    }

    const fn rejected(
        denial: PhysicalManifestObservationDenial,
        integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    ) -> Self {
        Self {
            state: ManifestFactsState::Rejected(denial),
            integrity_trace,
        }
    }

    const fn observed(
        blocks: Vec<PhysicalManifestBlockProjection>,
        integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    ) -> Self {
        Self {
            state: ManifestFactsState::Observed { blocks },
            integrity_trace,
        }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        ManifestFactsState,
        crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    ) {
        (self.state, self.integrity_trace)
    }

    pub(super) const fn integrity_trace(
        &self,
    ) -> &crate::integrity_ingress::RecoveryIntegrityIngressTrace {
        &self.integrity_trace
    }
}

#[cfg(test)]
mod tests;
