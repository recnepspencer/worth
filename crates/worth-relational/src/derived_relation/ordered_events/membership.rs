//! Newest-first event fold and inverse propagation after full chain validation.
use super::*;

pub(super) fn forward<Port: SourceRead>(
    source: &mut EventSource<'_, Port>,
    tip: EntityId,
) -> Result<BTreeSet<Member>, Denial> {
    // Fresh forward acquisition has no previously validated prefix to truncate.
    let chain = chain::prefix(source, tip)?;
    let mut retired = BTreeSet::new();
    let mut live = BTreeSet::new();
    for event in chain {
        for row in source.targets(source.recipe.retired, event, false)? {
            insert_member(source.read, &mut retired, row.into())?;
        }
        for row in source.targets(source.recipe.introduced, event, false)? {
            super::super::evaluation::member_work(source.read, &row.into(), retired.len())?;
            if !retired.contains(&Member::Entity(row)) {
                insert_member(source.read, &mut live, row.into())?;
            }
        }
    }
    Ok(live)
}

pub(super) fn inverse<Port: SourceRead>(
    source: &mut EventSource<'_, Port>,
    row: EntityId,
) -> Result<BTreeSet<Member>, Denial> {
    let seeds = source.targets(source.recipe.introduced, row, true)?;
    let validated = chain::validate_candidates(source, seeds)?;
    propagate(source, validated, row)
}
fn propagate<Port: SourceRead>(
    source: &mut EventSource<'_, Port>,
    validated: chain::ValidatedEventCandidates,
    row: EntityId,
) -> Result<BTreeSet<Member>, Denial> {
    let mut seeds = Vec::new();
    source.read.retain(
        validated
            .seeds()
            .len()
            .checked_mul(size_of::<EntityId>())
            .ok_or(Definition::WorkBudgetExceeded)?,
    )?;
    seeds
        .try_reserve_exact(validated.seeds().len())
        .map_err(|_| allocation_failed())?;
    for &event in validated.seeds() {
        if !source.retired_at(event, row)? {
            seeds.push(event);
        }
    }
    let members = chain::successors(source, &seeds, Some(row), false)?;
    let mut result = BTreeSet::new();
    for member in members {
        insert_member(source.read, &mut result, member.into())?;
    }
    Ok(result)
}
