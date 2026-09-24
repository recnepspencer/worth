use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, ManifestBlockReference,
    PhysicalRootRoutingBlock,
};

use super::super::access::manifest_routing::{ManifestDiscoveryCounterSnapshot, ManifestReader};
use super::super::residency::serving_artifacts::ServingRecordArtifacts;
use super::bootstrap::{backend_before_effect, BootstrapTransitionFailure};
use super::displaced_segments::membership_failure;
use crate::physical_runtime::durability::{DisplacedArtifact, RetiredArtifact};

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
    artifacts: &ServingRecordArtifacts,
    reader: impl Fn(&'reader DurablePhysicalRootManifest) -> ManifestReader<'reader>,
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
                for (extent, generation) in successors {
                    let artifact = RetiredArtifact::Extent {
                        extent,
                        generation: generation - 1,
                    };
                    if displaced.iter().any(|charge| charge.artifact == artifact) {
                        continue;
                    }
                    if let Some(bytes) = artifacts
                        .retained_bytes(artifact)
                        .map_err(backend_before_effect)?
                    {
                        displaced.push(DisplacedArtifact {
                            source_root: older.generation(),
                            artifact,
                            bytes,
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
    successors: &mut Vec<(u64, u64)>,
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
                    Some((placement.extent().get(), placement.extent_generation()))
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
