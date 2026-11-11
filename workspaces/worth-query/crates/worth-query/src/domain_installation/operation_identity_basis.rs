use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_foundational::facade::{
    canonical_basis_sequence_material, prepare_canonical_basis_sequence, CanonicalBasisDomain,
    CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue,
    CanonicalizationRuleVersion,
};

use crate::domain_installation::{
    WorthQueryGraphProviderCallKind, WorthQueryOperationExecutionWarning,
    WorthQueryOperationResultState, WorthQueryWorkflowRunCounters, WorthQueryWorkflowSemanticValue,
    WorthQueryWorkflowStageWarning,
};
use crate::identity_evolution::InstalledIdentityEvolutionOutcome;

pub(crate) fn canonical_operation_identity(
    role: &'static str,
    entries: Vec<(&str, String)>,
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
            crate::evidence_identity::WorthQueryEvidenceScope::InstalledDomainExecution,
        )
        .field_shape(
            crate::evidence_identity::WorthQueryEvidenceTag::new("operation_evidence_role"),
            role,
        )
        .field_value(
            crate::evidence_identity::WorthQueryEvidenceTag::new("canonical_operation_basis"),
            canonical_operation_material(entries),
        )
        .seal()?
        .terminal_projection_for_reporting()
        .to_owned(),
    )
}

pub(crate) fn canonical_operation_material(entries: Vec<(&str, String)>) -> String {
    canonical_owned_operation_material(
        entries
            .into_iter()
            .map(|(locus, value)| (locus.to_owned(), value)),
    )
}

pub(crate) fn canonical_indexed_operation_material(
    locus: &str,
    values: impl IntoIterator<Item = String>,
) -> String {
    let values = values.into_iter().collect::<Vec<_>>();
    if values.is_empty() {
        return canonical_owned_operation_material([(
            format!("{locus}.empty"),
            "explicitly-empty".into(),
        )]);
    }
    canonical_owned_operation_material(
        values
            .into_iter()
            .enumerate()
            .map(|(index, value)| (format!("{locus}.{index}"), value)),
    )
}

fn canonical_owned_operation_material(
    entries: impl IntoIterator<Item = (String, String)>,
) -> String {
    let entries = entries.into_iter().map(|(locus, value)| {
        CanonicalBasisEntry::new(
            CanonicalBasisDomain::Future("query-operation-identity"),
            CanonicalBasisLocus::Named(locus.into()),
            CanonicalBasisEntryKind::Value,
            CanonicalBasisValue::ExactText(value.into()),
        )
    });
    let ready = prepare_canonical_basis_sequence(
        CanonicalizationRuleVersion::new("query-operation-identity-v1")
            .expect("static canonicalization rule version is valid"),
        CanonicalBasisDomain::Future("query-operation-identity"),
        entries,
    )
    .into_result()
    .expect("Query operation identity basis is structurally canonical");
    canonical_basis_sequence_material(ready.payload())
}

pub(crate) fn workflow_semantic_value_material(value: &WorthQueryWorkflowSemanticValue) -> String {
    match value {
        WorthQueryWorkflowSemanticValue::NotRequired => "not-required".into(),
        WorthQueryWorkflowSemanticValue::Bool(value) => format!("bool:{value}"),
        WorthQueryWorkflowSemanticValue::I64(value) => format!("i64:{value}"),
        WorthQueryWorkflowSemanticValue::U64(value) => format!("u64:{value}"),
        WorthQueryWorkflowSemanticValue::Text(value) => {
            canonical_operation_material(vec![("workflow.text", value.clone())])
        }
        WorthQueryWorkflowSemanticValue::EntityIdentity(value) => {
            canonical_operation_material(vec![("workflow.entity", value.clone())])
        }
        WorthQueryWorkflowSemanticValue::Projection {
            canonical_query_identity,
            rows,
        } => {
            let row_material = rows.iter().map(|row| {
                canonical_operation_material(vec![
                    (
                        "projection.row.identity",
                        row.identity().evidence_identity().as_str().to_owned(),
                    ),
                    (
                        "projection.row.values",
                        canonical_indexed_operation_material(
                            "projection.row.value",
                            row.terminal_result_digest_parts(),
                        ),
                    ),
                ])
            });
            canonical_operation_material(vec![
                ("projection.query", canonical_query_identity.clone()),
                (
                    "projection.rows",
                    canonical_indexed_operation_material("projection.row", row_material),
                ),
            ])
        }
        WorthQueryWorkflowSemanticValue::InstalledArtifact(meaning) => meaning.canonical_part(),
    }
}

pub(crate) fn workflow_warning_material(warning: &WorthQueryWorkflowStageWarning) -> String {
    match warning {
        WorthQueryWorkflowStageWarning::Advisory(detail) => {
            canonical_operation_material(vec![("workflow.warning.advisory", detail.clone())])
        }
        WorthQueryWorkflowStageWarning::Partial(detail) => {
            canonical_operation_material(vec![("workflow.warning.partial", detail.clone())])
        }
    }
}

pub(crate) fn operation_warning_material(warning: &WorthQueryOperationExecutionWarning) -> String {
    match warning {
        WorthQueryOperationExecutionWarning::Advisory(detail) => {
            canonical_operation_material(vec![("operation.warning.advisory", detail.clone())])
        }
        WorthQueryOperationExecutionWarning::Partial(detail) => {
            canonical_operation_material(vec![("operation.warning.partial", detail.clone())])
        }
    }
}

