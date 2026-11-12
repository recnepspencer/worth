//! Inverse UTF-8 bounds borrow framework-maintained real probe extrema.
use super::*;
use worth_query_consumer_values::{compare_prefix_keys, prefix_watch_bounds_may_match};
impl<S: ApplicationSchema, I: PrefixSourceIndex<S>> WorthQueryManagedIndexPredicateProvider<S, I>
    for Provider
{
    type PredicateSummary = ();
    fn compare_probes(
        &self,
        a: &Probe,
        b: &Probe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<std::cmp::Ordering, Stop> {
        compare_prefix_keys(&a.prefix, 0, &b.prefix, 0, |work| {
            control.checkpoint(work).map_err(Stop::Controlled)
        })
    }
    fn predicate_summary(
        &self,
        _: &Probe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        control.poll().map_err(Into::into)
    }
    fn predicate_region_may_match(
        &self,
        _: &(),
        extrema: [&Probe; 2],
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        #[cfg(test)]
        self.0.predicate_regions.fetch_add(1, Ordering::SeqCst);
        I::with_name(&row, |text| {
            prefix_watch_bounds_may_match(&extrema[0].prefix, &extrema[1].prefix, text, |work| {
                control.checkpoint(work).map_err(Stop::Controlled)
            })
        })
        .map_err(text_denial)?
    }
}
