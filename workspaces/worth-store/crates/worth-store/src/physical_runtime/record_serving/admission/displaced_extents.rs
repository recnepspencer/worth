use std::collections::HashMap;
use std::mem::size_of;

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, DurablePhysicalRootManifest,
    ExtentArenaRange, FreeSpaceBlockReference, ManifestBlockReference, PhysicalRootRoutingBlock,
    RecordFreeSpaceManifestEntry,
};

use super::super::access::manifest_routing::{ManifestDiscoveryCounterSnapshot, ManifestReader};
use super::bootstrap::{BootstrapTransitionFailure, RecordBootstrapDenial};
use super::displaced_segments::membership_failure;
use crate::physical_runtime::durability::{DisplacedArtifact, RetiredArtifact};

pub(super) fn range_was_published_free(
    admission: &super::open::CurrentRootAdmission<'_>,
    root: &DurablePhysicalRootManifest,
    range: ExtentArenaRange,
    source_root: u64,
    format: super::super::AdmittedPhysicalRecordFormat,
    access: super::super::AdmittedRecordAccessPolicy,
) -> Result<bool, BootstrapTransitionFailure> {
    let historical = super::open::CurrentRootAdmission {
        generation: root.generation(),
        lifecycle: std::sync::Arc::clone(&admission.lifecycle),
        ..admission.clone()
    };
    let header = super::current_free_space::load_free_space_manifest(&historical, root)?;
    let reader = super::super::planning::free_space_routing::FreeSpaceReader::with_loader(
        admission.media,
        admission.loader,
        format,
        access,
        &header,
        std::sync::Arc::clone(&admission.lifecycle),
        admission.resident_integrity_counters,
    );
    let entry = reader
        .floor(
            admission.allocation,
            worth_store_physical_format::FreeSpaceKey::arena(range.arena(), range.offset()),
            &mut ManifestDiscoveryCounterSnapshot::default(),
        )
        .map_err(membership_failure)?;
    Ok(entry.is_some_and(|entry| {
        entry.generation() > source_root
            && entry.generation() <= root.generation()
            && entry.arena_free_range().is_some_and(|free| {
                free.arena() == range.arena()
                    && free.offset() <= range.offset()
                    && free.end() >= range.end()
            })
    }))
}

/// Reconstruct every old extent route removed by a durable root transition.
///
/// A record drop need not publish an extent generation above one, and neither
/// same-count nor changed-count roots prove which ranges became garbage. Exact
/// shared references in the same tree avoid rereading unchanged subtrees; all
/// other old routes are compared with the authenticated successor root.
pub(super) fn retained_displaced_extents<'reader>(
    prior: &'reader [DurablePhysicalRootManifest],
    current: &'reader DurablePhysicalRootManifest,
    reader: impl Fn(&'reader DurablePhysicalRootManifest) -> ManifestReader<'reader>,
    released: impl Fn(
        &'reader DurablePhysicalRootManifest,
        ExtentArenaRange,
        u64,
    ) -> Result<bool, BootstrapTransitionFailure>,
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
) -> Result<Vec<DisplacedArtifact>, BootstrapTransitionFailure> {
    let mut displaced = Vec::new();
    let mut by_artifact = HashMap::new();
    let mut before = prior.first();
    for root in prior.iter().skip(1).chain(std::iter::once(current)) {
        if let Some(older) = before {
            let older_reader = reader(older);
            let newer_reader = reader(root);
            let transition = RootTransition {
                older,
                newer: root,
                older_reader: &older_reader,
                newer_reader: &newer_reader,
                allocation,
            };
            transition.reconstruct(prior, current, &released, &mut displaced, &mut by_artifact)?;
        }
        before = Some(root);
    }
    Ok(displaced)
}

struct RootTransition<'a, 'reader> {
    older: &'reader DurablePhysicalRootManifest,
    newer: &'reader DurablePhysicalRootManifest,
    older_reader: &'a ManifestReader<'reader>,
    newer_reader: &'a ManifestReader<'reader>,
    allocation: &'a worth_store_buffer_pool::OperationAllocationGrant,
}

