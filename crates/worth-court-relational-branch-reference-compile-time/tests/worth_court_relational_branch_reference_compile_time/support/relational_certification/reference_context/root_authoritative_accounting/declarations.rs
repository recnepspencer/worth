pub(crate) use {std::collections::BTreeMap, std::collections::BTreeSet};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world};
pub(crate) use {worth_relational::facade::branch::RelationalBranchReferenceState};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind, worth_relational::facade::inspection::RelationalExcludedAllocationLane};

pub(crate) fn independent_unique_allocations(
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

pub(crate) fn independent_excluded_totals(
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
    let mut totals = std::collections::BTreeMap::from([
        (worth_relational::facade::inspection::RelationalExcludedAllocationLane::Diagnostics, 0_u64),
        (worth_relational::facade::inspection::RelationalExcludedAllocationLane::RetentionMetadata, 0_u64),
        (
            worth_relational::facade::inspection::RelationalExcludedAllocationLane::AllocatorBookkeeping,
            0_u64,
        ),
        (worth_relational::facade::inspection::RelationalExcludedAllocationLane::OptionalCache, 0_u64),
    ]);
    for ((lane, _, _), bytes) in unique {
        totals.entry(lane).and_modify(|total| *total += bytes);
    }
    totals
}

pub(crate) fn independently_sum(
    allocations: &[worth_relational::facade::inspection::RelationalAuthoritativeAllocationObservation],
    kind: RelationalAuthoritativeAllocationKind,
) -> u64 {
    allocations
        .iter()
        .filter(|allocation| allocation.locator().kind() == kind)
        .map(|allocation| allocation.authoritative_bytes())
        .sum()
}

pub(crate) fn mutant_total_excluding(
    allocations: &[worth_relational::facade::inspection::RelationalAuthoritativeAllocationObservation],
    omitted: &BTreeSet<RelationalAuthoritativeAllocationKind>,
) -> u64 {
    allocations
        .iter()
        .filter(|allocation| !omitted.contains(&allocation.locator().kind()))
        .map(|allocation| allocation.authoritative_bytes())
        .sum()
}