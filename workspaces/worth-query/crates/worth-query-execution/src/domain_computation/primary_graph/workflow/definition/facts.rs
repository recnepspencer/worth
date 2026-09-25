use std::collections::BTreeMap;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_program::{
    ApplicationProgramRevision, ApplicationWorkflowNodeKind, ApplicationWorkflowSpec,
    ValidatedWorkflowDefinition,
};
use worth_relational::facade::identity::{EntityId, PartitionId, RelationId};
use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

use super::super::super::application_attempt::{
    WorthQueryApplicationCreationPartition, WorthQueryApplicationRealizedEffect,
};
use super::super::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION;
use super::super::schema::WorthQueryWorkflowLayout;
use super::codec::WorkflowNodeTag;

#[path = "facts/encoding.rs"]
mod encoding;
use encoding::*;

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) enum WorkflowLineagePublicationTarget {
    New,
    Existing {
        lineage: EntityId,
        current_relation: RelationId,
    },
}

struct WorkflowDefinitionPublicationContext {
    lineage: EntityReference,
    creation_partition: WorthQueryApplicationCreationPartition,
    symbolic_partition: PartitionId,
}

pub(in crate::domain_computation::primary_graph) fn visit_definition_facts<Spec, Error>(
    layout: &WorthQueryWorkflowLayout,
    program_revision: &ApplicationProgramRevision,
    definition: &ValidatedWorkflowDefinition<Spec>,
    assessment_bindings: &[(String, &'static str)],
    condition_bindings: &[(String, &'static str)],
    approval_bindings: &[worth_query_installation::facade::WorthQueryInstalledWorkflowApprovalBinding],
    lineage: WorkflowLineagePublicationTarget,
    mut emit: impl FnMut(WorthQueryApplicationRealizedEffect) -> Result<(), Error>,
) -> Result<CreatedEntityRef, Error>
where
    Spec: ApplicationWorkflowSpec,
{
    let limits = definition.limits();
    let component_limits = limits.component_limits();
    let fields = BTreeMap::from([
        (
            layout.definition.content_identity.clone(),
            text(definition.content_identity().to_string()),
        ),
        (
            layout.definition.program_revision.clone(),
            text(program_revision.to_string()),
        ),
        (
            layout.definition.maximum_nodes.clone(),
            AspectValue::UInt64(u64::from(limits.maximum_nodes())),
        ),
        (
            layout.definition.maximum_connections.clone(),
            AspectValue::UInt64(u64::from(limits.maximum_connections())),
        ),
        (
            layout.definition.maximum_effects.clone(),
            AspectValue::UInt64(u64::from(limits.maximum_effects())),
        ),
        (
            layout.definition.maximum_component_occurrences.clone(),
            AspectValue::UInt64(u64::from(component_limits.maximum_occurrences())),
        ),
        (
            layout.definition.maximum_component_depth.clone(),
            AspectValue::UInt64(u64::from(component_limits.maximum_depth())),
        ),
        (
            layout.definition.maximum_node_provenance.clone(),
            AspectValue::UInt64(u64::from(component_limits.maximum_node_provenance())),
        ),
        (
            layout.definition.maximum_connection_provenance.clone(),
            AspectValue::UInt64(u64::from(component_limits.maximum_connection_provenance())),
        ),
        (
            layout.definition.maximum_port_provenance.clone(),
            AspectValue::UInt64(u64::from(component_limits.maximum_port_provenance())),
        ),
        (
            layout.definition.maximum_canonical_bytes.clone(),
            AspectValue::UInt64(u64::from(limits.maximum_canonical_bytes())),
        ),
    ]);
    let publication = stage_lineage(layout, definition, lineage, &mut emit)?;
    let definition_ref = created(
        publication.symbolic_partition,
        layout.definition.entity_kind,
        "definition",
    );
    emit(WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: definition_ref.kind_id,
        key: raw_key(&definition_ref),
        fields,
        partition: publication.creation_partition,
    })?;
    let mut nodes = BTreeMap::new();
    for (ordinal, node) in definition.nodes().iter().enumerate() {
        let node_ref = created(
            publication.symbolic_partition,
            layout.node.entity_kind,
            format!("node-{ordinal}"),
        );
        let assessment_binding = assessment_bindings
            .iter()
            .find_map(|(path, binding)| (path == node.identity().as_str()).then_some(*binding));
        let approval_binding = approval_bindings
            .iter()
            .find(|binding| binding.node_path == node.identity().as_str());
        let condition_binding = condition_bindings
            .iter()
            .find_map(|(path, binding)| (path == node.identity().as_str()).then_some(*binding));
        emit(create_node(
            layout,
            &node_ref,
            node,
            assessment_binding,
            condition_binding,
            approval_binding,
            publication.creation_partition,
        ))?;
        emit(create_relation(
            layout.definition_node_relation,
            format!("definition-node-{ordinal}"),
            EntityReference::Created(definition_ref.clone()),
            EntityReference::Created(node_ref.clone()),
        ))?;
        nodes.insert(node.identity().as_str(), node_ref);
    }
    let start = nodes
        .get(definition.start().as_str())
        .expect("validated start node is present")
        .clone();
    emit(create_relation(
        layout.definition_start_relation,
        "definition-start",
        EntityReference::Created(definition_ref.clone()),
        EntityReference::Created(start),
    ))?;
    for (ordinal, connection) in definition.connections().iter().enumerate() {
        let connection_ref = created(
            publication.symbolic_partition,
            layout.connection.entity_kind,
            format!("connection-{ordinal}"),
        );
        let source = nodes
            .get(connection.source().as_str())
            .expect("validated connection source is present")
            .clone();
        let target = nodes
            .get(connection.target().as_str())
            .expect("validated connection target is present")
            .clone();
        emit(create_connection(
            layout,
            &connection_ref,
            connection.kind(),
            publication.creation_partition,
        ))?;
        emit(create_relation(
            layout.definition_connection_relation,
            format!("definition-connection-{ordinal}"),
            EntityReference::Created(definition_ref.clone()),
            EntityReference::Created(connection_ref.clone()),
        ))?;
        emit(create_relation(
            layout.connection_source_relation,
            format!("connection-source-{ordinal}"),
            EntityReference::Created(connection_ref.clone()),
            EntityReference::Created(source),
        ))?;
        emit(create_relation(
            layout.connection_target_relation,
            format!("connection-target-{ordinal}"),
            EntityReference::Created(connection_ref.clone()),
            EntityReference::Created(target),
        ))?;
    }
    emit(create_relation(
        layout.lineage_definition_relation,
        "lineage-definition",
        publication.lineage.clone(),
        EntityReference::Created(definition_ref.clone()),
    ))?;
    emit(create_relation(
        layout.current_definition_relation,
        "lineage-current-definition",
        publication.lineage,
        EntityReference::Created(definition_ref.clone()),
    ))?;
    Ok(definition_ref)
}

fn stage_lineage<Spec: ApplicationWorkflowSpec, Error>(
    layout: &WorthQueryWorkflowLayout,
    definition: &ValidatedWorkflowDefinition<Spec>,
    lineage: WorkflowLineagePublicationTarget,
    emit: &mut impl FnMut(WorthQueryApplicationRealizedEffect) -> Result<(), Error>,
) -> Result<WorkflowDefinitionPublicationContext, Error> {
    match lineage {
        WorkflowLineagePublicationTarget::New => {
            let creation_partition = WorthQueryApplicationCreationPartition::Issued;
            let symbolic_partition = PartitionId::main();
            let reference = created(symbolic_partition, layout.lineage.entity_kind, "lineage");
            let fields = BTreeMap::from([
                (
                    layout.lineage.identity.clone(),
                    text(definition.identity().as_str()),
                ),
                (layout.lineage.spec.clone(), text(Spec::IDENTITY.as_str())),
                (
                    layout.lineage.protocol_version.clone(),
                    AspectValue::UInt64(WORKFLOW_FACT_PROTOCOL_VERSION),
                ),
            ]);
            emit(WorthQueryApplicationRealizedEffect::CreateEntity {
                kind: reference.kind_id,
                key: raw_key(&reference),
                fields,
                partition: creation_partition,
            })?;
            Ok(WorkflowDefinitionPublicationContext {
                lineage: EntityReference::Created(reference),
                creation_partition,
                symbolic_partition,
            })
        }
        WorkflowLineagePublicationTarget::Existing {
            lineage,
            current_relation,
        } => {
            emit(WorthQueryApplicationRealizedEffect::DeleteRelation {
                relation_id: current_relation,
            })?;
            Ok(WorkflowDefinitionPublicationContext {
                lineage: EntityReference::Existing(lineage),
                creation_partition: WorthQueryApplicationCreationPartition::Context(
                    lineage.partition_id,
                ),
                symbolic_partition: lineage.partition_id,
            })
        }
    }
}

fn create_node(
    layout: &WorthQueryWorkflowLayout,
    reference: &CreatedEntityRef,
    node: &worth_query_declaration::facade::application_program::ApplicationWorkflowNode,
    assessment_binding: Option<&str>,
    condition_binding: Option<&str>,
    approval_binding: Option<
        &worth_query_installation::facade::WorthQueryInstalledWorkflowApprovalBinding,
    >,
    creation_partition: WorthQueryApplicationCreationPartition,
) -> WorthQueryApplicationRealizedEffect {
    let kind = WorkflowNodeTag::from_declared(node.kind()).persisted();
    let (member, input_type, parameter_type, result_type, capability_type, requires_authority) =
        match node.kind() {
            ApplicationWorkflowNodeKind::Operation {
                operation,
                requires_workflow_authority,
            } => (
                operation.identifier(),
                Some(operation.input_type().as_str()),
                None,
                None,
                None,
                *requires_workflow_authority,
            ),
            ApplicationWorkflowNodeKind::Assessment(assessment) => (
                assessment.identifier(),
                None,
                Some(assessment.parameter_type().as_str()),
                Some(assessment.result_type().as_str()),
                None,
                false,
            ),
            ApplicationWorkflowNodeKind::Condition(condition) => (
                condition.identifier(),
                None,
                Some(condition.parameter_type().as_str()),
                Some(condition.result_type().as_str()),
                None,
                false,
            ),
            ApplicationWorkflowNodeKind::Approval(approval) => (
                approval.identifier(),
                None,
                None,
                None,
                Some(approval.capability_type().as_str()),
                false,
            ),
            ApplicationWorkflowNodeKind::EvidenceJoin(policy) => {
                (policy.identity(), None, None, None, None, false)
            }
            ApplicationWorkflowNodeKind::Terminal => ("", None, None, None, None, false),
        };
    let mut fields = BTreeMap::from([
        (layout.node.path.clone(), text(node.identity().as_str())),
        (layout.node.kind.clone(), AspectValue::UInt64(kind)),
        (layout.node.member.clone(), text(member)),
        (
            layout.node.requires_authority.clone(),
            AspectValue::Bool(requires_authority),
        ),
    ]);
    for (locator, value) in [
        (&layout.node.input_type, input_type),
        (&layout.node.parameter_type, parameter_type),
        (&layout.node.result_type, result_type),
        (&layout.node.assessment_binding, assessment_binding),
        (&layout.node.condition_binding, condition_binding),
        (&layout.node.capability_type, capability_type),
        (
            &layout.node.approval_operation,
            approval_binding.map(|binding| binding.operation),
        ),
    ] {
        if let Some(value) = value {
            fields.insert(locator.clone(), text(value));
        }
    }
    if let Some(binding) = approval_binding {
        fields.insert(
            layout.node.approval_capability_identity.clone(),
            text(hex(binding.installed_capability_identity)),
        );
    }
    if let ApplicationWorkflowNodeKind::Assessment(assessment) = node.kind() {
        fields.insert(
            layout.node.assessment_subject.clone(),
            text(assessment.subject().persistence_identity()),
        );
    }
    WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: reference.kind_id,
        key: raw_key(reference),
        fields,
        partition: creation_partition,
    }
}
