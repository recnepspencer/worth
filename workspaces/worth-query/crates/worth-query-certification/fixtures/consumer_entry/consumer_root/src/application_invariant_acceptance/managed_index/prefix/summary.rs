//! Exact UTF-8 enclosure uses only framework-confined current Source rows.
use super::*;
use worth_query_consumer_values::{
    compare_prefix_keys, join_prefix_bounds, prefix_bounds_may_match, PrefixSourceBounds,
};
type Bounds = PrefixSourceBounds<WorthQueryManagedIndexSourceSlot>;
type Sources<'a, 'b, I> = WorthQueryManagedIndexSummarySources<'a, 'b, Catalog, I>;

pub(super) fn merge<I: PrefixSourceIndex>(
    parts: [Option<&Bounds>; 3],
    sources: &Sources<'_, '_, I>,
    control: &WorthQueryManagedIndexControl<'_>,
    calls: &Calls,
) -> Result<Bounds, Stop> {
    #[cfg(not(test))]
    let _ = calls;
    let mut result = None;
    for bounds in parts.into_iter().flatten().copied() {
        control.checkpoint(1)?;
        result = Some(match result {
            None => bounds,
            Some(before) => join_prefix_bounds(before, bounds, |left, right| {
                #[cfg(test)]
                calls.cost_stages.record(cost_stages::Stage::LoanBegin);
                let a = sources.get(left)?;
                #[cfg(test)]
                calls.cost_stages.record(cost_stages::Stage::LoanEnd);
                #[cfg(test)]
                calls.cost_stages.record(cost_stages::Stage::LoanBegin);
                let b = sources.get(right)?;
                #[cfg(test)]
                calls.cost_stages.record(cost_stages::Stage::LoanEnd);
                a.with_text(query::name(), |a| {
                    b.with_text(query::name(), |b| {
                        compare_prefix_keys(a, 0, b, 0, |units| {
                            control.checkpoint(units).map_err(Stop::Controlled)
                        })
                        .map(|order| order.then_with(|| left.cmp(&right)))
                    })
                    .map_err(text_denial)?
                })
                .map_err(text_denial)?
            })?,
        });
    }
    result.ok_or(Stop::Domain(PrefixIndexDenial::InvalidTopology))
}
pub(super) fn may_match<I: PrefixSourceIndex>(
    bounds: &Bounds,
    sources: &Sources<'_, '_, I>,
    probe: &query::Probe,
    control: &WorthQueryManagedIndexControl<'_>,
    calls: &Calls,
) -> Result<bool, Stop> {
    #[cfg(not(test))]
    let _ = calls;
    #[cfg(test)]
    calls.cost_stages.record(cost_stages::Stage::LoanBegin);
    let lower = sources.get(bounds.lower)?;
    #[cfg(test)]
    calls.cost_stages.record(cost_stages::Stage::LoanEnd);
    #[cfg(test)]
    calls.cost_stages.record(cost_stages::Stage::LoanBegin);
    let upper = sources.get(bounds.upper)?;
    #[cfg(test)]
    calls.cost_stages.record(cost_stages::Stage::LoanEnd);
    lower
        .with_text(query::name(), |lower| {
            upper
                .with_text(query::name(), |upper| {
                    prefix_bounds_may_match(lower, upper, &probe.prefix, |units| {
                        control.checkpoint(units).map_err(Stop::Controlled)
                    })
                })
                .map_err(text_denial)?
        })
        .map_err(text_denial)?
}
