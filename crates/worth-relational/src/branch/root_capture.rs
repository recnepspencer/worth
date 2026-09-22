use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::identity::data::PartitionId;
use crate::storage::overlay::PartitionState;
use crate::storage::RelationalPublishedPartitionDelta;

use super::{
    RelationalBranchRoot, RelationalBranchRootCaptureDenial, RelationalBranchRootIdentityIssuer,
    RelationalBranchRootPublicationCost, RelationalPersistentRegionSet, RelationalRootRegion,
};

pub(super) fn build_initial_regions(
    issuer: &RelationalBranchRootIdentityIssuer,
    root_id: u64,
    partitions: &BTreeMap<PartitionId, PartitionState>,
    symbols: &crate::symbols::data::StringInterner,
) -> Result<
    (
        Arc<RelationalPersistentRegionSet>,
        RelationalBranchRootPublicationCost,
    ),
    RelationalBranchRootCaptureDenial,
> {
    let mut regions = BTreeMap::new();
    let mut new_region_authoritative_bytes = 0_u64;
    let mut content_values_hashed = 0_u64;
    for (partition_id, partition) in partitions {
        let region = Arc::new(RelationalRootRegion::new(
            root_id,
            issuer.issue_region_id()?,
            partition.clone(),
            symbols,
            None,
        )?);
        let observation = region.observation();
        content_values_hashed += region.content_values_hashed;
        new_region_authoritative_bytes = new_region_authoritative_bytes
            .saturating_add(observation.authoritative_bytes)
            .saturating_add(observation.partition_state_bytes)
            .saturating_add(observation.root_region_bytes);
        regions.insert(*partition_id, region);
    }
    let first_new_node_id = issuer.next_reachability_id();
    let regions = RelationalPersistentRegionSet::initial(root_id, regions, issuer)?;
    let (persistent_index_path_nodes, reachability_bytes) =
        regions.newly_owned_allocation_cost(first_new_node_id);
    let cost = RelationalBranchRootPublicationCost {
        touched_regions: regions.len() as u64,
        reused_regions: 0,
        persistent_index_path_nodes,
        content_values_hashed,
        new_authoritative_bytes: new_region_authoritative_bytes
            .saturating_add(reachability_bytes)
            .saturating_add(std::mem::size_of::<RelationalBranchRoot>() as u64),
        copied_truth_bytes: 0,
        copied_commit_envelopes: 0,
        new_schema_authorities: 0,
        reused_schema_authorities: 0,
    };
    Ok((regions, cost))
}

pub(super) fn build_incremental_regions(
    issuer: &RelationalBranchRootIdentityIssuer,
    root_id: u64,
    published_delta: &RelationalPublishedPartitionDelta,
    previous: &RelationalBranchRoot,
    symbols: &crate::symbols::data::StringInterner,
) -> Result<
    (
        Arc<RelationalPersistentRegionSet>,
        RelationalBranchRootPublicationCost,
    ),
    RelationalBranchRootCaptureDenial,
> {
    let mut replacements = BTreeMap::new();
    let mut new_region_authoritative_bytes = 0_u64;
    let mut copied_truth_bytes = 0_u64;
    let mut content_values_hashed = 0_u64;
    let mut replaced_prior_regions = 0_usize;
    for partition_id in published_delta.partition_ids() {
        let (overlay, journal) = published_delta
            .publication(partition_id)
            .ok_or(RelationalBranchRootCaptureDenial::PublishedPartitionMissing(partition_id))?;
        let (partition, copied_partition_bytes) =
            if let Some(previous_partition) = previous.partition_state(partition_id) {
                replaced_prior_regions = replaced_prior_regions.saturating_add(1);
                let mut partition = previous_partition.clone();
                let mut overlay = overlay.clone();
                let copied_truth_bytes = partition.merge_overlay_from_owned(&mut overlay, journal);
                (partition, copied_truth_bytes)
            } else {
                (overlay.clone(), 0)
            };
        let region = Arc::new(RelationalRootRegion::new(
            root_id,
            issuer.issue_region_id()?,
            partition,
            symbols,
            previous
                .regions
                .get(partition_id)
                .map(|region| (region.as_ref(), journal)),
        )?);
        let observation = region.observation();
        content_values_hashed += region.content_values_hashed;
        let new_payload_bytes = previous.partition_state(partition_id).map_or(
            observation.authoritative_bytes,
            |previous| {
                region
                    .partition
                    .new_authoritative_allocation_bytes_since(previous)
            },
        );
        new_region_authoritative_bytes = new_region_authoritative_bytes
            .saturating_add(new_payload_bytes)
            .saturating_add(observation.partition_state_bytes)
            .saturating_add(observation.root_region_bytes);
        copied_truth_bytes = copied_truth_bytes.saturating_add(copied_partition_bytes);
        replacements.insert(partition_id, region);
    }
    let first_new_node_id = issuer.next_reachability_id();
    let regions = RelationalPersistentRegionSet::replace(
        root_id,
        &previous.regions,
        replacements,
        BTreeSet::new(),
        issuer,
    )?;
    let (persistent_index_path_nodes, reachability_bytes) =
        regions.newly_owned_allocation_cost(first_new_node_id);
    let cost = RelationalBranchRootPublicationCost {
        touched_regions: published_delta.len() as u64,
        reused_regions: previous
            .regions
            .len()
            .saturating_sub(replaced_prior_regions) as u64,
        persistent_index_path_nodes,
        content_values_hashed,
        new_authoritative_bytes: new_region_authoritative_bytes
            .saturating_add(reachability_bytes)
            .saturating_add(std::mem::size_of::<RelationalBranchRoot>() as u64),
        copied_truth_bytes,
        copied_commit_envelopes: 0,
        new_schema_authorities: 0,
        reused_schema_authorities: 0,
    };
    Ok((regions, cost))
}
