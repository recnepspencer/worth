//! One complete coverage check over the actual source-issued member universe.
use super::*;

pub(super) fn read<Projection>(
    port: &AggregateReadPort<'_, '_>,
    source: AggregateInvocation<'_>,
    definition: AggregateDefinitionIdentity,
    universe: VisibleMembers,
) -> Result<AggregateReadCollection<Projection>, AggregateReadDenial> {
    let budget = port.view.read_budget();
    let mut covered = BTreeSet::new();
    let mut result = AggregateReadCollection {
        groups: BTreeMap::new(),
        _retention: port.view.work_meter(),
    };
    for member in &universe.0 {
        lookup(member, covered.len(), &budget)?;
        if covered.contains(member) {
            continue;
        }
        let group = AggregateRead::from_backing(
            port.read_selected(AggregateSelection::Definition(definition), member.clone())?,
        );
        for actual in group.member_set() {
            lookup(actual, universe.0.len(), &budget)?;
            lookup(actual, covered.len(), &budget)?;
            if universe.0.binary_search(actual).is_err() || covered.contains(actual) {
                return Err(inconsistent(
                    source,
                    "census cohort differs from exact member universe",
                ));
            }
            budget
                .claim_retained(extent::tree_entry::<NativeMemberKey, ()>())
                .map_err(AggregateReadDenial::Budget)?;
            lookup(actual, covered.len(), &budget)?;
            covered.insert(actual.clone());
        }
        budget
            .checkpoint(search_work(result.groups.len()).saturating_mul(2))
            .map_err(AggregateReadDenial::Budget)?;
        if result.groups.contains_key(&group.identity()) {
            return Err(inconsistent(
                source,
                "distinct census cohorts share an identity",
            ));
        }
        budget
            .claim_retained(extent::tree_entry::<
                worth_foundational::facade::AggregateIdentity,
                AggregateRead<Projection>,
            >())
            .map_err(AggregateReadDenial::Budget)?;
        result.groups.insert(group.identity(), group);
    }
    if covered.len() != universe.0.len() {
        return Err(inconsistent(source, "census coverage is incomplete"));
    }
    budget.check_live().map_err(AggregateReadDenial::Budget)?;
    Ok(result)
}
fn lookup(
    member: &NativeMemberKey,
    count: usize,
    budget: &crate::validation::data::CustomInvariantReadBudget<'_>,
) -> Result<(), AggregateReadDenial> {
    let units = member
        .comparison_work(count)
        .and_then(|units| u64::try_from(units).ok())
        .and_then(|units| units.checked_add(search_work(count)))
        .ok_or(AggregateReadDenial::PreparationRefused)?;
    budget
        .checkpoint(units)
        .map_err(AggregateReadDenial::Budget)
}
