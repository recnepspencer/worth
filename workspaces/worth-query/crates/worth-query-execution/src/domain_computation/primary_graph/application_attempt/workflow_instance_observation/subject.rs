//! Read subject correspondence according to the persisted instance protocol.
use super::{
    adjacency, denial, exact, observe_field_value, AspectValue, EntityId,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
    WorthQueryWorkflowLayout,
};

pub(super) fn observe_subject(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    entity: EntityId,
    subject: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
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
            if version == WORKFLOW_BIRTH_FACT_PROTOCOL_VERSION {
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
                if subjects.as_slice() != [subject] {
                    return Err(denial("workflow instance subject binding changed"));
                }
            } else {
                for (locator, expected) in [
                    (
                        &layout.instance.subject_partition,
                        u64::from(subject.partition_value()),
                    ),
                    (&layout.instance.subject_slot, subject.local_slot_value()),
                    (
                        &layout.instance.subject_generation,
                        u64::from(subject.generation_value()),
                    ),
                ] {
                    exact(
                        execution,
                        runtime,
                        snapshot,
                        entity,
                        kind,
                        locator,
                        AspectValue::UInt64(expected),
                        facts,
                    )?;
                }
                // A legacy coordinate record must not acquire a conflicting
                // v3 endpoint through another publication.
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
            }
            Ok(())
        }
        _ => Err(denial("workflow instance protocol is unavailable")),
    }
}
