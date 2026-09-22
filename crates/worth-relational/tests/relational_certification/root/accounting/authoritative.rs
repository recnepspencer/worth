use std::collections::{BTreeMap, BTreeSet};

use super::world::supply_chain::SupplyChainScale;
use super::world::supply_chain::{assert_oracle_matches, certified_supply_chain_world};
use worth_relational::facade::branch::RelationalBranchReferenceState;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::inspection::{
    RelationalAuthoritativeAllocationKind, RelationalExcludedAllocationLane,
};

#[test]
fn phase5_full_authoritative_accounting_deduplicates_one_shared_envelope_and_root() {
    let (world, expected) = certified_supply_chain_world(SupplyChainScale::court());
    assert_oracle_matches(&world, &expected);
    let mut identities = vec![world.runtime.main_branch_identity()];
    for branch_name in ["storm-accounting", "maintenance-accounting"] {
        let (_, source_basis) = world
            .runtime
            .observe_fork_source(&BranchId("main".to_owned()))
            .expect("main is an admitted fork source");
        let branch_id = BranchId(branch_name.to_owned());
        world
            .runtime
            .fork_branch(branch_id.clone(), source_basis)
            .expect("fork retains the complete immutable owner allocation inventory");
        identities.push(
            world
                .runtime
                .branch_identity(&branch_id)
                .expect("fork identity is owner issued"),
        );
    }

    let observation = world
        .runtime
        .observe_branch_sharing(&identities)
        .expect("sharing inspection accepts exact owner identities");
    let ledger = world
        .runtime
        .inspect_owner_allocation_ledger(&identities)
        .expect("owner allocation ledger is independent of the sharing summary");
    let allocations = independent_unique_allocations(ledger.authoritative_allocations());
    let allocations = allocations.as_slice();
    let unique_locators = allocations
        .iter()
        .map(|allocation| allocation.locator())
        .collect::<BTreeSet<_>>();
    assert_eq!(unique_locators.len(), allocations.len());

    let partition_bytes = independently_sum(
        allocations,
        RelationalAuthoritativeAllocationKind::PartitionPayload,
    );
    let partition_state_bytes = independently_sum(
        allocations,
        RelationalAuthoritativeAllocationKind::PartitionStateObject,
    );
    let root_region_bytes = independently_sum(
        allocations,
        RelationalAuthoritativeAllocationKind::RootRegionObject,
    );
    let root_metadata_bytes = independently_sum(
        allocations,
        RelationalAuthoritativeAllocationKind::RootMetadata,
    );
    let root_schema_authority_bytes = independently_sum(
        allocations,
        RelationalAuthoritativeAllocationKind::RootSchemaAuthority,
    );
    let node_object_bytes = independently_sum(
        allocations,
        RelationalAuthoritativeAllocationKind::RootReachabilityStructure,
    );
    let set_object_bytes = independently_sum(
        allocations,
        RelationalAuthoritativeAllocationKind::RootReachabilitySetObject,
    );
    let replacement_storage_bytes = independently_sum(
        allocations,
        RelationalAuthoritativeAllocationKind::RootReplacementStorage,
    );
    let removal_storage_bytes = independently_sum(
        allocations,
        RelationalAuthoritativeAllocationKind::RootRemovalStorage,
    );
    let reachability_bytes = set_object_bytes
        .saturating_add(node_object_bytes)
        .saturating_add(replacement_storage_bytes)
        .saturating_add(removal_storage_bytes);
    let artifact_object_bytes = independently_sum(
        allocations,
        RelationalAuthoritativeAllocationKind::CanonicalCommitArtifact,
    );
    let canonical_payload_bytes = independently_sum(
        allocations,
        RelationalAuthoritativeAllocationKind::CanonicalCommitPayload,
    );
    let envelope_object_bytes = independently_sum(
        allocations,
        RelationalAuthoritativeAllocationKind::CanonicalCommitEnvelope,
    );
    let envelope_nested_bytes = independently_sum(
        allocations,
        RelationalAuthoritativeAllocationKind::CanonicalCommitEnvelopeNested,
    );
    let commit_bytes = artifact_object_bytes
        + canonical_payload_bytes
        + envelope_object_bytes
        + envelope_nested_bytes;
    let independently_summed_total = partition_bytes
        + partition_state_bytes
        + root_region_bytes
        + root_metadata_bytes
        + root_schema_authority_bytes
        + reachability_bytes
        + commit_bytes;

    assert!(
        partition_bytes > 0,
        "the causal world owns nested aspect truth"
    );
    assert!(
        partition_state_bytes > 0,
        "the Arc owns a PartitionState object"
    );
    assert!(root_region_bytes > 0, "the map owns a root-region wrapper");
    assert!(root_metadata_bytes > 0);
    assert!(
        root_schema_authority_bytes > 0,
        "the exact root owns its schema authority allocation"
    );
    assert!(set_object_bytes > 0);
    assert!(node_object_bytes > 0);
    assert!(replacement_storage_bytes > 0);
    for (bytes, owner) in [
        (artifact_object_bytes, "artifact object"),
        (canonical_payload_bytes, "canonical payload"),
        (envelope_object_bytes, "envelope object"),
        (envelope_nested_bytes, "nested envelope owner storage"),
    ] {
        assert!(bytes > 0, "the canonical {owner} is inventoried");
    }
    assert_eq!(observation.unique_canonical_commit_artifacts(), 1);
    assert_eq!(observation.unique_root_count(), 1);
    assert_eq!(
        observation.unique_physical_partition_payload_bytes(),
        partition_bytes
    );
    assert_eq!(
        observation.unique_physical_root_metadata_bytes(),
        root_metadata_bytes
    );
    assert_eq!(
        observation.unique_physical_root_reachability_bytes(),
        reachability_bytes
    );
    assert_eq!(
        observation.unique_physical_canonical_commit_bytes(),
        commit_bytes
    );
    assert_eq!(
        observation.unique_physical_authoritative_bytes(),
        independently_summed_total,
        "the full metric must include every independently inventoried owner allocation"
    );
    assert_eq!(observation.branch_count(), identities.len() as u64);
    assert_eq!(
        observation.logical_branch_authoritative_bytes(),
        independently_summed_total * observation.branch_count(),
        "three branches charge one shared envelope/root inventory three times logically and once physically"
    );
    assert_eq!(
        observation.branch_metadata_bytes(),
        observation.branch_count() * std::mem::size_of::<RelationalBranchReferenceState>() as u64,
        "branch metadata is only the shallow inline live reference-state lane"
    );
    let excluded = independent_excluded_totals(ledger.excluded_allocations());
    assert_eq!(
        observation.unique_diagnostic_bytes(),
        excluded[&RelationalExcludedAllocationLane::Diagnostics]
    );
    assert_eq!(
        observation.unique_retention_metadata_bytes(),
        excluded[&RelationalExcludedAllocationLane::RetentionMetadata]
    );
    assert_eq!(
        observation.unique_allocator_bookkeeping_bytes(),
        excluded[&RelationalExcludedAllocationLane::AllocatorBookkeeping]
    );
    assert_eq!(
        observation.unique_optional_cache_bytes(),
        excluded[&RelationalExcludedAllocationLane::OptionalCache]
    );
    assert!(observation.unique_diagnostic_bytes() > 0);
    assert_eq!(
        observation.unique_retention_metadata_bytes(),
        0,
        "immutable roots clear runtime pins without allocating zero-count rows"
    );

    let sabotaged_total_without_reachability = mutant_total_excluding(
        allocations,
        &BTreeSet::from([
            RelationalAuthoritativeAllocationKind::RootReachabilityStructure,
            RelationalAuthoritativeAllocationKind::RootReachabilitySetObject,
            RelationalAuthoritativeAllocationKind::RootReplacementStorage,
            RelationalAuthoritativeAllocationKind::RootRemovalStorage,
        ]),
    );
    assert_ne!(
        observation.unique_physical_authoritative_bytes(),
        sabotaged_total_without_reachability,
        "omitting the persistent reachability structure must fail the accounting oracle"
    );
    for omitted_kind in [
        RelationalAuthoritativeAllocationKind::RootSchemaAuthority,
        RelationalAuthoritativeAllocationKind::PartitionStateObject,
        RelationalAuthoritativeAllocationKind::RootRegionObject,
    ] {
        assert_ne!(
            observation.unique_physical_authoritative_bytes(),
            mutant_total_excluding(allocations, &BTreeSet::from([omitted_kind])),
            "omitting either separately allocated region object must fail the accounting oracle"
        );
    }
    for omitted_kind in [
        RelationalAuthoritativeAllocationKind::CanonicalCommitArtifact,
        RelationalAuthoritativeAllocationKind::CanonicalCommitPayload,
        RelationalAuthoritativeAllocationKind::CanonicalCommitEnvelope,
        RelationalAuthoritativeAllocationKind::CanonicalCommitEnvelopeNested,
    ] {
        assert_ne!(
            observation.unique_physical_authoritative_bytes(),
            mutant_total_excluding(allocations, &BTreeSet::from([omitted_kind])),
            "omitting any canonical artifact/envelope owner allocation turns the oracle red"
        );
    }
}

