use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

use super::super::definition::CompiledWorkflowDefinition;
use super::super::schema::WorthQueryWorkflowLayout;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationCreationPartition, WorthQueryApplicationRealizedEffect,
};

/// What an instance carries from its lineage: the steps its sources took, the
/// assessment evidence bytes they retain and the earliest total deadline any
/// definition it ran declared.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorkflowLineageCarry {
    pub(in crate::domain_computation::primary_graph) inherited_steps: u64,
    pub(in crate::domain_computation::primary_graph) inherited_evidence_bytes: u64,
    /// Unix-epoch milliseconds.
    pub(in crate::domain_computation::primary_graph) deadline: Option<u64>,
}

pub(in crate::domain_computation::primary_graph) fn visit_instance_start_facts<Error>(
    layout: &WorthQueryWorkflowLayout,
    compiled: &CompiledWorkflowDefinition,
    instance_identity: &str,
    branch_occurrence: u64,
    subject: EntityId,
    carry: WorkflowLineageCarry,
    mut emit: impl FnMut(WorthQueryApplicationRealizedEffect) -> Result<(), Error>,
) -> Result<CreatedEntityRef, Error> {
    let partition = compiled.definition().partition_id;
    let instance = CreatedEntityRef {
        partition_id: partition,
        kind_id: layout.instance.entity_kind,
        client_key: ClientKey::raw("instance"),
    };
    let mut fields = BTreeMap::from([
        (layout.instance.identity.clone(), text(instance_identity)),
        (
            layout.instance.protocol_version.clone(),
            AspectValue::UInt64(
                super::super::schema::version::WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION,
            ),
        ),
        (
            layout.instance.branch_occurrence.clone(),
            AspectValue::UInt64(branch_occurrence),
        ),
        (
            layout.instance.program_revision.clone(),
            text(compiled.program_revision().to_string()),
        ),
        (
            layout.instance.definition_content_identity.clone(),
            text(compiled.content_identity().to_string()),
        ),
        (
            layout.instance.subject_partition.clone(),
            AspectValue::UInt64(subject.partition_value_u64()),
        ),
        (
            layout.instance.subject_slot.clone(),
            AspectValue::UInt64(subject.local_slot_value()),
        ),
        (
            layout.instance.subject_generation.clone(),
            AspectValue::UInt64(u64::from(subject.generation_value())),
        ),
        (
            layout.instance.state.clone(),
            AspectValue::UInt64(super::state::WorkflowInstanceState::Ready.persisted_tag()),
        ),
    ]);
    if let Some(path) = compiled.resumed_path() {
        fields.insert(layout.instance.resume_node_path.clone(), text(path));
    }
    if carry.inherited_steps > 0 {
        fields.insert(
            layout.instance.inherited_steps.clone(),
            AspectValue::UInt64(carry.inherited_steps),
        );
    }
    if carry.inherited_evidence_bytes > 0 {
        fields.insert(
            layout.instance.inherited_evidence_bytes.clone(),
            AspectValue::UInt64(carry.inherited_evidence_bytes),
        );
    }
    if let Some(deadline) = carry.deadline {
        fields.insert(
            layout.instance.deadline.clone(),
            AspectValue::UInt64(deadline),
        );
    }
    emit(WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: instance.kind_id,
        key: "instance".to_owned(),
        fields,
        partition: WorthQueryApplicationCreationPartition::Context(partition),
    })?;
    emit(WorthQueryApplicationRealizedEffect::CreateRelation {
        kind: layout.instance_lineage_relation,
        key: "instance-lineage".to_owned(),
        from: EntityReference::Created(instance.clone()),
        to: EntityReference::Existing(compiled.lineage()),
    })?;
    emit(WorthQueryApplicationRealizedEffect::CreateRelation {
        kind: layout.live_instance_lineage_relation,
        key: "live-instance-lineage".to_owned(),
        from: EntityReference::Created(instance.clone()),
        to: EntityReference::Existing(compiled.lineage()),
    })?;
    emit(WorthQueryApplicationRealizedEffect::CreateRelation {
        kind: layout.instance_definition_relation,
        key: "instance-definition".to_owned(),
        from: EntityReference::Created(instance.clone()),
        to: EntityReference::Existing(compiled.definition()),
    })?;
    Ok(instance)
}

fn text(value: impl Into<String>) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.into()))
}
