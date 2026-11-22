//! Original start DATA, source-relative live custody and canonical instance roles.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::workflow_instance_observation::{observe_instance_roles, subject},
    workflow::instance::{start_provenance, WorkflowInstanceState},
};
use worth_foundational::facade::WorkflowEndingActorProvenance;

pub(super) fn readmit(source: &WorkflowSourceRecords<'_, '_, '_>) -> Result<(), Denial> {
    let fields = &source.fields;
    let layout = source.layout;
    let kind = layout.instance.entity_kind;
    for instance in source.population.entities(kind) {
        let instance = *instance;
        let original = start_provenance::read_recorded(
            fields.execution,
            fields.runtime,
            fields.snapshot(),
            layout,
            instance,
        )
        .map_err(Denial::Start)?;
        source.attribute(instance, original)?;
        let effective = source.revision(instance, kind, &layout.instance.program_revision)?;
        source.attribute(instance, effective)?;
        let lineage = source
            .population
            .one(layout.instance_lineage_relation, instance)?
            .ok_or_else(|| source.corrupt(instance, kind))?;
        let definition = source
            .population
            .one(layout.instance_definition_relation, instance)?
            .ok_or_else(|| source.corrupt(instance, kind))?;
        let data = source
            .definitions
            .get(&definition)
            .ok_or_else(|| source.corrupt(instance, kind))?;
        let content =
            source.revision(instance, kind, &layout.instance.definition_content_identity)?;
        if data.lineage() != lineage || content.bytes() != data.content().as_bytes() {
            return Err(source.corrupt(instance, kind));
        }
        let state = WorkflowInstanceState::from_persisted_tag(
            fields
                .u64(instance, kind, &layout.instance.state)
                .map_err(Denial::Record)?,
        )
        .ok_or_else(|| source.corrupt(instance, kind))?;
        let mut facts = Vec::new();
        subject::read_subject(
            fields.execution,
            fields.runtime,
            fields.snapshot(),
            layout,
            instance,
            &mut facts,
        )
        .map_err(|cause| Denial::Record(WorthQueryWorkflowReadDenial::Execution(cause)))?;
        let roles = observe_instance_roles(
            fields.execution,
            fields.runtime,
            fields.snapshot(),
            layout,
            instance,
            &mut facts,
        )
        .map_err(|cause| Denial::Record(WorthQueryWorkflowReadDenial::Execution(cause)))?;
        if let Some(ending) = roles.end {
            match ending {
                WorkflowEndingActorProvenance::Principal(_) => {}
                WorkflowEndingActorProvenance::HomeWorldOwner(owner) => {
                    if source.branch.owner_identity().durable() != Some(owner) {
                        return Err(source.corrupt(instance, kind));
                    }
                }
            }
        }
        if let Some(live_lineage) = source
            .population
            .one(layout.live_instance_lineage_relation, instance)?
        {
            if live_lineage != lineage || state != WorkflowInstanceState::Ready {
                return Err(source.corrupt(instance, kind));
            }
            let authored_branch = fields
                .u64(instance, kind, &layout.instance.branch_occurrence)
                .map_err(Denial::Record)?;
            // A fork's copied Ready row stays historical; it is not live work on this incarnation.
            if authored_branch == source.branch.lifecycle_incarnation().ordinal()
                && (effective != source.current || data.effective_revision() != source.current)
            {
                return Err(Denial::CurrentProgramMismatch { entity: instance });
            }
        }
    }
    Ok(())
}
