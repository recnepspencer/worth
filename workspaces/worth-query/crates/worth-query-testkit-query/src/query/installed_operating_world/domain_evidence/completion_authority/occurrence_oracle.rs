pub use sha2::{Digest, Sha256};
pub use worth_foundational::facade::{
    canonical_basis_sequence_material, prepare_canonical_basis_sequence, CanonicalBasisDomain,
    CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue,
    CanonicalizationRuleVersion,
};
pub use worth_query::facade::domain;
pub use worth_query::facade::installed::operation::WorthQueryExecutionResourceAttemptEvidence;

pub fn expected_direct_occurrence(
    binding: &domain::WorthQueryDomainEvidenceBinding,
    resources: &WorthQueryExecutionResourceAttemptEvidence,
    graph_receipts: &[String],
) -> String {
    expected_occurrence(binding, resources, None, None, graph_receipts)
}

pub fn expected_workflow_occurrence(
    binding: &domain::WorthQueryDomainEvidenceBinding,
    resources: &WorthQueryExecutionResourceAttemptEvidence,
    graph_receipts: &[String],
) -> String {
    expected_occurrence(
        binding,
        resources,
        binding.run_identity(),
        binding.stage_identity(),
        graph_receipts,
    )
}

pub fn expected_occurrence(
    binding: &domain::WorthQueryDomainEvidenceBinding,
    resources: &WorthQueryExecutionResourceAttemptEvidence,
    run: Option<&str>,
    stage: Option<&str>,
    graph_receipts: &[String],
) -> String {
    hash_parts(&[
        "worth_query_ordinary_domain_evidence_occurrence_v1".into(),
        format!("operation:{}", binding.operation_identity()),
        format!("binding:{}", binding.binding_identity()),
        format!("basis:{}", binding.basis_identity()),
        format!("snapshot:{}", binding.execution_snapshot_identity()),
        format!("run:{}", run.unwrap_or("not-required")),
        format!("stage:{}", stage.unwrap_or("not-required")),
        format!("output:{}", binding.output_occurrence_identity()),
        format!("provider-session:{}", resources.provider_session_identity()),
        format!(
            "provider-session-attempt:{}",
            resources.provider_session_attempt_identity()
        ),
        format!(
            "graph-receipts:{}",
            graph_receipt_material(graph_receipts.iter().cloned())
        ),
    ])
}

pub fn graph_receipt_material(receipts: impl IntoIterator<Item = String>) -> String {
    let receipts = receipts.into_iter().collect::<Vec<_>>();
    let entries = if receipts.is_empty() {
        vec![(
            "ordinary.domain-evidence.graph-receipt.empty".to_owned(),
            "explicitly-empty".to_owned(),
        )]
    } else {
        receipts
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                (
                    format!("ordinary.domain-evidence.graph-receipt.{index}"),
                    value,
                )
            })
            .collect()
    };
    let entries = entries.into_iter().map(|(locus, value)| {
        CanonicalBasisEntry::new(
            CanonicalBasisDomain::Future("query-operation-identity"),
            CanonicalBasisLocus::Named(locus.into()),
            CanonicalBasisEntryKind::Value,
            CanonicalBasisValue::ExactText(value.into()),
        )
    });
    let ready = prepare_canonical_basis_sequence(
        CanonicalizationRuleVersion::new("query-operation-identity-v1").unwrap(),
        CanonicalBasisDomain::Future("query-operation-identity"),
        entries,
    )
    .into_result()
    .unwrap();
    canonical_basis_sequence_material(ready.payload())
}

pub fn hash_parts(parts: &[String]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update([0x1f]);
    }
    format!("{:x}", hasher.finalize())
}
