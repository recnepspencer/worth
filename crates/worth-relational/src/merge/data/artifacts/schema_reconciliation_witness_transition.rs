use crate::merge::data::MergePolicyDecisionBoundary;
use crate::schema::data::{
    DescriptorCanonicalBasisVersion, DescriptorSemanticsVersion, FreeFormSchemaDiffIntent,
    HistoricalInterpretationSensitivity, ProposedSchemaTransition, SchemaDiffAtom,
    SchemaDiffDetail, SchemaElementKind, SchemaElementRef, SchemaId, SchemaPublicationImpact,
    SchemaReconciliationClassification, SchemaReconciliationPolicy, SchemaStratum,
    SchemaSubscriberImpact, SchemaVersionId,
};
use crate::schema::{
    classify_schema_transition, lower_schema_transition, validate_schema_transition,
};
use worth_foundational::FieldKey;

use super::super::MergeManualResolutionClass;
use super::schema_reconciliation_witness_rows::{
    RelationalSchemaReconciliationWitnessDenial, RelationalSchemaReconciliationWitnessPosture,
    RelationalSchemaReconciliationWitnessRowInput,
};

pub(super) struct DerivedSchemaReconciliationTruth {
    pub(super) classification: SchemaReconciliationClassification,
    pub(super) policy: Option<SchemaReconciliationPolicy>,
    pub(super) denial: Option<RelationalSchemaReconciliationWitnessDenial>,
    pub(super) posture: RelationalSchemaReconciliationWitnessPosture,
}

pub(super) fn derive_schema_reconciliation_truth(
    input: &RelationalSchemaReconciliationWitnessRowInput,
) -> DerivedSchemaReconciliationTruth {
    let denial = retained_schema_reconciliation_denial(input);
    let posture = if denial.is_some() {
        RelationalSchemaReconciliationWitnessPosture::Denied
    } else {
        RelationalSchemaReconciliationWitnessPosture::Reconciled
    };
    let proposed_transition = proposed_schema_transition(input);
    let classification = classify_schema_transition(
        proposed_transition.clone(),
        retained_schema_reconciliation_admission_policy(input),
    )
    .reconciliation;
    let policy = if posture == RelationalSchemaReconciliationWitnessPosture::Denied {
        None
    } else {
        let admitted_policy = retained_schema_reconciliation_admission_policy(input)
            .expect(/* Proof: These private reconciliation rows are constructed only after schema-policy admission and continuity validation; the retained input and proposed transition are unchanged. */ "admitted schema reconciliation rows must declare a schema policy");
        let validated = validate_schema_transition(proposed_transition, Some(admitted_policy))
            .expect(/* Proof: These private reconciliation rows are constructed only after schema-policy admission and continuity validation; the retained input and proposed transition are unchanged. */ "admitted schema reconciliation rows must validate through schema continuity");
        Some(
            lower_schema_transition(
                validated,
                Some(admitted_policy),
                DescriptorSemanticsVersion::default(),
                DescriptorCanonicalBasisVersion::default(),
            )
            .reconciliation_descriptor
            .policy,
        )
    };
    DerivedSchemaReconciliationTruth {
        classification,
        policy,
        denial,
        posture,
    }
}

fn retained_schema_reconciliation_denial(
    input: &RelationalSchemaReconciliationWitnessRowInput,
) -> Option<RelationalSchemaReconciliationWitnessDenial> {
    if input.relation_endpoint_divergence
        || matches!(
            (&input.basis.source_kind_id, &input.basis.target_kind_id),
            (Some(source_kind_id), Some(target_kind_id)) if source_kind_id != target_kind_id
        )
    {
        Some(RelationalSchemaReconciliationWitnessDenial::StructuralIncompatible)
    } else {
        match input.decision_boundary {
            MergePolicyDecisionBoundary::AutoResolved => None,
            MergePolicyDecisionBoundary::RequiresManualResolution {
                class: MergeManualResolutionClass::UnvalidatedSchemaCorrespondence,
            } => Some(RelationalSchemaReconciliationWitnessDenial::UnvalidatedSchemaCorrespondence),
            MergePolicyDecisionBoundary::RequiresManualResolution { .. } => {
                Some(RelationalSchemaReconciliationWitnessDenial::ManualResolutionRequired)
            }
            MergePolicyDecisionBoundary::Reject { .. } => {
                Some(RelationalSchemaReconciliationWitnessDenial::PolicyRejected)
            }
        }
    }
}

fn retained_schema_reconciliation_admission_policy(
    input: &RelationalSchemaReconciliationWitnessRowInput,
) -> Option<SchemaReconciliationPolicy> {
    if input.target_only_aspect_count > 0 {
        Some(SchemaReconciliationPolicy::PreserveTargetContract)
    } else if matches!(
        input.decision_boundary,
        MergePolicyDecisionBoundary::AutoResolved
    ) {
        Some(SchemaReconciliationPolicy::PreserveInformation)
    } else {
        None
    }
}

