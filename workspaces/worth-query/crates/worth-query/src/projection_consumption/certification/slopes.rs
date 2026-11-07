use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::identity::{
    compose_certification_counter_snapshot_digest, compose_slope_digest,
    compose_support_matrix_support_width_digest,
};
use crate::projection_consumption::{
    discover_projection_consumption_support, ProjectionConsumptionSource,
};

use super::fixtures::{certification_row_set, control_row_set_lifecycle};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionCertificationCounterSnapshot {
    declared_fact_family_count: usize,
    admitted_fact_family_count: usize,
    extracted_fact_count: usize,
    source_row_width_consumed: usize,
    source_evidence_lookup_width: usize,
    authority_reopen_count: usize,
    digest: String,
}

impl ProjectionConsumptionCertificationCounterSnapshot {
    pub fn declared_fact_family_count(&self) -> usize {
        self.declared_fact_family_count
    }

    pub fn admitted_fact_family_count(&self) -> usize {
        self.admitted_fact_family_count
    }

    pub fn extracted_fact_count(&self) -> usize {
        self.extracted_fact_count
    }

    pub fn source_row_width_consumed(&self) -> usize {
        self.source_row_width_consumed
    }

    pub fn source_evidence_lookup_width(&self) -> usize {
        self.source_evidence_lookup_width
    }

