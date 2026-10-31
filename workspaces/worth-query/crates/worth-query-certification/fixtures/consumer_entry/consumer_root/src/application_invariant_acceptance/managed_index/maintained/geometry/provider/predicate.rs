//! Fixed exact box union is the qualified release algebra; no Probe loans in merge.
use super::*;
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PredicateBounds(Box2);
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
impl WorthQueryManagedIndexPredicateProvider<Catalog, BoxIndex> for Provider {
    type PredicateSummary = PredicateBounds;
    fn compare_probes(
        &self,
        a: &Coordinates,
        b: &Coordinates,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<std::cmp::Ordering, Stop> {
        control.checkpoint(1)?;
        let a = Box2::new(a.raw()).map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))?;
        let b = Box2::new(b.raw()).map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))?;
        Ok(worth_query_consumer_values::compare_box_bounds(a, b))
    }
    fn predicate_summary(
        &self,
        probe: &Coordinates,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<PredicateBounds, Stop> {
        control.checkpoint(1)?;
        Box2::new(probe.raw())
            .map(PredicateBounds)
            .map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))
    }
    fn predicate_region_may_match(
        &self,
        summary: PredicateBounds,
        _: [&Coordinates; 2],
        row: WorthQueryManagedIndexSourceRow<'_, Catalog, BoxIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        control.checkpoint(1)?;
        Ok(summary.0.overlaps(row_bounds(row)?))
    }
}
