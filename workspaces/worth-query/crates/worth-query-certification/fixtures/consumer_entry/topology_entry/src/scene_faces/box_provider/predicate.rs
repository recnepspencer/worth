//! Conservative inverse routing uses the same exact unit/frame enclosure algebra.
use super::*;
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PredicateBounds(Bounds);
impl WorthQueryManagedIndexPredicateSummary for PredicateBounds {
    fn merge(own: Self, children: [Option<Self>; 2]) -> Self {
        Self(
            children
                .into_iter()
                .flatten()
                .fold(own.0, |a, b| a.union(b.0)),
        )
    }
}
impl<S: SceneFaceSchemaBinding> WorthQueryManagedIndexPredicateProvider<S, SceneFaceBoxIndex<S>>
    for Provider
{
    type PredicateSummary = PredicateBounds;
    fn compare_probes(
        &self,
        a: &SceneBoxProbe,
        b: &SceneBoxProbe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<std::cmp::Ordering, Stop> {
        control.checkpoint(1)?;
        Ok(worth_query_consumer_values::compare_box_bounds(
            probe_bounds(a)?,
            probe_bounds(b)?,
        ))
    }
    fn predicate_summary(
        &self,
        probe: &SceneBoxProbe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<PredicateBounds, Stop> {
        control.checkpoint(1)?;
        probe_bounds(probe).map(PredicateBounds)
    }
    fn predicate_region_may_match(
        &self,
        summary: PredicateBounds,
        _endpoints: [&SceneBoxProbe; 2],
        row: WorthQueryManagedIndexSourceRow<'_, S, SceneFaceBoxIndex<S>>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        control.checkpoint(1)?;
        Ok(summary.0.overlaps(row_bounds(row)?))
    }
}