impl<'reader> RootTransition<'_, 'reader> {
    fn reconstruct(
        &self,
        prior: &'reader [DurablePhysicalRootManifest],
        current: &'reader DurablePhysicalRootManifest,
        released: &impl Fn(
            &'reader DurablePhysicalRootManifest,
            ExtentArenaRange,
            u64,
        ) -> Result<bool, BootstrapTransitionFailure>,
        displaced: &mut Vec<DisplacedArtifact>,
        by_artifact: &mut HashMap<RetiredArtifact, usize>,
    ) -> Result<(), BootstrapTransitionFailure> {
        let Some(reference) = self.older.routing_root() else {
            return Ok(());
        };
        self.admit_scratch(1, displaced.capacity(), by_artifact.capacity())?;
        let mut pending = vec![reference];
        while let Some(reference) = pending.pop() {
            if self.shared_in_successor(reference)? {
                continue;
            }
            match self.read_older(reference)? {
                PhysicalRootRoutingBlock::Leaf { entries, .. } => {
                    for entry in entries {
                        if let CurrentPhysicalRecordPlacement::Extent(source) = entry {
                            self.charge_if_displaced(
                                source,
                                prior,
                                current,
                                released,
                                displaced,
                                by_artifact,
                                pending.capacity(),
                            )?;
                        }
                    }
                }
                PhysicalRootRoutingBlock::Branch { children, .. } => {
                    let total = pending
                        .len()
                        .checked_add(children.len())
                        .ok_or_else(allocation_denial)?;
                    self.admit_scratch(
                        pending.capacity().max(total),
                        displaced.capacity(),
                        by_artifact.capacity(),
                    )?;
                    pending
                        .try_reserve(children.len())
                        .map_err(|_| allocation_denial())?;
                    pending.extend(children.into_iter().rev());
                }
            }
        }
        Ok(())
    }

    fn shared_in_successor(
        &self,
        older: ManifestBlockReference,
    ) -> Result<bool, BootstrapTransitionFailure> {
        if self.older.tree_identity() != self.newer.tree_identity() {
            return Ok(false);
        }
        let Some(mut candidate) = self.newer.routing_root() else {
            return Ok(false);
        };
        loop {
            if candidate == older {
                return Ok(true);
            }
            if candidate.level() <= older.level()
                || candidate.first() > older.first()
                || candidate.last() < older.last()
            {
                return Ok(false);
            }
            let block = self
                .newer_reader
                .read_block(
                    self.allocation,
                    candidate,
                    &mut ManifestDiscoveryCounterSnapshot::default(),
                )
                .map_err(membership_failure)?;
            let PhysicalRootRoutingBlock::Branch { children, .. } = block else {
                return Ok(false);
            };
            let Some(child) = children
                .into_iter()
                .find(|child| child.contains(older.first()))
            else {
                return Ok(false);
            };
            candidate = child;
        }
    }

    fn read_older(
        &self,
        reference: ManifestBlockReference,
    ) -> Result<PhysicalRootRoutingBlock, BootstrapTransitionFailure> {
        self.older_reader
            .read_block(
                self.allocation,
                reference,
                &mut ManifestDiscoveryCounterSnapshot::default(),
            )
            .map_err(membership_failure)
    }

    fn charge_if_displaced(
        &self,
        source: DurableExtentRecordPlacement,
        prior: &'reader [DurablePhysicalRootManifest],
        current: &'reader DurablePhysicalRootManifest,
        released: &impl Fn(
            &'reader DurablePhysicalRootManifest,
            ExtentArenaRange,
            u64,
        ) -> Result<bool, BootstrapTransitionFailure>,
        displaced: &mut Vec<DisplacedArtifact>,
        by_artifact: &mut HashMap<RetiredArtifact, usize>,
        pending_capacity: usize,
    ) -> Result<(), BootstrapTransitionFailure> {
        let successor = self
            .newer_reader
            .locate(
                self.allocation,
                source.record(),
                &mut ManifestDiscoveryCounterSnapshot::default(),
            )
            .map_err(membership_failure)?;
        if let Some(CurrentPhysicalRecordPlacement::Extent(next)) = successor {
            if next.extent_cell() == source.extent_cell() {
                return if next == source {
                    Ok(())
                } else {
                    Err(damaged_root())
                };
            }
        }
        let artifact = RetiredArtifact::Extent {
            extent: source.extent().get(),
            generation: source.extent_generation(),
            range: source.arena_range(),
        };
        let exact = DisplacedArtifact {
            source_root: self.older.generation(),
            artifact,
            bytes: source.arena_range().length(),
        };
        if let Some(index) = by_artifact.get(&artifact) {
            return if displaced[*index] == exact {
                Ok(())
            } else {
                Err(damaged_root())
            };
        }
        for published in prior
            .iter()
            .chain(std::iter::once(current))
            .filter(|published| published.generation() > self.older.generation())
        {
            if released(published, source.arena_range(), self.older.generation())? {
                return Ok(());
            }
        }
        self.admit_scratch(
            pending_capacity,
            displaced.capacity().max(
                displaced
                    .len()
                    .checked_add(1)
                    .ok_or_else(allocation_denial)?,
            ),
            by_artifact.capacity().max(
                by_artifact
                    .len()
                    .checked_add(1)
                    .ok_or_else(allocation_denial)?,
            ),
        )?;
        displaced.try_reserve(1).map_err(|_| allocation_denial())?;
        by_artifact
            .try_reserve(1)
            .map_err(|_| allocation_denial())?;
        by_artifact.insert(artifact, displaced.len());
        displaced.push(exact);
        Ok(())
    }

    fn admit_scratch(
        &self,
        pending: usize,
        displaced: usize,
        indexed: usize,
    ) -> Result<(), BootstrapTransitionFailure> {
        let page_bytes = self.older_reader.format_declaration().page_size().bytes() as usize;
        let node_capacity = usize::from(self.older.node_capacity().max(self.newer.node_capacity()));
        admit_scratch_limit(
            self.allocation.bytes(),
            pending,
            displaced,
            indexed,
            page_bytes,
            node_capacity,
        )
    }
}