    pub fn authority_reopen_count(&self) -> usize {
        self.authority_reopen_count
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionSlopeReport {
    counter_snapshot: ProjectionConsumptionCertificationCounterSnapshot,
    declaration_slope_digest: String,
    eligibility_slope_digest: String,
    contract_binding_slope_digest: String,
    fact_extraction_slope_digest: String,
    receipt_materialization_slope_digest: String,
    envelope_materialization_slope_digest: String,
    support_lookup_slope_digest: String,
}

impl ProjectionConsumptionSlopeReport {
    pub fn counter_snapshot(&self) -> &ProjectionConsumptionCertificationCounterSnapshot {
        &self.counter_snapshot
    }

    pub fn declaration_slope_digest(&self) -> &str {
        &self.declaration_slope_digest
    }

    pub fn eligibility_slope_digest(&self) -> &str {
        &self.eligibility_slope_digest
    }

    pub fn contract_binding_slope_digest(&self) -> &str {
        &self.contract_binding_slope_digest
    }

    pub fn fact_extraction_slope_digest(&self) -> &str {
        &self.fact_extraction_slope_digest
    }

    pub fn receipt_materialization_slope_digest(&self) -> &str {
        &self.receipt_materialization_slope_digest
    }

    pub fn envelope_materialization_slope_digest(&self) -> &str {
        &self.envelope_materialization_slope_digest
    }

    pub fn support_lookup_slope_digest(&self) -> &str {
        &self.support_lookup_slope_digest
    }
}

pub fn projection_consumption_slope_report(
) -> Result<ProjectionConsumptionSlopeReport, IdentityDerivationDenial> {
    let control = control_row_set_lifecycle(2)?;
    let counters = control.facts().counters();
    let counter_snapshot = ProjectionConsumptionCertificationCounterSnapshot {
        declared_fact_family_count: counters.declared_fact_family_count(),
        admitted_fact_family_count: counters.admitted_fact_family_count(),
        extracted_fact_count: counters.extracted_fact_count(),
        source_row_width_consumed: counters.source_row_width_consumed(),
        source_evidence_lookup_width: counters.source_evidence_lookup_width(),
        authority_reopen_count: counters.authority_reopen_count(),
        digest: compose_certification_counter_snapshot_digest(
            counters.declared_fact_family_count(),
            counters.admitted_fact_family_count(),
            counters.extracted_fact_count(),
            counters.source_row_width_consumed(),
            counters.source_evidence_lookup_width(),
            counters.authority_reopen_count(),
        )?,
    };
    Ok(ProjectionConsumptionSlopeReport {
        counter_snapshot,
        declaration_slope_digest: compose_slope_digest(
            "declaration",
            (1..=3).map(|scale| {
                let lifecycle = control_row_set_lifecycle(scale);
                (
                    scale,
                    vec![
                        (
                            "declared",
                            lifecycle
                                .facts()
                                .counters()
                                .declared_fact_family_count()
                                .to_string(),
                        ),
                        (
                            "declaration",
                            lifecycle.declaration().declaration_digest().to_string(),
                        ),
                    ],
                )
            }),
        )?,
        eligibility_slope_digest: compose_slope_digest(
            "eligibility",
            (1..=3).map(|scale| {
                let lifecycle = control_row_set_lifecycle(scale);
                (
                    scale,
                    vec![
                        (
                            "admitted",
                            lifecycle
                                .facts()
                                .counters()
                                .admitted_fact_family_count()
                                .to_string(),
                        ),
                        (
                            "eligibility",
                            lifecycle.contract().eligibility_digest().to_string(),
                        ),
                    ],
                )
            }),
        )?,
        contract_binding_slope_digest: compose_slope_digest(
            "contract_binding",
            (1..=3).map(|scale| {
                let lifecycle = control_row_set_lifecycle(scale);
                (
                    scale,
                    vec![
                        (
                            "contract",
                            lifecycle.contract().contract_digest().to_string(),
                        ),
                        (
                            "query",
                            lifecycle
                                .contract()
                                .query_digest()
                                .unwrap_or("none")
                                .to_string(),
                        ),
                    ],
                )
            }),
        )?,
        fact_extraction_slope_digest: compose_slope_digest(
            "fact_extraction",
            (1..=3).map(|scale| {
                let lifecycle = control_row_set_lifecycle(scale);
                let counters = lifecycle.facts().counters();
                (
                    scale,
                    vec![
                        ("extracted", counters.extracted_fact_count().to_string()),
                        (
                            "row_width",
                            counters.source_row_width_consumed().to_string(),
                        ),
                    ],
                )
            }),
        )?,
        receipt_materialization_slope_digest: compose_slope_digest(
            "receipt_materialization",
            (1..=3).map(|scale| {
                let lifecycle = control_row_set_lifecycle(scale);
                (
                    scale,
                    vec![
                        ("receipt", lifecycle.receipt().receipt_digest().to_string()),
                        (
                            "extracted",
                            lifecycle.receipt().extracted_fact_count().to_string(),
                        ),
                    ],
                )
            }),
        )?,
        envelope_materialization_slope_digest: compose_slope_digest(
            "envelope_materialization",
            (1..=3).map(|scale| {
                let lifecycle = control_row_set_lifecycle(scale);
                (
                    scale,
                    vec![
                        (
                            "envelope",
                            lifecycle.envelope().envelope_digest().to_string(),
                        ),
                        (
                            "authority_reopen",
                            lifecycle.envelope().authority_reopen_count().to_string(),
                        ),
                    ],
                )
            }),
        )?,
        support_lookup_slope_digest: compose_slope_digest(
            "support_lookup",
            (1..=3).map(|scale| {
                let row_set = certification_row_set(scale);
                let support = discover_projection_consumption_support(
                    &ProjectionConsumptionSource::from_relational_row_set(&row_set),
                );
                (
                    scale,
                    vec![
                        ("support_width", support.rows().len().to_string()),
                        (
                            "support_digest",
                            compose_support_matrix_support_width_digest(
                                support.rows().iter().map(|row| row.support_digest()),
                            ),
                        ),
                    ],
                )
            })?,
        )?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slope_report_emits_counter_snapshot_and_all_phase_slope_digests(
    ) -> Result<(), IdentityDerivationDenial> {
        let report = projection_consumption_slope_report()?;
        assert_eq!(report.counter_snapshot().authority_reopen_count(), 0);
        assert!(!report.counter_snapshot().digest().is_empty());
        assert!(!report.declaration_slope_digest().is_empty());
        assert!(!report.eligibility_slope_digest().is_empty());
        assert!(!report.contract_binding_slope_digest().is_empty());
        assert!(!report.fact_extraction_slope_digest().is_empty());
        assert!(!report.receipt_materialization_slope_digest().is_empty());
        assert!(!report.envelope_materialization_slope_digest().is_empty());
        assert!(!report.support_lookup_slope_digest().is_empty());

        Ok(())
    }
}
