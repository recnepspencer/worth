#[allow(unused_imports)]
use crate::relational_certification::reference_context::root_authoritative_accounting::*;
pub(crate) use {std::collections::BTreeMap, std::collections::BTreeSet};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world};
pub(crate) use {worth_relational::facade::branch::RelationalBranchReferenceState};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind, worth_relational::facade::inspection::RelationalExcludedAllocationLane};

#[test]
fn full_authoritative_accounting_deduplicates_one_shared_envelope_and_root() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, expected) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &expected);
    let mut identities = vec![world.runtime.main_branch_identity()];
    for branch_name in ["storm-accounting", "maintenance-accounting"] {
        let (_, source_basis) = world
            .runtime
            .observe_fork_source(&BranchId("main".to_owned()))
            .expect("main is an admitted fork source");
        let branch_id = BranchId(branch_name.to_owned());
        world
            .runtime
            .fork_branch(execution, branch_id.clone(), source_basis)
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
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::PartitionPayload,
    );
    let partition_state_bytes = independently_sum(
        allocations,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::PartitionStateObject,
    );
    let root_region_bytes = independently_sum(
        allocations,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::RootRegionObject,
    );
    let root_metadata_bytes = independently_sum(
        allocations,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::RootMetadata,
    );
    let root_schema_authority_bytes = independently_sum(
        allocations,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::RootSchemaAuthority,
    );
    let node_object_bytes = independently_sum(
        allocations,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::RootReachabilityStructure,
    );
    let set_object_bytes = independently_sum(
        allocations,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::RootReachabilitySetObject,
    );
    let replacement_storage_bytes = independently_sum(
        allocations,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::RootReplacementStorage,
    );
    let removal_storage_bytes = independently_sum(
        allocations,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::RootRemovalStorage,
    );
    let reachability_bytes = set_object_bytes
        .saturating_add(node_object_bytes)
        .saturating_add(replacement_storage_bytes)
        .saturating_add(removal_storage_bytes);
    let artifact_object_bytes = independently_sum(
        allocations,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::CanonicalCommitArtifact,
    );
    let canonical_payload_bytes = independently_sum(
        allocations,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::CanonicalCommitPayload,
    );
    let envelope_object_bytes = independently_sum(
        allocations,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::CanonicalCommitEnvelope,
    );
    let envelope_nested_bytes = independently_sum(
        allocations,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::CanonicalCommitEnvelopeNested,
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
        excluded
            [&worth_relational::facade::inspection::RelationalExcludedAllocationLane::Diagnostics]
    );
    assert_eq!(
        observation.unique_retention_metadata_bytes(),
        excluded[&worth_relational::facade::inspection::RelationalExcludedAllocationLane::RetentionMetadata]
    );
    assert_eq!(
        observation.unique_allocator_bookkeeping_bytes(),
        excluded[&worth_relational::facade::inspection::RelationalExcludedAllocationLane::AllocatorBookkeeping]
    );
    assert_eq!(
        observation.unique_optional_cache_bytes(),
        excluded[&worth_relational::facade::inspection::RelationalExcludedAllocationLane::OptionalCache]
    );
    assert!(observation.unique_diagnostic_bytes() > 0);
    assert_eq!(
        observation.unique_retention_metadata_bytes(),
        0,
        "immutable roots clear runtime pins without allocating zero-count rows"
    );

    let sabotaged_total_without_reachability = mutant_total_excluding(
        allocations,
        &std::collections::BTreeSet::from([
            worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::RootReachabilityStructure,
            worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::RootReachabilitySetObject,
            worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::RootReplacementStorage,
            worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::RootRemovalStorage,
        ]),
    );
    assert_ne!(
        observation.unique_physical_authoritative_bytes(),
        sabotaged_total_without_reachability,
        "omitting the persistent reachability structure must fail the accounting oracle"
    );
    for omitted_kind in [
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::RootSchemaAuthority,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::PartitionStateObject,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::RootRegionObject,
    ] {
        assert_ne!(
            observation.unique_physical_authoritative_bytes(),
            mutant_total_excluding(allocations, &std::collections::BTreeSet::from([omitted_kind])),
            "omitting either separately allocated region object must fail the accounting oracle"
        );
    }
    for omitted_kind in [
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::CanonicalCommitArtifact,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::CanonicalCommitPayload,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::CanonicalCommitEnvelope,
        worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind::CanonicalCommitEnvelopeNested,
    ] {
        assert_ne!(
            observation.unique_physical_authoritative_bytes(),
            mutant_total_excluding(allocations, &std::collections::BTreeSet::from([omitted_kind])),
            "omitting any canonical artifact/envelope owner allocation turns the oracle red"
        );
    }
}
