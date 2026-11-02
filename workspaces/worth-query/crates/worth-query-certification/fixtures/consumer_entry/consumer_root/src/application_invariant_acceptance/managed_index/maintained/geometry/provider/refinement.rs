//! Domain fixture exact closed contact; Source endpoint boxes must be points.
use super::*;
impl WorthQueryManagedIndexRefinementProvider<Catalog, BoxIndex> for Provider {
    const REFINEMENT_IDENTITY: &'static str = "maintained-geometry.closed-segment-box";
    const REFINEMENT_VERSION: u32 = 1;
    fn validate_refinement_probe(
        &self,
        probe: &Coordinates,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        control.checkpoint(4)?;
        Box2::new(probe.raw()).map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))?;
        Ok(())
    }
    fn refine_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, Catalog, BoxIndex>,
        probe: &Coordinates,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        control.checkpoint(32)?;
        let a: Coordinates = row
            .one(query::start())
            .and_then(|r| r.field(query::start_region()))
            .map_err(|_| Stop::Domain(BoxIndexDenial::InvalidTopology))?;
        let b: Coordinates = row
            .one(query::end())
            .and_then(|r| r.field(query::end_region()))
            .map_err(|_| Stop::Domain(BoxIndexDenial::InvalidTopology))?;
        let bounds = |coordinates| {
            Box2::new(coordinates).map_err(|cause| Stop::Domain(BoxIndexDenial::Bounds(cause)))
        };
        worth_query_consumer_values::endpoint_segment_intersects_box(
            bounds(a.raw())?,
            bounds(b.raw())?,
            bounds(probe.raw())?,
        )
        .map_err(|cause| Stop::Domain(BoxIndexDenial::SegmentContact(cause)))
    }
}