fn admit_scratch_limit(
    grant_bytes: u64,
    pending: usize,
    displaced: usize,
    indexed: usize,
    page_bytes: usize,
    node_capacity: usize,
) -> Result<(), BootstrapTransitionFailure> {
    // Vec/HashMap growth can coexist with old storage. While one old leaf is
    // live, successor lookup or historical free-space lookup decodes another
    // block, and both readers may retain bounded frame-load buffers. Reserve
    // four page transfers plus four max-width decoded nodes and all readers
    // before allowing collection growth under the same operation grant.
    let decoded_entry = size_of::<CurrentPhysicalRecordPlacement>()
        .max(size_of::<ManifestBlockReference>())
        .max(size_of::<RecordFreeSpaceManifestEntry>())
        .max(size_of::<FreeSpaceBlockReference>());
    let reader_bytes = size_of::<ManifestReader<'static>>()
        .checked_mul(2)
        .and_then(|bytes| {
            bytes.checked_add(size_of::<
                super::super::planning::free_space_routing::FreeSpaceReader<'static>,
            >())
        })
        .ok_or_else(allocation_denial)?;
    let headroom = page_bytes
        .checked_mul(4)
        .and_then(|bytes| {
            node_capacity
                .checked_mul(decoded_entry)?
                .checked_mul(4)?
                .checked_add(bytes)
        })
        .and_then(|bytes| bytes.checked_add(reader_bytes))
        .ok_or_else(allocation_denial)?;
    let pending_bytes = pending
        .checked_mul(size_of::<ManifestBlockReference>())
        .and_then(|bytes| bytes.checked_mul(4));
    let displaced_bytes = displaced
        .checked_mul(size_of::<DisplacedArtifact>())
        .and_then(|bytes| bytes.checked_mul(4));
    let index_bytes = indexed
        .checked_mul(size_of::<(RetiredArtifact, usize)>() + 16)
        .and_then(|bytes| bytes.checked_mul(4));
    let admitted = pending_bytes
        .and_then(|bytes| displaced_bytes.and_then(|other| bytes.checked_add(other)))
        .and_then(|bytes| index_bytes.and_then(|other| bytes.checked_add(other)))
        .and_then(|bytes| bytes.checked_add(headroom))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .is_some_and(|bytes| bytes <= grant_bytes);
    admitted.then_some(()).ok_or_else(allocation_denial)
}

fn allocation_denial() -> BootstrapTransitionFailure {
    BootstrapTransitionFailure::Denied(RecordBootstrapDenial::from_residency(
        worth_store_buffer_pool::PhysicalResidencyDenial::AllocationFailed,
    ))
}

fn damaged_root() -> BootstrapTransitionFailure {
    BootstrapTransitionFailure::Denied(RecordBootstrapDenial::CurrentRootDamaged)
}

#[cfg(test)]
#[path = "displaced_extents/tests.rs"]
mod tests;
