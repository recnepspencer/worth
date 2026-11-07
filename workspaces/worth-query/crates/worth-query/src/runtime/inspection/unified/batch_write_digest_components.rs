use super::WorthQueryBatchWriteComponentInspection;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::{WorthQueryAspectTouch, WorthQueryGraphCompositionResolutionMap};

pub(super) fn component_artifact_identities(
    components: &[WorthQueryBatchWriteComponentInspection],
) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    Ok(components
        .iter()
        .enumerate()
        .map(|(index, component)| component_artifact_identity(index, component))
        .collect::<Result<Vec<_>, _>>()?)
}

pub(super) fn graph_resolution_identities(
    resolution_map: &WorthQueryGraphCompositionResolutionMap,
) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    Ok(resolution_map
        .entries()
        .iter()
        .map(|entry| {
            let aspect_digest = entry
                .aspect_touch()
                .map(WorthQueryAspectTouch::admitted_touch_digest_part);
            worth_query_evidence_identity(WorthQueryEvidenceScope::BatchWriteReceiptGraphResolution)
                .field_usize(
                    WorthQueryEvidenceTag::new("component_index"),
                    entry.component_index(),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("admitted_aspect_touch"),
                    aspect_digest.as_deref(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("symbol_identity"),
                    entry.symbol().evidence_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("resolved_entity_identity"),
                    &entry.resolved_entity_identity().evidence_identity(),
                )
                .optional_evidence_identity(
                    WorthQueryEvidenceTag::new("target_collection"),
                    entry
                        .target_collection()
                        .map(|collection| collection.evidence_identity()),
                )
                .seal()
        })
        .collect::<Result<Vec<_>, _>>()?)
}

