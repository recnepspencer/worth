//! A performed input joins its canonical source without any output-query projection.
use super::*;
impl ObservationConnection<'_, '_> {
    /// Diagnostic O(inputs squared times (installed fields + disclosed fields + source patches)).
    pub(in super::super) fn require_independent_input(
        &self,
        context: &mut MapKernelContext<'_, '_>,
        source: &WorthQueryApplicationCommittedChanges,
        inputs: &FactPayload,
        diagnostic: &DiagnosticComparisonAdmission<'_>,
        subject: worth_relational::facade::identity::EntityId,
    ) -> Result<(), Stop> {
        for fact in inputs.iter() {
            checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
            let Fact::SourceFieldRevision {
                entity_id,
                locator,
                native_revision: Some(revision),
            } = fact
            else {
                continue;
            };
            // Source disclosure admits this original full subject, not every same-kind entity.
            if *entity_id != subject || revision.version() != source.commit_reference().version_id {
                continue;
            }
            let [field] = locator.field_path().fields() else {
                continue;
            };
            for handler in inputs.iter() {
                checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
                let Some(read) = PerformedInput::from_fact(handler) else {
                    continue;
                };
                if read.entity != *entity_id || !read.matches_revision(revision.presence()) {
                    continue;
                }
                let Some(owner) = self.layout.entity_name(read.kind) else {
                    continue;
                };
                let mut disclosed = false;
                for projection in diagnostic.graph().projections() {
                    checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
                    if projection.entity() == owner
                        && projection.aspect() == locator.aspect().aspect_key().as_str()
                        && projection.field() == field.as_str()
                    {
                        disclosed = true;
                        break;
                    }
                }
                if !disclosed {
                    continue;
                }
                let Some(coordinate) = self
                    .layout
                    .resolve_field_coordinate(self.contracts, owner, locator, |work| {
                        checkpoint(context, work)
                    })
                    .map_err(MapKernelFailure::Stop)?
                else {
                    continue;
                };
                let handler_coordinate = self
                    .layout
                    .resolve_field_coordinate(self.contracts, owner, read.locator, |work| {
                        checkpoint(context, work)
                    })
                    .map_err(MapKernelFailure::Stop)?;
                if coordinate.entity_kind() != read.kind
                    || !handler_coordinate.is_some_and(|handler| coordinate.same_field(&handler))
                {
                    continue;
                }
                if read.matches_effect(
                    source
                        .field_effect(context, *entity_id, coordinate)
                        .map_err(MapKernelFailure::Stop)?,
                ) {
                    return Ok(());
                }
            }
        }
        Err(MapKernelFailure::Domain(Denial::SourceInputUnconnected))
    }
}
