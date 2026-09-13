use std::collections::{BTreeSet, VecDeque};

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    ManifestBlockReference, PhysicalRootRoutingBlock, PhysicalTreeIdentity,
};
use worth_store_recovery_physics::{
    PhysicalManifestBlockProjection, PhysicalRootSlotObservation, PhysicalRootSourceCandidate,
};

use crate::entry::{
    PhysicalManifestObservationDenial, PhysicalRecoveryBlockKind as PhysicalRecoveryBlock,
    PhysicalRecoveryLimitDimension,
};
use crate::orchestration::discovery::DiscoveryFailure;

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

pub(super) struct ManifestObservationBudget<'a> {
    pub remaining_bytes: &'a mut u64,
    pub admitted_bytes: u64,
    pub remaining_entries: &'a mut u64,
    pub admitted_entries: u64,
    pub remaining_blocks: &'a mut u64,
    pub admitted_blocks: u64,
}

pub(super) fn observe_manifest_facts(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &PhysicalRootSlotObservation,
    mut budget: ManifestObservationBudget<'_>,
) -> Result<ManifestFactsDiscovery, DiscoveryFailure> {
    let PhysicalRootSlotObservation::Candidate(root) = root else {
        return Ok(ManifestFactsDiscovery::unavailable());
    };
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
        let queued_blocks = pending.len() as u64;
        let observed = match observe_manifest_block(
            discovery,
            root,
            reference,
            queued_blocks,
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
        let block = projected.block;
        if let PhysicalRootRoutingBlock::Branch { children, .. } = &block {
            pending.extend(children.iter().copied());
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
    queued_blocks: u64,
    budget: &mut ManifestObservationBudget<'_>,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<
    Result<
        crate::integrity_ingress::projection::AdmittedRootRoutingProjection,
        PhysicalManifestObservationDenial,
    >,
    DiscoveryFailure,
> {
    consume_block_budget(budget)?;
    let block_byte_limit =
        u64::from(root.selector().format().page_size().bytes()).min(*budget.remaining_bytes);
    let artifact = discovery
        .read_root_routing_block(reference.generation(), reference.block(), block_byte_limit)
        .map_err(|failure| {
            super::discovery::map_cumulative_discovery_failure(
                failure,
                PhysicalRecoveryLimitDimension::ManifestEntries,
                PhysicalRecoveryLimitDimension::ManifestBytes,
                budget.admitted_bytes,
                *budget.remaining_bytes,
            )
        })?;
    let observed_bytes = artifact.bytes().map_or(0, |bytes| bytes.len() as u64);
    *budget.remaining_bytes = budget
        .remaining_bytes
        .checked_sub(observed_bytes)
        .ok_or_else(|| DiscoveryFailure::from(PhysicalRecoveryBlock::DiscoveryLimit))?;
    let observed = match admit_manifest_block(
        &artifact,
        discovery.store_identity(),
        root,
        reference,
        integrity_trace,
    ) {
        Ok(observed) => observed,
        Err(ManifestBlockObservationFailure::Format(denial)) => return Ok(Err(denial)),
    };
    if let Some(children) = observed.block.children() {
        let remaining = budget.remaining_blocks.saturating_sub(queued_blocks);
        if children.len() as u64 > remaining {
            let consumed = budget
                .admitted_blocks
                .saturating_sub(*budget.remaining_blocks);
            return Err(super::discovery::discovery_limit(
                PhysicalRecoveryLimitDimension::ManifestEntries,
                consumed
                    .saturating_add(queued_blocks)
                    .saturating_add(children.len() as u64),
                budget.admitted_blocks,
            ));
        }
    }
    consume_entry_budget(&observed.block, budget)?;
    Ok(Ok(observed))
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

fn consume_block_budget(
    budget: &mut ManifestObservationBudget<'_>,
) -> Result<(), DiscoveryFailure> {
    if *budget.remaining_blocks == 0 {
        return Err(super::discovery::discovery_limit(
            PhysicalRecoveryLimitDimension::ManifestEntries,
            budget.admitted_blocks.saturating_add(1),
            budget.admitted_blocks,
        ));
    }
    *budget.remaining_blocks -= 1;
    Ok(())
}

fn consume_entry_budget(
    block: &PhysicalRootRoutingBlock,
    budget: &mut ManifestObservationBudget<'_>,
) -> Result<(), DiscoveryFailure> {
    let entry_count = match block {
        PhysicalRootRoutingBlock::Leaf { entries, .. } => entries.len() as u64,
        PhysicalRootRoutingBlock::Branch { .. } => 0,
    };
    if entry_count > *budget.remaining_entries {
        return Err(super::discovery::discovery_limit(
            PhysicalRecoveryLimitDimension::ManifestEntries,
            budget
                .admitted_entries
                .saturating_sub(*budget.remaining_entries)
                .saturating_add(entry_count),
            budget.admitted_entries,
        ));
    }
    *budget.remaining_entries -= entry_count;
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

    pub(crate) fn block_count(&self) -> u64 {
        match &self.state {
            ManifestFactsState::Observed { blocks } => blocks.len() as u64,
            ManifestFactsState::Unavailable | ManifestFactsState::Rejected(_) => 0,
        }
    }

    pub(super) const fn integrity_trace(
        &self,
    ) -> &crate::integrity_ingress::RecoveryIntegrityIngressTrace {
        &self.integrity_trace
    }
}
