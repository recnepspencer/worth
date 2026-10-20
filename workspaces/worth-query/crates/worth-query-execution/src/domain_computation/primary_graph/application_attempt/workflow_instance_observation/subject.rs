//! Read subject correspondence according to the persisted instance protocol.
use super::{
    adjacency, denial, exact, observe_field_value, AspectValue, EntityId,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
    WorthQueryWorkflowLayout,
};

pub(in crate::domain_computation::primary_graph) fn observe_subject(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    entity: EntityId,
    subject: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    if read_subject(execution, runtime, snapshot, layout, entity, facts)? != subject {
        return Err(denial("workflow instance subject binding changed"));
    }
    Ok(())
}

pub(super) fn read_subject(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    entity: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<EntityId, WorthQueryApplicationAttemptDenial> {
    let kind = layout.instance.entity_kind;
    let protocol = observe_field_value(
        runtime,
        execution,
        snapshot,
        entity,
        kind,
        &layout.instance.protocol_version,
    )?;
    use crate::domain_computation::primary_graph::workflow::schema::version::{
        WORKFLOW_BIRTH_FACT_PROTOCOL_VERSION, WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION,
    };
    match protocol {
        Some(AspectValue::UInt64(
            version @ (WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION
            | WORKFLOW_BIRTH_FACT_PROTOCOL_VERSION),
        )) => {
            exact(
                execution,
                runtime,
                snapshot,
                entity,
                kind,
                &layout.instance.protocol_version,
                AspectValue::UInt64(version),
                facts,
            )?;
            let subject = if version == WORKFLOW_BIRTH_FACT_PROTOCOL_VERSION {
                for locator in [
                    &layout.instance.subject_partition,
                    &layout.instance.subject_slot,
                    &layout.instance.subject_generation,
                ] {
                    if observe_field_value(runtime, execution, snapshot, entity, kind, locator)?
                        .is_some()
                    {
                        return Err(denial("workflow birth has conflicting subject coordinates"));
                    }
                    facts.push(WorthQueryApplicationObservedFact::AbsentField {
                        entity_id: entity,
                        kind,
                        locator: locator.clone(),
                    });
                }
                let subjects = adjacency(
                    execution,
                    runtime,
                    snapshot,
                    layout.instance_subject_relation,
                    entity,
                    2,
                    "workflow subject correspondence is unavailable",
                    facts,
                )?;
                let [subject] = subjects.as_slice() else {
                    return Err(denial("workflow instance subject binding is not singular"));
                };
                *subject
            } else {
                let mut components = [0u64; 3];
                for (index, locator) in [
                    &layout.instance.subject_partition,
                    &layout.instance.subject_slot,
                    &layout.instance.subject_generation,
                ]
                .into_iter()
                .enumerate()
                {
                    components[index] = super::exact_u64(
                        execution, runtime, snapshot, entity, kind, locator, facts,
                    )?;
                }
                let subject = EntityId::new(
                    worth_relational::facade::identity::PartitionId::new(
                        u32::try_from(components[0])
                            .map_err(|_| denial("workflow subject partition is invalid"))?,
                    ),
                    components[1],
                    u32::try_from(components[2])
                        .map_err(|_| denial("workflow subject generation is invalid"))?,
                );
                // A legacy coordinate record must not acquire a conflicting
                // birth endpoint through another publication.
                if !adjacency(
                    execution,
                    runtime,
                    snapshot,
                    layout.instance_subject_relation,
                    entity,
                    2,
                    "workflow subject correspondence is unavailable",
                    facts,
                )?
                .is_empty()
                {
                    return Err(denial("legacy workflow has conflicting subject relation"));
                }
                subject
            };
            Ok(subject)
        }
        _ => Err(denial("workflow instance protocol is unavailable")),
    }
}
