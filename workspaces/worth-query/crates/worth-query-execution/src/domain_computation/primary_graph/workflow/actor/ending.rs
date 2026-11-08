//! One native field for the complete, versioned ending-actor sum.
use super::record::actor_record_corruption;
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectFieldLocator, AspectValue, WorkflowEndingActorProvenance};
use worth_relational::facade::identity::{EntityId, KindId};

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorkflowEndingActorFields {
    payload: AspectFieldLocator,
}
impl WorkflowEndingActorFields {
    pub(in crate::domain_computation::primary_graph) fn planned() -> Result<
        Self,
        crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    > {
        Ok(Self {
            payload:
                crate::domain_computation::primary_graph::schema_layout::planned_field_locator(
                    "workflow-instance",
                    "ending-actor",
                )?,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn append(
        &self,
        actor: WorkflowEndingActorProvenance,
        fields: &mut BTreeMap<AspectFieldLocator, AspectValue>,
    ) {
        fields.insert(
            self.payload.clone(),
            AspectValue::InlineBytes(actor.recorded_bytes()),
        );
    }

    pub(in crate::domain_computation::primary_graph) fn read_adoption(
        &self,
        record: &super::super::adoption::WorkflowAdoptionEntity<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
    ) -> Result<
        Option<WorkflowEndingActorProvenance>,
        super::super::adoption::WorkflowAdoptionReadDenial,
    > {
        decode(record.value(execution, &self.payload)?).map_err(|_| {
            super::super::adoption::WorkflowAdoptionReadDenial::UnreadableEntity { entity }
        })
    }

    pub(in crate::domain_computation::primary_graph) fn observe(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        entity: EntityId,
        kind: KindId,
        facts: &mut Vec<
            crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact,
        >,
    ) -> Result<
        Option<WorkflowEndingActorProvenance>,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    > {
        use crate::domain_computation::primary_graph::application_attempt::observe_field_value;
        use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;
        let value = observe_field_value(runtime, execution, snapshot, entity, kind, &self.payload)?;
        let actor = decode(value.clone()).map_err(|_| actor_record_corruption())?;
        facts.push(match value {
            Some(value) => Fact::Field {
                entity_id: entity,
                kind,
                locator: self.payload.clone(),
                value,
            },
            None => Fact::AbsentField {
                entity_id: entity,
                kind,
                locator: self.payload.clone(),
            },
        });
        Ok(actor)
    }
}

fn decode(value: Option<AspectValue>) -> Result<Option<WorkflowEndingActorProvenance>, ()> {
    match value {
        None => Ok(None),
        Some(AspectValue::InlineBytes(bytes)) => {
            WorkflowEndingActorProvenance::from_recorded_bytes(bytes.as_bytes())
                .map(Some)
                .map_err(|_| ())
        }
        Some(_) => Err(()),
    }
}