fn component_artifact_identity(
    index: usize,
    component: &WorthQueryBatchWriteComponentInspection,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let symbolic_aspect_resolution_identities = symbolic_aspect_resolution_identities(component);
    let declared_aspect_operation_identities = declared_aspect_operation_identities(component);
    let target_declared_entity_identity = component
        .target_evidence()
        .declared()
        .entity_identity()
        .map(|identity| identity.evidence_identity());
    let target_resolved_entity_identity = component
        .target_evidence()
        .resolved()
        .entity_identity()
        .map(|identity| identity.evidence_identity());
    let existing_truth_resolved_target_identity = component
        .existing_truth_binding_evidence()
        .map(|evidence| evidence.resolved_target_identity().evidence_identity());
    let entity_identities = component
        .entity_identities()
        .iter()
        .map(|identity| identity.evidence_identity())
        .collect::<Result<Vec<_>, _>>()?;
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::BatchWriteReceiptComponent)
            .field_usize(WorthQueryEvidenceTag::new("index"), index)
            .field_shape(WorthQueryEvidenceTag::new("family"), component.family())
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("commit_identity"),
                &component.commit_identity().evidence_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("target_declared_class"),
                component
                    .target_evidence()
                    .declared()
                    .target_class()
                    .as_str(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("target_declared_collection"),
                component
                    .target_evidence()
                    .declared()
                    .collection()
                    .map(|collection| collection.evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("target_declared_entity_identity"),
                target_declared_entity_identity.as_ref(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("target_resolved_class"),
                component
                    .target_evidence()
                    .resolved()
                    .target_class()
                    .as_str(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("target_resolved_collection"),
                component
                    .target_evidence()
                    .resolved()
                    .collection()
                    .map(|collection| collection.evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("target_resolved_entity_identity"),
                target_resolved_entity_identity.as_ref(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("existing_assertion_verification_digest"),
                component
                    .existing_truth_assertion_evidence()
                    .map(|evidence| evidence.verification_evidence_identity()),
            )
            .optional_shape(
                WorthQueryEvidenceTag::new("existing_truth_family"),
                component
                    .existing_truth_binding_evidence()
                    .map(|evidence| evidence.family().as_str()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("existing_truth_authoritative_identity"),
                component
                    .existing_truth_binding_evidence()
                    .map(|evidence| evidence.authoritative_identity().evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("existing_truth_resolved_target_identity"),
                existing_truth_resolved_target_identity.as_ref(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("existing_truth_target_collection"),
                component
                    .existing_truth_binding_evidence()
                    .and_then(|evidence| evidence.target_collection())
                    .map(|collection| collection.evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("existing_truth_binding_digest"),
                component
                    .existing_truth_binding_evidence()
                    .map(|evidence| evidence.binding_digest().evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("symbolic_target_symbol"),
                component
                    .symbolic_target_reference_evidence()
                    .map(|evidence| evidence.symbol().evidence_identity()),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("symbolic_aspect_resolution_identity"),
                symbolic_aspect_resolution_identities.iter(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("causality_digest"),
                component
                    .causality_evidence()
                    .map(|evidence| evidence.causality_digest().evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("continuity_lineage_digest"),
                component
                    .continuity_mutation_evidence()
                    .map(|evidence| evidence.lineage_digest().evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("continuity_resolution_digest"),
                component
                    .continuity_mutation_evidence()
                    .map(|evidence| evidence.continuity_resolution_digest().evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("provenance_execution_record_digest"),
                component
                    .provenance_evidence()
                    .map(|evidence| evidence.execution_record_digest().evidence_identity()),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("collection"),
                evidence_value_identities("component-collection", component.collections()).iter(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("entity_identity"),
                entity_identities.iter(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("declared_aspect_operation"),
                declared_aspect_operation_identities.iter(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("touched_aspect"),
                terminal_touch_projection_identities(
                    "component-touched-aspect",
                    component.admitted_touched_aspects(),
                )
                .iter(),
            )
            .seal()?,
    )
}

fn evidence_value_identities(
    role: &'static str,
    values: &[String],
) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    Ok(values
        .iter()
        .map(|value| {
            worth_query_evidence_identity(
                WorthQueryEvidenceScope::BatchWriteReceiptInspectionArtifact,
            )
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_value(WorthQueryEvidenceTag::new("value"), value)
            .seal()
        })
        .collect::<Result<Vec<_>, _>>()?)
}

fn terminal_touch_projection_identities(
    role: &'static str,
    touches: &[WorthQueryAspectTouch],
) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    Ok(touches
        .iter()
        .map(|touch| {
            worth_query_evidence_identity(
                WorthQueryEvidenceScope::BatchWriteReceiptInspectionArtifact,
            )
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_value(
                WorthQueryEvidenceTag::new("value"),
                touch.admitted_touch_digest_part(),
            )
            .seal()
        })
        .collect::<Result<Vec<_>, _>>()?)
}

fn symbolic_aspect_resolution_identities(
    component: &WorthQueryBatchWriteComponentInspection,
) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    Ok(component
        .symbolic_aspect_resolution_evidence()
        .iter()
        .map(|evidence| {
            worth_query_evidence_identity(
                WorthQueryEvidenceScope::BatchWriteReceiptSymbolicAspectResolution,
            )
            .field_value(
                WorthQueryEvidenceTag::new("admitted_aspect_touch"),
                evidence.aspect_touch().admitted_touch_digest_part(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("symbol_identity"),
                evidence.symbol().evidence_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("resolved_entity_identity"),
                &evidence.resolved_entity_identity().evidence_identity(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("target_collection"),
                evidence
                    .target_collection()
                    .map(|collection| collection.evidence_identity()),
            )
            .seal()
        })
        .collect::<Result<Vec<_>, _>>()?)
}

fn declared_aspect_operation_identities(
    component: &WorthQueryBatchWriteComponentInspection,
) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    Ok(component
        .declared_aspect_operations()
        .iter()
        .map(|operation| {
            worth_query_evidence_identity(
                WorthQueryEvidenceScope::WriteReceiptDeclaredAspectOperation,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("kind"),
                operation.kind().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("admitted_aspect_touch"),
                operation.aspect_touch().admitted_touch_digest_part(),
            )
            .seal()
        })
        .collect::<Result<Vec<_>, _>>()?)
}
