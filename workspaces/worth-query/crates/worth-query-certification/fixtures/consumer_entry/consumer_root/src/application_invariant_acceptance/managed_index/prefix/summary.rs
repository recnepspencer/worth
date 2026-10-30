//! Exact UTF-8 enclosure uses only framework-confined current Source rows.
use super::*;
use worth_query_consumer_values::{
    compare_prefix_keys, join_prefix_bounds, prefix_bounds_may_match, PrefixSourceBounds,
};
type Bounds = PrefixSourceBounds<WorthQueryManagedIndexSourceSlot>;
type Sources<'a, 'b> = WorthQueryManagedIndexSummarySources<'a, 'b, Catalog, PrefixIndex>;

fn name(sources: &Sources<'_, '_>, slot: WorthQueryManagedIndexSourceSlot) -> Result<String, Stop> {
    sources
        .get(slot)?
        .field(query::name())
        .map_err(|_| Stop::Domain(PrefixIndexDenial::InvalidTopology))
}
pub(super) fn merge(
    parts: [Option<Bounds>; 3],
    sources: &Sources<'_, '_>,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<Bounds, Stop> {
    let mut result = None;
    for bounds in parts.into_iter().flatten() {
        control.checkpoint(1)?;
        result = Some(match result {
            None => bounds,
            Some(before) => join_prefix_bounds(before, bounds, |left, right| {
                let a = name(sources, left)?;
                let b = name(sources, right)?;
                compare_prefix_keys(&a, 0, &b, 0, |units| {
                    control.checkpoint(units).map_err(Stop::Controlled)
                })
                .map(|order| order.then_with(|| left.cmp(&right)))
            })?,
        });
    }
    result.ok_or(Stop::Domain(PrefixIndexDenial::InvalidTopology))
}
pub(super) fn may_match(
    bounds: Bounds,
    sources: &Sources<'_, '_>,
    probe: &query::Probe,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<bool, Stop> {
    prefix_bounds_may_match(
        &name(sources, bounds.lower)?,
        &name(sources, bounds.upper)?,
        &probe.prefix,
        |units| control.checkpoint(units).map_err(Stop::Controlled),
    )
}