fn proposed_schema_transition(
    input: &RelationalSchemaReconciliationWitnessRowInput,
) -> ProposedSchemaTransition {
    let (source_schema_id, source_schema_version_id) = schema_transition_source_basis(input);
    let (target_schema_id, target_schema_version_id) = schema_transition_target_basis(input);
    ProposedSchemaTransition {
        source_schema_id,
        source_schema_version_id,
        target_schema_id: target_schema_id.clone(),
        target_schema_version_id,
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                target_schema_id,
                target_schema_version_id,
                input.basis.source_kind_id.or(input.basis.target_kind_id),
                "merge_schema_reconciliation",
            ),
            schema_transition_strata(input),
            SchemaPublicationImpact::ObservableSurfaceChanged,
            schema_transition_subscriber_impact(input),
            schema_transition_historical_interpretation(input),
            schema_transition_detail(input),
        )],
    }
}

fn schema_transition_source_basis(
    input: &RelationalSchemaReconciliationWitnessRowInput,
) -> (SchemaId, SchemaVersionId) {
    (
        input
            .basis
            .source_schema_id
            .clone()
            .or_else(|| input.basis.target_schema_id.clone())
            .unwrap_or_else(|| {
                SchemaId(format!(
                    "merge-schema-source-{}",
                    input.basis.registry_digest
                ))
            }),
        input
            .basis
            .source_schema_version_id
            .or(input.basis.target_schema_version_id)
            .unwrap_or(SchemaVersionId(0)),
    )
}

fn schema_transition_target_basis(
    input: &RelationalSchemaReconciliationWitnessRowInput,
) -> (SchemaId, SchemaVersionId) {
    (
        input
            .basis
            .target_schema_id
            .clone()
            .or_else(|| input.basis.source_schema_id.clone())
            .unwrap_or_else(|| {
                SchemaId(format!(
                    "merge-schema-target-{}",
                    input.basis.registry_digest
                ))
            }),
        input
            .basis
            .target_schema_version_id
            .or(input.basis.source_schema_version_id)
            .unwrap_or(SchemaVersionId(0)),
    )
}

fn schema_transition_detail(
    input: &RelationalSchemaReconciliationWitnessRowInput,
) -> SchemaDiffDetail {
    if input.relation_endpoint_divergence
        || matches!(
            (&input.basis.source_kind_id, &input.basis.target_kind_id),
            (Some(source_kind_id), Some(target_kind_id)) if source_kind_id != target_kind_id
        )
    {
        SchemaDiffDetail::FreeText {
            detail: "merge schema reconciliation detected structural continuity denial".into(),
            declared_intent: FreeFormSchemaDiffIntent::StructuralContinuityDenied,
        }
    } else if matches!(
        input.decision_boundary,
        MergePolicyDecisionBoundary::Reject { .. }
    ) || input.divergent_aspect_count > 0
    {
        SchemaDiffDetail::TypeChanged {
            field: synthetic_field_key(),
            from_type: "merge-source-shape".into(),
            to_type: "merge-target-shape".into(),
        }
    } else if input.target_only_aspect_count > 0
        || matches!(
            input.decision_boundary,
            MergePolicyDecisionBoundary::RequiresManualResolution {
                class: MergeManualResolutionClass::UnvalidatedSchemaCorrespondence
            }
        )
    {
        SchemaDiffDetail::RemovedField {
            field: synthetic_field_key(),
        }
    } else {
        SchemaDiffDetail::AddedField {
            field: synthetic_field_key(),
            required: false,
            default_expression: None,
        }
    }
}

fn schema_transition_strata(
    input: &RelationalSchemaReconciliationWitnessRowInput,
) -> Vec<SchemaStratum> {
    if input.relation_endpoint_divergence {
        vec![
            SchemaStratum::BehavioralSemantics,
            SchemaStratum::PublicationContract,
        ]
    } else if input.divergent_aspect_count > 0
        || matches!(
            input.decision_boundary,
            MergePolicyDecisionBoundary::Reject { .. }
        )
    {
        vec![
            SchemaStratum::ValueDomain,
            SchemaStratum::PublicationContract,
        ]
    } else {
        vec![
            SchemaStratum::StructuralShape,
            SchemaStratum::PublicationContract,
        ]
    }
}

fn schema_transition_subscriber_impact(
    input: &RelationalSchemaReconciliationWitnessRowInput,
) -> SchemaSubscriberImpact {
    if matches!(
        input.decision_boundary,
        MergePolicyDecisionBoundary::AutoResolved
    ) && input.target_only_aspect_count == 0
        && input.divergent_aspect_count == 0
        && !input.relation_endpoint_divergence
    {
        SchemaSubscriberImpact::ConsumableSurfaceChanged
    } else {
        SchemaSubscriberImpact::RenegotiationRequired
    }
}

fn schema_transition_historical_interpretation(
    input: &RelationalSchemaReconciliationWitnessRowInput,
) -> HistoricalInterpretationSensitivity {
    if input.relation_endpoint_divergence {
        HistoricalInterpretationSensitivity::SensitiveToLegalityMeaning
    } else if input.divergent_aspect_count > 0
        || matches!(
            input.decision_boundary,
            MergePolicyDecisionBoundary::Reject { .. }
        )
    {
        HistoricalInterpretationSensitivity::SensitiveToValueMeaning
    } else {
        HistoricalInterpretationSensitivity::SensitiveToPublicationMeaning
    }
}

fn synthetic_field_key() -> FieldKey {
    FieldKey::new("merge_schema").expect(/* Proof: This constructor receives a fixed nonempty literal in the required grammar and version/domain; it has no runtime-dependent input. */ "valid synthetic schema field")
}
