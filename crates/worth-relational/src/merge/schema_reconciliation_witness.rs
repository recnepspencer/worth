use worth_relational_contracts::internal_port::merge::data::artifacts::schema_reconciliation_witness::RelationalSchemaReconciliationWitnessOwnerPort as _;
use worth_relational_contracts::internal_port::merge::data::artifacts::schema_reconciliation_witness_rows::RelationalSchemaReconciliationWitnessRowOwnerPort as _;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::merge::data::{
    MergeConflictClass, MergePolicyDecisionBoundary, MergeSchemaSnapshotDigestBasis,
    RelationalSchemaReconciliationBasisRow, RelationalSchemaReconciliationCorrespondenceLinkRow,
    RelationalSchemaReconciliationWitness, RelationalSchemaReconciliationWitnessRow,
    RelationalSchemaReconciliationWitnessRowInput, VisibleMergeRecord,
};

pub(crate) fn retained_schema_reconciliation_witness(
    plan: &crate::merge::data::LoweredMergePlan,
    schema_snapshot: &MergeSchemaSnapshotDigestBasis,
) -> RelationalSchemaReconciliationWitness {
    let source_records = plan
        .source_records
        .iter()
        .map(|record| (record.record_ref.clone(), record))
        .collect::<BTreeMap<_, _>>();
    let classifications = plan
        .classifications
        .iter()
        .map(|classification| (&classification.record, classification))
        .collect::<BTreeMap<_, _>>();
    let policy_records = plan
        .policy_records
        .iter()
        .map(|record| (&record.record, record))
        .collect::<BTreeMap<_, _>>();
    let links = plan
        .validated_schema_correspondences
        .iter()
        .map(|link| {
            (
                &link.source_record,
                RelationalSchemaReconciliationCorrespondenceLinkRow {
                    scope: link.scope.clone(),
                    basis: link.basis.clone(),
                    source_record: link.source_record.clone(),
                    target_record: link.target_record.clone(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let rows = plan
        .lowered_records
        .iter()
        .filter_map(|record| {
            let classification = classifications.get(&record.record)?;
            let policy_record = policy_records.get(&record.record)?;
            let source_record = source_records.get(&record.record)?;
            schema_witness_row(
                source_record,
                classification,
                policy_record.proof_boundary.decision_boundary,
                links.get(&record.record).cloned(),
                schema_snapshot,
            )
        })
        .collect::<Vec<_>>();

    RelationalSchemaReconciliationWitness::retained(
        plan.request.request_digest().to_string(),
        plan.basis.basis_digest(),
        Arc::from(rows),
    )
}

fn schema_witness_row(
    source_record: &VisibleMergeRecord,
    classification: &crate::merge::data::MergeConflictClassification,
    decision_boundary: MergePolicyDecisionBoundary,
    correspondence_linkage: Option<RelationalSchemaReconciliationCorrespondenceLinkRow>,
    schema_snapshot: &MergeSchemaSnapshotDigestBasis,
) -> Option<RelationalSchemaReconciliationWitnessRow> {
    let schema_relevant = matches!(
        classification.class,
        MergeConflictClass::SchemaDeclaredCorrespondence
            | MergeConflictClass::RelationEndpointDivergence
    ) || matches!(
        decision_boundary,
        MergePolicyDecisionBoundary::RequiresManualResolution {
            class: crate::merge::data::MergeManualResolutionClass::UnvalidatedSchemaCorrespondence
        }
    );
    if !schema_relevant {
        return None;
    }
    let source_schema = kind_schema_basis(source_record.source_kind_id, schema_snapshot);
    let target_schema = kind_schema_basis(source_record.target_kind_id, schema_snapshot);
    let (
        source_only_aspect_count,
        target_only_aspect_count,
        divergent_aspect_count,
        unavailable_aspect_count,
    ) = classification
        .aspect_evidence
        .iter()
        .fold((0, 0, 0, 0), |counts, aspect| match aspect.comparison {
            crate::merge::data::AspectComparisonState::SourceOnly => {
                (counts.0 + 1, counts.1, counts.2, counts.3)
            }
            crate::merge::data::AspectComparisonState::TargetOnly => {
                (counts.0, counts.1 + 1, counts.2, counts.3)
            }
            crate::merge::data::AspectComparisonState::Divergent => {
                (counts.0, counts.1, counts.2 + 1, counts.3)
            }
            crate::merge::data::AspectComparisonState::Unavailable => {
                (counts.0, counts.1, counts.2, counts.3 + 1)
            }
            crate::merge::data::AspectComparisonState::Equal => counts,
        });
    Some(RelationalSchemaReconciliationWitnessRow::retained(
        RelationalSchemaReconciliationWitnessRowInput {
            record: classification.record.clone(),
            target_record: classification.target_record.clone(),
            basis: RelationalSchemaReconciliationBasisRow {
                source_kind_id: source_record.source_kind_id,
                target_kind_id: source_record.target_kind_id,
                source_schema_id: source_schema.as_ref().map(|basis| basis.0.clone()),
                source_schema_version_id: source_schema.as_ref().map(|basis| basis.1),
                target_schema_id: target_schema.as_ref().map(|basis| basis.0.clone()),
                target_schema_version_id: target_schema.as_ref().map(|basis| basis.1),
                registry_digest: schema_snapshot.registry_digest.clone(),
            },
            source_only_aspect_count,
            target_only_aspect_count,
            divergent_aspect_count,
            unavailable_aspect_count,
            decision_boundary,
            relation_endpoint_divergence: classification.class
                == MergeConflictClass::RelationEndpointDivergence,
            correspondence_linkage,
        },
    ))
}

fn kind_schema_basis(
    kind_id: Option<crate::identity::data::KindId>,
    schema_snapshot: &MergeSchemaSnapshotDigestBasis,
) -> Option<(
    crate::schema::data::SchemaId,
    crate::schema::data::SchemaVersionId,
)> {
    let kind_id = kind_id?;
    schema_snapshot
        .touched_kinds
        .iter()
        .find(|snapshot| snapshot.kind_id == kind_id)
        .map(|snapshot| (snapshot.schema_id.clone(), snapshot.schema_version_id))
}
