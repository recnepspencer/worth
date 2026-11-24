//! Actual immutable before/after incidence; allocation order supplies no metadata.
use super::*;
pub(super) fn observe(
    id: RelationId,
    before: &InvariantStateView<'_>,
    after: &InvariantStateView<'_>,
    budget: &CustomPreparationBudget,
) -> Result<AuthoredSourceIncidence, Denial> {
    admission::work(budget, 2)?;
    let old = before.relation_metadata(id);
    let new = after.relation_metadata(id);
    let kind = old
        .as_ref()
        .or(new.as_ref())
        .ok_or(Denial::ForeignSource)?
        .kind_id;
    if old.as_ref().is_some_and(|value| value.kind_id != kind)
        || new.as_ref().is_some_and(|value| value.kind_id != kind)
    {
        return Err(Denial::ForeignSource);
    }
    Ok(AuthoredSourceIncidence {
        relation: id,
        kind,
        before: old.map(|value| (value.source, value.target)),
        after: new.map(|value| (value.source, value.target)),
    })
}
