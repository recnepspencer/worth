//! Physical field revisions and declared roles, never equal business values.
use super::*;
use crate::domain_computation::primary_graph::{
    admitted_capture::checkpoint, causal_inspection::WorthQueryApplicationQueryObservation,
    schema_layout::InstalledFieldCoordinate, WorthQueryApplicationObservedFact as Fact,
    WorthQueryApplicationOutputPosture, WorthQueryPrimaryGraphLayout,
};
use worth_execution::MapKernelContext;
mod input;
mod output;
mod performed_input;
use performed_input::PerformedInput;

type Denial = WorthQueryProducerConnectionDenial;
type Stop = MapKernelFailure<ConnectionRefusal>;
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
            if revision.version() != source.commit_reference().version_id {
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
                    {
                        continue;
                    }
                    let Some(handler) = self.handler_read(
                        context,
                        inputs,
                        *entity_id,
                        &observed.entity_name,
                        source_field,
                    )?
                    else {
                        continue;
                    };
                    if handler.matches_revision(revision.presence())
                        && handler.matches_effect(
                            source
                                .field_effect(context, *entity_id, source_field)
                                .map_err(MapKernelFailure::Stop)?,
                        )
                    {
                        return Ok(());
                    }
                }
            }
        }
        Err(MapKernelFailure::Domain(
            Denial::SourceInputUnconnected.into(),
        ))
    }
    /// Only a completed handler read of this full entity and installed field counts.
    fn handler_read<'a>(
        &self,
        context: &mut MapKernelContext<'_, '_>,
        inputs: &'a FactPayload,
        entity: worth_relational::facade::identity::EntityId,
        entity_name: &str,
        source: InstalledFieldCoordinate<'_>,
    ) -> Result<Option<PerformedInput<'a>>, Stop> {
        for fact in inputs.iter() {
            checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
            let Some(read) = PerformedInput::from_fact(fact) else {
                continue;
            };
            if read.entity != entity || read.kind != source.entity_kind() {
                continue;
            }
            let resolved = self
                .layout
                .resolve_field_coordinate(self.contracts, entity_name, read.locator, |work| {
                    checkpoint(context, work)
                })
                .map_err(MapKernelFailure::Stop)?;
            if resolved.is_some_and(|field| source.same_field(&field)) {
                return Ok(Some(read));
            }
        }
        Ok(None)
    }
    pub(super) fn projected(&self, entity: &str, aspect: &str, field: &str) -> bool {
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
