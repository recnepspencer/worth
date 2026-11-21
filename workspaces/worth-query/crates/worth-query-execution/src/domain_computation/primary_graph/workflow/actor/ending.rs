//! One native field for the complete, versioned ending-actor sum.
use super::record::actor_record_corruption;
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectFieldLocator, AspectValue, WorkflowEndingActorProvenance};
use worth_relational::facade::identity::{EntityId, KindId};

pub(in crate::domain_computation::primary_graph) use worth_query_execution_layout::facade::WorkflowEndingActorFields;

pub(in crate::domain_computation::primary_graph) fn append_ending_actor(
    fields_layout: &WorkflowEndingActorFields,
    actor: &worth_proof::AdmittedWorkflowEndingActor<
        super::record::WorkflowAdmittedActor,
        worth_relational::facade::runtime::RelationalHomeProgramPlanningSource,
    >,
    fields: &mut BTreeMap<AspectFieldLocator, AspectValue>,
) {
    let provenance = match actor {
        worth_proof::AdmittedWorkflowEndingActor::Principal(actor) => actor.ending_provenance(),
        worth_proof::AdmittedWorkflowEndingActor::HomeProgramTransition(request) => {
            WorkflowEndingActorProvenance::HomeWorldOwner(request.source().actor())
        }
    };
    fields.insert(
        fields_layout.payload.clone(),
        AspectValue::InlineBytes(provenance.recorded_bytes()),
    );
}

pub(in crate::domain_computation::primary_graph) fn read_adoption_ending_actor(
    fields_layout: &WorkflowEndingActorFields,
    record: &super::super::adoption::WorkflowAdoptionEntity<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity: EntityId,
) -> Result<Option<WorkflowEndingActorProvenance>, super::super::adoption::WorkflowAdoptionReadDenial>
{
    decode(record.value(execution, &fields_layout.payload)?).map_err(|_| {
        super::super::adoption::WorkflowAdoptionReadDenial::UnreadableEntity { entity }
    })
}

pub(in crate::domain_computation::primary_graph) fn observe_ending_actor(
    fields_layout: &WorkflowEndingActorFields,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    facts: &mut Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>,
) -> Result<
    Option<WorkflowEndingActorProvenance>,
    crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
> {
    use crate::domain_computation::primary_graph::application_attempt::observe_field_value;
    use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;
    let value = observe_field_value(
        runtime,
        execution,
        snapshot,
        entity,
        kind,
        &fields_layout.payload,
    )?;
    let actor = decode(value.clone()).map_err(|_| actor_record_corruption())?;
    facts.push(match value {
        Some(value) => Fact::Field {
            entity_id: entity,
            kind,
            locator: fields_layout.payload.clone(),
            value,
        },
        None => Fact::AbsentField {
            entity_id: entity,
            kind,
            locator: fields_layout.payload.clone(),
        },
    });
    Ok(actor)
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
