use worth_foundational::facade::{aspects, AspectIdentity, ScalarAspectType};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId};

use super::{relations::register_platform_entity, WorkflowAssessmentEvidenceLayout};
use crate::domain_computation::primary_graph::schema_layout::{
    invalid_member, planned_field_locator,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial;

const EVIDENCE_ENTITY: &str = "worth-query-workflow-assessment-evidence";
const EVIDENCE_ASPECT: &str = "workflow-assessment-evidence";

pub(super) fn lower_assessment_evidence(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    version: SchemaVersionId,
    kind: KindId,
    identity: AspectIdentity,
) -> Result<
    (RelationalSchemaRegistry, WorkflowAssessmentEvidenceLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let layout = WorkflowAssessmentEvidenceLayout {
        entity_kind: kind,
        identity: planned_field_locator(EVIDENCE_ASPECT, "identity")?,
        protocol_version: planned_field_locator(EVIDENCE_ASPECT, "protocol-version")?,
        producer: planned_field_locator(EVIDENCE_ASPECT, "producer")?,
        family: planned_field_locator(EVIDENCE_ASPECT, "family")?,
        query: planned_field_locator(EVIDENCE_ASPECT, "query")?,
        parameter_type: planned_field_locator(EVIDENCE_ASPECT, "parameter-type")?,
        result_type: planned_field_locator(EVIDENCE_ASPECT, "result-type")?,
        binding: planned_field_locator(EVIDENCE_ASPECT, "binding")?,
        subject_partition: planned_field_locator(EVIDENCE_ASPECT, "subject-partition")?,
        subject_slot: planned_field_locator(EVIDENCE_ASPECT, "subject-slot")?,
        subject_generation: planned_field_locator(EVIDENCE_ASPECT, "subject-generation")?,
        proposal_identity: planned_field_locator(EVIDENCE_ASPECT, "proposal-identity")?,
        coverage_identity: planned_field_locator(EVIDENCE_ASPECT, "coverage-identity")?,
        source_identity: planned_field_locator(EVIDENCE_ASPECT, "source-identity")?,
        passing: planned_field_locator(EVIDENCE_ASPECT, "passing")?,
        publication_identity: planned_field_locator(EVIDENCE_ASPECT, "publication-identity")?,
        output_content_identity: planned_field_locator(EVIDENCE_ASPECT, "output-content-identity")?,
        program_revision: planned_field_locator(EVIDENCE_ASPECT, "program-revision")?,
    };
    let shape = aspects()
        .struct_fields()
        .required("identity", ScalarAspectType::String)
        .required("protocol-version", ScalarAspectType::UInt64)
        .required("producer", ScalarAspectType::String)
        .required("family", ScalarAspectType::String)
        .required("query", ScalarAspectType::String)
        .required("parameter-type", ScalarAspectType::String)
        .required("result-type", ScalarAspectType::String)
        .required("binding", ScalarAspectType::String)
        .required("subject-partition", ScalarAspectType::UInt64)
        .required("subject-slot", ScalarAspectType::UInt64)
        .required("subject-generation", ScalarAspectType::UInt64)
        .required("proposal-identity", ScalarAspectType::String)
        .required("coverage-identity", ScalarAspectType::String)
        .required("source-identity", ScalarAspectType::String)
        .required("passing", ScalarAspectType::Bool)
        .required("publication-identity", ScalarAspectType::String)
        .required("output-content-identity", ScalarAspectType::String)
        .optional("program-revision", ScalarAspectType::String)
        .finish()
        .map_err(|_| invalid_member(EVIDENCE_ASPECT))?;
    let registry = register_platform_entity(
        registry,
        schema_id,
        version,
        EVIDENCE_ENTITY,
        kind,
        EVIDENCE_ASPECT,
        identity,
        shape,
    )?;
    Ok((registry, layout))
}
