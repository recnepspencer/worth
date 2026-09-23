//! Query-owned relational vocabulary for authored workflow model facts.
//!
//! These kinds extend the application's existing schema registry. They do not
//! create another graph, scheduler, or state store.

use worth_foundational::facade::AspectIdentity;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::indexes::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
use worth_relational::facade::schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId};

mod approval;
mod assessment;
mod evidence_dependency;
mod layout;
mod proposal;
mod relations;
mod transition;
pub(in crate::domain_computation::primary_graph) mod version;

pub(in crate::domain_computation::primary_graph) use layout::*;

use approval::lower_approval;
use evidence_dependency::lower_evidence_dependency;
use proposal::lower_proposal;
use relations::{
    allocate_kinds, connection_endpoint_integrity, current_definition_integrity, lower_connection,
    lower_definition, lower_instance, lower_lineage, lower_node, owned_fact_integrity,
};
use transition::lower_transition;

use super::super::schema_layout::register_relation;
use super::super::WorthQueryPrimaryGraphInstallationDenial;

const LINEAGE_ENTITY: &str = "worth-query-workflow-lineage";
const DEFINITION_ENTITY: &str = "worth-query-workflow-definition";
const NODE_ENTITY: &str = "worth-query-workflow-node";
const CONNECTION_ENTITY: &str = "worth-query-workflow-connection";
const INSTANCE_ENTITY: &str = "worth-query-workflow-instance";
const LINEAGE_ASPECT: &str = "workflow-lineage";
const DEFINITION_ASPECT: &str = "workflow-definition";
const NODE_ASPECT: &str = "workflow-node";
const CONNECTION_ASPECT: &str = "workflow-connection";
const INSTANCE_ASPECT: &str = "workflow-instance";

