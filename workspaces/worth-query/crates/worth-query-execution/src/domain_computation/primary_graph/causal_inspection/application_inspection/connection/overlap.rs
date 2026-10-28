//! Physical field revisions and declared roles, never equal business values.
use super::*;
use crate::domain_computation::primary_graph::{
    admitted_capture::checkpoint, causal_inspection::WorthQueryApplicationQueryObservation,
    schema_layout::InstalledFieldCoordinate, WorthQueryApplicationObservedFact as Fact,
    WorthQueryApplicationOutputPosture, WorthQueryPrimaryGraphLayout,
};
use worth_execution::MapKernelContext;
use worth_relational::facade::runtime::RelationalFieldPresence;
mod input;

type Denial = WorthQueryProducerConnectionDenial;
type Stop = MapKernelFailure<Denial>;
pub(super) struct ObservationConnection<'a, 'governance> {
    pub(super) original: &'a WorthQueryApplicationQueryObservation,
    pub(super) diagnostic: &'a DiagnosticComparisonAdmission<'governance>,
    pub(super) layout: &'a WorthQueryPrimaryGraphLayout,
    pub(super) contracts:
        &'a worth_query_installation::facade::WorthQueryInstalledApplicationSchemaContractCatalog,
}
impl ObservationConnection<'_, '_> {
    /// Diagnostic O(inputs × observed fields × (inputs × installed fields + source patches)).
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
            for row in self.original.source.original_row_footprints() {
                for observed in &row.aspects {
                    checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
                    if !row.complete
                        || observed.entity != *entity_id
                        || observed.native_revision != Some(*revision)
                    {
                        continue;
                    }
                    let Some(source_field) = self
                        .layout
                        .resolve_field_coordinate(
                            self.contracts,
                            &observed.entity_name,
                            locator,
                            |work| checkpoint(context, work),
                        )
                        .map_err(MapKernelFailure::Stop)?
                    else {
                        continue;
                    };
                    let Some(observed_field) = self
                        .layout
                        .resolve_declared_field_coordinate(
                            self.contracts,
                            &observed.entity_name,
                            &observed.aspect,
                            &observed.field,
                            |work| checkpoint(context, work),
                        )
                        .map_err(MapKernelFailure::Stop)?
                    else {
                        continue;
                    };
                    if !source_field.same_field(&observed_field)
                        || source_field.contract_revision() != observed.contract_revision
                        || !self.projected(
                            &observed.entity_name,
                            observed.aspect.as_str(),
                            field.as_str(),
                        )
                        || !self.handler_read(
                            context,
                            inputs,
                            *entity_id,
                            &observed.entity_name,
                            source_field,
                        )?
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
    /// Diagnostic O(roles × observed fields × (installed fields + producer patches)).
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
                    let Some(coordinate) = self
                        .layout
                        .resolve_declared_field_coordinate(
                            self.contracts,
                            entity_name,
                            &field.aspect,
                            &field.field,
                            |work| checkpoint(context, work),
                        )
                        .map_err(MapKernelFailure::Stop)?
                    else {
                        continue;
                    };
                    if coordinate.contract_revision() != field.contract_revision {
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
    /// Only a completed handler read of this full entity and installed field counts.
    fn handler_read(
        &self,
        context: &mut MapKernelContext<'_, '_>,
        inputs: &FactPayload,
        entity: worth_relational::facade::identity::EntityId,
        entity_name: &str,
        source: InstalledFieldCoordinate<'_>,
    ) -> Result<bool, Stop> {
        for fact in inputs.iter() {
            checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
            let Fact::Field {
                entity_id,
                kind,
                locator,
                ..
            } = fact
            else {
                continue;
            };
            if *entity_id != entity || *kind != source.entity_kind() {
                continue;
            }
            let resolved = self
                .layout
                .resolve_field_coordinate(self.contracts, entity_name, locator, |work| {
                    checkpoint(context, work)
                })
                .map_err(MapKernelFailure::Stop)?;
            if resolved.is_some_and(|field| source.same_field(&field)) {
                return Ok(true);
            }
        }
        Ok(false)
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
