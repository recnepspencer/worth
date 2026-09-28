use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, DurablePhysicalRootManifest,
    ExtentArenaRange, ManifestBlockReference, PhysicalRootRoutingBlock,
};

use super::super::access::manifest_routing::{ManifestDiscoveryCounterSnapshot, ManifestReader};
use super::bootstrap::BootstrapTransitionFailure;
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

/// Extent generations a same-count rewrite displaced whose files remain.
///
/// An extent rewrite changes neither the record count nor the inline tail, so
/// only the routing blocks the rewrite's root wrote name the successor. Each
/// root rewrites copy-on-write, so walking just the blocks stamped with that
/// root's generation visits exactly the changed paths. Any extent generation
/// below a published one is unread by every later root; retirement removes
/// its files and the charge ends with them.
///
/// A rewritten leaf also carries its neighbours' earlier successors. Roots are
/// walked oldest first from generation 1 and a charge is kept once, so each
/// displaced generation is charged to the root that first published its
/// successor.
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
    let mut displaced: Vec<DisplacedArtifact> = Vec::new();
    let mut before = prior.first();
    for root in prior.iter().skip(1).chain(std::iter::once(current)) {
        if let Some(older) = before {
            if root.requires_maintenance_protocol() && older.record_count() == root.record_count() {
                let mut successors = Vec::new();
                if let Some(reference) = root.routing_root() {
                    collect_rewritten_extents(
                        &reader(root),
                        allocation,
                        root.generation(),
                        reference,
                        &mut successors,
                    )?;
                }
                for successor in successors {
                    let Some(CurrentPhysicalRecordPlacement::Extent(source)) = reader(older)
                        .locate(
                            allocation,
                            successor.record(),
                            &mut ManifestDiscoveryCounterSnapshot::default(),
                        )
                        .map_err(membership_failure)?
                    else {
                        continue;
                    };
                    if source == successor {
                        continue;
                    }
                    let artifact = RetiredArtifact::Extent {
                        extent: source.extent().get(),
                        generation: source.extent_generation(),
                        range: source.arena_range(),
                    };
                    if displaced.iter().any(|charge| charge.artifact == artifact) {
                        continue;
                    }
                    let mut was_released = false;
                    for published in prior
                        .iter()
                        .chain(std::iter::once(current))
                        .filter(|published| published.generation() > older.generation())
                    {
                        if released(published, source.arena_range(), older.generation())? {
                            was_released = true;
                            break;
                        }
                    }
                    if !was_released {
                        displaced.push(DisplacedArtifact {
                            source_root: older.generation(),
                            artifact,
                            bytes: source.arena_range().length(),
                        });
                    }
                }
            }
        }
        before = Some(root);
    }
    Ok(displaced)
}

fn collect_rewritten_extents(
    reader: &ManifestReader<'_>,
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    generation: u64,
    reference: ManifestBlockReference,
    successors: &mut Vec<DurableExtentRecordPlacement>,
) -> Result<(), BootstrapTransitionFailure> {
    if reference.generation() != generation {
        return Ok(());
    }
    let block = reader
        .read_block(
            allocation,
            reference,
            &mut ManifestDiscoveryCounterSnapshot::default(),
        )
        .map_err(membership_failure)?;
    match block {
        PhysicalRootRoutingBlock::Leaf { entries, .. } => {
            successors.extend(entries.iter().filter_map(|entry| match entry {
                CurrentPhysicalRecordPlacement::Extent(placement)
                    if placement.extent_generation() > 1 =>
                {
                    Some(*placement)
                }
                _ => None,
            }));
        }
        PhysicalRootRoutingBlock::Branch { children, .. } => {
            for child in children {
                collect_rewritten_extents(reader, allocation, generation, child, successors)?;
            }
        }
    }
    Ok(())
}