pub(in crate::domain_computation::primary_graph) fn lower_workflow(
    mut registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    schema_version_id: SchemaVersionId,
    first_kind: KindId,
    identities: [AspectIdentity; 10],
) -> Result<
    (RelationalSchemaRegistry, WorthQueryWorkflowLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let kinds = allocate_kinds(first_kind)?;
    let (next, lineage) = lower_lineage(
        registry,
        schema_id,
        schema_version_id,
        kinds[0],
        identities[0],
    )?;
    registry = next;
    let (next, definition) = lower_definition(
        registry,
        schema_id,
        schema_version_id,
        kinds[1],
        identities[1],
    )?;
    registry = next;
    let (next, transition) = lower_transition(
        registry,
        schema_id,
        schema_version_id,
        kinds[14],
        identities[5],
    )?;
    registry = next;
    let (next, proposal) = lower_proposal(
        registry,
        schema_id,
        schema_version_id,
        kinds[18],
        identities[6],
    )?;
    registry = next;
    let (next, assessment_evidence) = lower_assessment_evidence(
        registry,
        schema_id,
        schema_version_id,
        kinds[20],
        identities[7],
    )?;
    registry = next;
    let (next, approval) = lower_approval(
        registry,
        schema_id,
        schema_version_id,
        kinds[22],
        identities[8],
    )?;
    registry = next;
    let (next, evidence_dependency) = lower_evidence_dependency(
        registry,
        schema_id,
        schema_version_id,
        kinds[26],
        identities[9],
    )?;
    registry = next;
    let (next, node) = lower_node(
        registry,
        schema_id,
        schema_version_id,
        kinds[2],
        identities[2],
    )?;
    registry = next;
    let (next, connection) = lower_connection(
        registry,
        schema_id,
        schema_version_id,
        kinds[3],
        identities[3],
    )?;
    registry = next;
    let (next, instance) = lower_instance(
        registry,
        schema_id,
        schema_version_id,
        kinds[4],
        identities[4],
    )?;
    registry = next;

    let relations = [
        (
            "worth-query-workflow-current-definition",
            kinds[0],
            kinds[1],
            current_definition_integrity(),
        ),
        (
            "worth-query-workflow-lineage-definition",
            kinds[0],
            kinds[1],
            owned_fact_integrity(),
        ),
        (
            "worth-query-workflow-definition-node",
            kinds[1],
            kinds[2],
            owned_fact_integrity(),
        ),
        (
            "worth-query-workflow-definition-connection",
            kinds[1],
            kinds[3],
            owned_fact_integrity(),
        ),
        (
            "worth-query-workflow-connection-source",
            kinds[3],
            kinds[2],
            connection_endpoint_integrity(),
        ),
        (
            "worth-query-workflow-connection-target",
            kinds[3],
            kinds[2],
            connection_endpoint_integrity(),
        ),
        (
            "worth-query-workflow-definition-start",
            kinds[1],
            kinds[2],
            current_definition_integrity(),
        ),
    ];
    for (offset, (name, from, to, integrity)) in relations.into_iter().enumerate() {
        registry = register_relation(
            registry,
            schema_id,
            schema_version_id,
            name,
            kinds[5 + offset],
            from,
            to,
            integrity,
        )?;
    }
    registry = register_relation(
        registry,
        schema_id,
        schema_version_id,
        "worth-query-workflow-instance-lineage",
        kinds[12],
        kinds[4],
        kinds[0],
        connection_endpoint_integrity(),
    )?;
    registry = register_relation(
        registry,
        schema_id,
        schema_version_id,
        "worth-query-workflow-transition-proposal",
        kinds[19],
        kinds[14],
        kinds[18],
        relations::workflow_proposal_integrity(),
    )?;
    registry = register_relation(
        registry,
        schema_id,
        schema_version_id,
        "worth-query-workflow-transition-assessment-evidence",
        kinds[21],
        kinds[14],
        kinds[20],
        relations::assessment_evidence_integrity(),
    )?;
    registry = register_relation(
        registry,
        schema_id,
        schema_version_id,
        "worth-query-workflow-transition-approval",
        kinds[23],
        kinds[14],
        kinds[22],
        relations::assessment_evidence_integrity(),
    )?;
    registry = register_relation(
        registry,
        schema_id,
        schema_version_id,
        "worth-query-workflow-approval-proposal",
        kinds[24],
        kinds[22],
        kinds[18],
        relations::approval_proposal_integrity(),
    )?;
    registry = register_relation(
        registry,
        schema_id,
        schema_version_id,
        "worth-query-workflow-approval-evidence",
        kinds[25],
        kinds[22],
        kinds[20],
        relations::approval_evidence_integrity(),
    )?;
    registry = register_relation(
        registry,
        schema_id,
        schema_version_id,
        "worth-query-workflow-evidence-dependency",
        kinds[27],
        kinds[20],
        kinds[26],
        owned_fact_integrity(),
    )?;
    registry = register_relation(
        registry,
        schema_id,
        schema_version_id,
        "worth-query-workflow-live-instance-lineage",
        kinds[17],
        kinds[4],
        kinds[0],
        relations::live_membership_integrity(),
    )?;
    registry = register_relation(
        registry,
        schema_id,
        schema_version_id,
        "worth-query-workflow-instance-transition",
        kinds[15],
        kinds[4],
        kinds[14],
        owned_fact_integrity(),
    )?;
    registry = register_relation(
        registry,
        schema_id,
        schema_version_id,
        "worth-query-workflow-transition-node",
        kinds[16],
        kinds[14],
        kinds[2],
        connection_endpoint_integrity(),
    )?;
    registry = register_relation(
        registry,
        schema_id,
        schema_version_id,
        "worth-query-workflow-instance-definition",
        kinds[13],
        kinds[4],
        kinds[1],
        connection_endpoint_integrity(),
    )?;
    Ok((
        registry,
        WorthQueryWorkflowLayout {
            lineage,
            definition,
            node,
            connection,
            instance,
            transition,
            proposal,
            assessment_evidence,
            approval,
            evidence_dependency,
            current_definition_relation: kinds[5],
            lineage_definition_relation: kinds[6],
            definition_node_relation: kinds[7],
            definition_connection_relation: kinds[8],
            connection_source_relation: kinds[9],
            connection_target_relation: kinds[10],
            definition_start_relation: kinds[11],
            instance_lineage_relation: kinds[12],
            live_instance_lineage_relation: kinds[17],
            instance_definition_relation: kinds[13],
            instance_transition_relation: kinds[15],
            transition_node_relation: kinds[16],
            transition_proposal_relation: kinds[19],
            transition_assessment_evidence_relation: kinds[21],
            transition_approval_relation: kinds[23],
            approval_proposal_relation: kinds[24],
            approval_evidence_relation: kinds[25],
            evidence_dependency_relation: kinds[27],
        },
    ))
}

pub(in crate::domain_computation::primary_graph) fn register_indexes(
    layout: &mut WorthQueryWorkflowLayout,
    mut install: impl FnMut(DerivedIndexDefinition) -> Result<DerivedIndexDefinition, String>,
) -> Result<(), String> {
    let installed = install(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "worth-query-workflow.lineage-identity".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: layout.lineage.identity.clone(),
        },
        branch_scoped: true,
    })?;
    layout.lineage.identity_index_id = installed.index_id;
    let installed = install(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "worth-query-workflow.definition-content-identity".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: layout.definition.content_identity.clone(),
        },
        branch_scoped: true,
    })?;
    layout.definition.content_identity_index_id = installed.index_id;
    let installed = install(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "worth-query-workflow.instance-identity".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: layout.instance.identity.clone(),
        },
        branch_scoped: true,
    })?;
    layout.instance.identity_index_id = installed.index_id;
    let installed = install(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "worth-query-workflow.proposal-identity".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: layout.proposal.identity.clone(),
        },
        branch_scoped: true,
    })?;
    layout.proposal.identity_index_id = installed.index_id;
    Ok(())
}
use assessment::lower_assessment_evidence;