fn independent_unique_allocations(
    allocations: &[worth_relational::facade::inspection::RelationalAuthoritativeAllocationObservation],
) -> Vec<worth_relational::facade::inspection::RelationalAuthoritativeAllocationObservation> {
    allocations
        .iter()
        .copied()
        .map(|allocation| (allocation.locator(), allocation))
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect()
}

fn independent_excluded_totals(
    allocations: &[worth_relational::facade::inspection::RelationalOwnerExcludedAllocationObservation],
) -> BTreeMap<RelationalExcludedAllocationLane, u64> {
    let unique = allocations
        .iter()
        .copied()
        .map(|allocation| {
            (
                (
                    allocation.lane(),
                    allocation.owner_id(),
                    allocation.partition_id(),
                ),
                allocation.bytes(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut totals = BTreeMap::from([
        (RelationalExcludedAllocationLane::Diagnostics, 0_u64),
        (RelationalExcludedAllocationLane::RetentionMetadata, 0_u64),
        (
            RelationalExcludedAllocationLane::AllocatorBookkeeping,
            0_u64,
        ),
        (RelationalExcludedAllocationLane::OptionalCache, 0_u64),
    ]);
    for ((lane, _, _), bytes) in unique {
        totals.entry(lane).and_modify(|total| *total += bytes);
    }
    totals
}

fn independently_sum(
    allocations: &[worth_relational::facade::inspection::RelationalAuthoritativeAllocationObservation],
    kind: RelationalAuthoritativeAllocationKind,
) -> u64 {
    allocations
        .iter()
        .filter(|allocation| allocation.locator().kind() == kind)
        .map(|allocation| allocation.authoritative_bytes())
        .sum()
}

fn mutant_total_excluding(
    allocations: &[worth_relational::facade::inspection::RelationalAuthoritativeAllocationObservation],
    omitted: &BTreeSet<RelationalAuthoritativeAllocationKind>,
) -> u64 {
    allocations
        .iter()
        .filter(|allocation| !omitted.contains(&allocation.locator().kind()))
        .map(|allocation| allocation.authoritative_bytes())
        .sum()
}