pub(crate) fn operation_result_state_material(
    state: Option<WorthQueryOperationResultState>,
) -> &'static str {
    match state {
        None => "not-declared",
        Some(WorthQueryOperationResultState::Ready) => "ready",
        Some(WorthQueryOperationResultState::Advisory) => "advisory",
        Some(WorthQueryOperationResultState::Pending) => "pending",
        Some(WorthQueryOperationResultState::Partial) => "partial",
        Some(WorthQueryOperationResultState::Violation) => "violation",
    }
}

pub(crate) fn graph_call_kind_material(kind: WorthQueryGraphProviderCallKind) -> &'static str {
    kind.as_str()
}

pub(crate) fn workflow_counter_material(counters: WorthQueryWorkflowRunCounters) -> String {
    canonical_operation_material(vec![
        (
            "counter.runtime_authority_checks",
            counters.runtime_authority_checks.to_string(),
        ),
        (
            "counter.stage_index_lookups",
            counters.stage_index_lookups.to_string(),
        ),
        (
            "counter.stage_admission_checks",
            counters.stage_admission_checks.to_string(),
        ),
        (
            "counter.predecessor_checks",
            counters.predecessor_checks.to_string(),
        ),
        (
            "counter.predecessor_receipt_lookups",
            counters.predecessor_receipt_lookups.to_string(),
        ),
        (
            "counter.required_capability_checks",
            counters.required_capability_checks.to_string(),
        ),
        (
            "counter.required_domain_checks",
            counters.required_domain_checks.to_string(),
        ),
        (
            "counter.graph_read_contacts",
            counters.graph_read_contacts.to_string(),
        ),
        (
            "counter.touch_effect_contacts",
            counters.touch_effect_contacts.to_string(),
        ),
        (
            "counter.effect_receipt_checks",
            counters.effect_receipt_checks.to_string(),
        ),
        (
            "counter.commit_admission_contacts",
            counters.commit_admission_contacts.to_string(),
        ),
        (
            "counter.invariant_checks",
            counters.invariant_checks.to_string(),
        ),
        (
            "counter.parallel_admission_checks",
            counters.parallel_admission_checks.to_string(),
        ),
        (
            "counter.stage_executor_contacts",
            counters.stage_executor_contacts.to_string(),
        ),
        (
            "counter.output_contract_checks",
            counters.output_contract_checks.to_string(),
        ),
        (
            "counter.terminal_contract_checks",
            counters.terminal_contract_checks.to_string(),
        ),
        (
            "counter.consumption_contacts",
            counters.consumption_contacts.to_string(),
        ),
        (
            "counter.unrelated_run_scans",
            counters.unrelated_run_scans.to_string(),
        ),
        (
            "counter.conditional_request_admission_checks",
            counters.conditional_request_admission_checks.to_string(),
        ),
        (
            "counter.conditional_contract_lookups",
            counters.conditional_contract_lookups.to_string(),
        ),
        (
            "counter.conditional_dependency_observation_reads",
            counters
                .conditional_dependency_observation_reads
                .to_string(),
        ),
        (
            "counter.conditional_dependency_checks",
            counters.conditional_dependency_checks.to_string(),
        ),
        (
            "counter.conditional_semantic_reads",
            counters.conditional_semantic_reads.to_string(),
        ),
        (
            "counter.conditional_condition_checks",
            counters.conditional_condition_checks.to_string(),
        ),
        (
            "counter.conditional_condition_deferrals",
            counters.conditional_condition_deferrals.to_string(),
        ),
        (
            "counter.conditional_temporal_deferrals",
            counters.conditional_temporal_deferrals.to_string(),
        ),
        (
            "counter.conditional_on_demand_deferrals",
            counters.conditional_on_demand_deferrals.to_string(),
        ),
        (
            "counter.conditional_comparator_checks",
            counters.conditional_comparator_checks.to_string(),
        ),
        (
            "counter.conditional_compute_contacts",
            counters.conditional_compute_contacts.to_string(),
        ),
        (
            "counter.conditional_output_version_reads",
            counters.conditional_output_version_reads.to_string(),
        ),
        (
            "counter.conditional_runtime_dependency_edges_captured",
            counters
                .conditional_runtime_dependency_edges_captured
                .to_string(),
        ),
        (
            "counter.conditional_application_contacts",
            counters.conditional_application_contacts.to_string(),
        ),
        (
            "counter.conditional_semantic_classifications",
            counters.conditional_semantic_classifications.to_string(),
        ),
        (
            "counter.conditional_reverted_clean_outcomes",
            counters.conditional_reverted_clean_outcomes.to_string(),
        ),
        (
            "counter.conditional_semantic_changes",
            counters.conditional_semantic_changes.to_string(),
        ),
        (
            "counter.conditional_reuse_checks",
            counters.conditional_reuse_checks.to_string(),
        ),
        (
            "counter.conditional_decisions_delivered",
            counters.conditional_decisions_delivered.to_string(),
        ),
    ])
}

pub(crate) fn lineage_outcome_material(outcome: &InstalledIdentityEvolutionOutcome) -> String {
    let artifact = outcome.engine_artifact();
    canonical_operation_material(vec![
        ("lineage.query", artifact.query_digest().to_owned()),
        ("lineage.basis", artifact.basis_digest().to_owned()),
        ("lineage.family", artifact.family().as_str().to_owned()),
        ("lineage.path", artifact.lineage_digest().to_owned()),
        ("lineage.result", artifact.result_digest().to_owned()),
        (
            "lineage.outcome",
            artifact
                .result_bundle()
                .outcome_family()
                .as_str()
                .to_owned(),
        ),
        ("lineage.semantic", outcome.semantic_identity().to_owned()),
    ])
}
