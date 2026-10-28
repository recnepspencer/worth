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
    build_initial_regions_owned(
        issuer,
        root_id,
        partitions
            .iter()
            .map(|(partition_id, partition)| (*partition_id, partition.clone())),
        symbols,
    )
}

pub(super) fn build_initial_regions_owned(
    issuer: &RelationalBranchRootIdentityIssuer,
    root_id: u64,
    partitions: impl IntoIterator<Item = (PartitionId, PartitionState)>,
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
            partition,
            symbols,
        )?);
        let observation = region.observation();
        content_values_hashed += region.content_values_hashed;
        new_region_authoritative_bytes = new_region_authoritative_bytes
            .saturating_add(observation.authoritative_bytes)
            .saturating_add(observation.partition_state_bytes)
            .saturating_add(observation.root_region_bytes);
        regions.insert(partition_id, region);
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
    symbols: &crate::runtime::RuntimeSymbolTable,
    schema: &Arc<super::RelationalBranchRootSchemaAuthority>,
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
        let prepared = if let Some(prior) = previous.regions.get(partition_id) {
            replaced_prior_regions += 1;
            crate::storage::residency::PreparedResidentRegion::incremental(
                crate::storage::residency::RegionCaptureBasis {
                    retention: &prior.partition,
                    inventory: prior.allocation_inventory(),
                    unavailable_records: prior.materialization_unavailable_records(),
                },
                overlay,
                journal,
                schema.clone(),
                symbols,
            )
        } else {
            crate::storage::residency::PreparedResidentRegion::initial(
                overlay.clone(),
                &symbols.interner_snapshot(),
            )
        }
        .map_err(|failure| super::super::root_region::capture_failure(partition_id, failure))?;
        let new_payload_bytes = prepared.new_payload_bytes;
        copied_truth_bytes = copied_truth_bytes.saturating_add(prepared.copied_truth_bytes);
        let region = Arc::new(RelationalRootRegion::from_prepared(
            root_id,
            issuer.issue_region_id()?,
            prepared,
        ));
        let observation = region.observation();
        content_values_hashed += region.content_values_hashed;
        new_region_authoritative_bytes = new_region_authoritative_bytes
            .saturating_add(new_payload_bytes)
            .saturating_add(observation.partition_state_bytes)
            .saturating_add(observation.root_region_bytes);
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
