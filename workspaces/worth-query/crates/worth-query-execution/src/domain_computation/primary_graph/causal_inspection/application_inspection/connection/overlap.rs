//! Physical field revisions and declared roles, never equal business values.
use super::*;
use crate::domain_computation::primary_graph::{
    admitted_capture::checkpoint, application_checkpoint::facts::FactSequence,
    causal_inspection::WorthQueryApplicationQueryObservation,
    WorthQueryApplicationObservedFact as Fact, WorthQueryApplicationOutputPosture,
};
use worth_execution::MapKernelContext;
use worth_relational::facade::runtime::RelationalFieldPresence;
type Denial = WorthQueryProducerConnectionDenial;
type Stop = MapKernelFailure<Denial>;
pub(super) struct ObservationConnection<'a, 'governance> {
    pub(super) original: &'a WorthQueryApplicationQueryObservation,
    pub(super) diagnostic: &'a DiagnosticComparisonAdmission<'governance>,
}
impl ObservationConnection<'_, '_> {
    /// O(inputs² + inputs × observed fields × source patches), diagnostic only.
    pub(super) fn require_input(
        &self,
        context: &mut MapKernelContext<'_, '_>,
        execution: &ExecutionRequest<'_, '_>,
        source: &WorthQueryApplicationCommittedChanges,
        inputs: &FactPayload,
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
            let [field] = locator.field_path().fields() else {
                continue;
            };
            if revision.version() != source.commit_reference().version_id
                || revision.presence() != RelationalFieldPresence::Present
            {
                continue;
            }
            if !handler_read(context, inputs, *entity_id, locator)? {
                continue;
            }
            for row in self.original.source.original_row_footprints() {
                for observed in &row.aspects {
                    checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
                    if !row.complete
                        || observed.entity != *entity_id
                        || observed.aspect != *locator.aspect().aspect_key()
                        || observed.field != *field
                        || observed.native_revision != Some(*revision)
                        || !self.projected(
                            &observed.entity_name,
                            observed.aspect.as_str(),
                            field.as_str(),
                        )
                    {
                        continue;
                    }
                    if source
                        .committed_field_ref(execution, *entity_id, locator)
                        .map_err(MapKernelFailure::Stop)?
                        .is_some()
                    {
                        return Ok(());
                    }
                }
            }
        }
        Err(MapKernelFailure::Domain(Denial::SourceInputUnconnected))
    }
    /// O(roles × observed fields × producer patches), on the diagnostic lane.
    pub(super) fn output_role(
        &self,
        context: &mut MapKernelContext<'_, '_>,
        execution: &ExecutionRequest<'_, '_>,
        changes: &WorthQueryApplicationCommittedChanges,
        correspondence: &WorthQueryApplicationOutputCorrespondence,
    ) -> Result<usize, Stop> {
        for (role, (_, posture, entity_name, entity)) in
            correspondence.native_witness_roles().enumerate()
        {
            checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
            if posture == WorthQueryApplicationOutputPosture::Retire {
                continue;
            }
            for row in self.original.source.original_row_footprints() {
                for field in &row.aspects {
                    checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
                    if !row.complete
                        || field.entity != entity
                        || field.entity_name != entity_name
                        || !field.native_revision.is_some_and(|revision| {
                            revision.version() == changes.commit_reference().version_id
                                && revision.presence() == RelationalFieldPresence::Present
                        })
                        || !self.projected(entity_name, field.aspect.as_str(), field.field.as_str())
                    {
                        continue;
                    }
                    if changes
                        .contains_present_field(execution, entity, &field.aspect, &field.field)
                        .map_err(MapKernelFailure::Stop)?
                    {
                        return Ok(role);
                    }
                }
            }
        }
        Err(MapKernelFailure::Domain(Denial::OutputUnconnected))
    }
    fn projected(&self, entity: &str, aspect: &str, field: &str) -> bool {
        self.diagnostic
            .graph()
            .projections()
            .iter()
            .any(|projection| {
                projection.entity() == entity
                    && projection.aspect() == aspect
                    && projection.field() == field
            })
    }
}

/// Source-query membership alone cannot claim that the handler read a field.
fn handler_read(
    context: &mut MapKernelContext<'_, '_>,
    inputs: &FactPayload,
    entity: worth_relational::facade::identity::EntityId,
    locator: &worth_foundational::facade::AspectFieldLocator,
) -> Result<bool, Stop> {
    for fact in inputs.iter() {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        if matches!(fact, Fact::Field { entity_id, locator: read, .. } if *entity_id == entity && read == locator)
        {
            return Ok(true);
        }
    }
    Ok(false)
}
