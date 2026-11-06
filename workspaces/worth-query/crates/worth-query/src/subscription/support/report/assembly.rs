use super::super::super::evidence_identities::{support_report_identity, typed_identity_drift};
use super::super::super::evidence_projection::subscription_evidence_projection;
use super::super::matrix::QuerySubscriptionSupportMatrix;
use super::super::subject::{QuerySubscriptionSupportEvidence, QuerySubscriptionSupportSubject};
use super::counters::counters_for_posture;
use super::errors::{
    QuerySubscriptionSupportReportDenialKind, QuerySubscriptionSupportReportError,
};
use super::lookup::{SupportLookupReceipt, SupportResolutionPosture};
use super::outcome::QuerySubscriptionSupportReport;

pub fn report_query_subscription_support(
    subject: QuerySubscriptionSupportSubject,
    evidence: QuerySubscriptionSupportEvidence,
) -> Result<
    (QuerySubscriptionSupportReport, SupportLookupReceipt),
    QuerySubscriptionSupportReportError,
> {
    validate_subject_matches_evidence(&subject, &evidence)?;

    let support_matrix = QuerySubscriptionSupportMatrix::for_family(
        evidence.family(),
        evidence.support_profile(),
        &subject,
    )?;
    let support_row = support_matrix
        .row_for_class(*subject.support_class())
        .expect("every support class must exist in the family support matrix");

    let counters = counters_for_posture(support_row.posture());
    let lookup_receipt = SupportLookupReceipt::new(
        evidence.family(),
        *subject.support_class(),
        SupportResolutionPosture::IndexedFamilyLookup,
        1,
        support_matrix.rows().len().saturating_sub(1),
    )?;
    let counter_snapshot_identity = counters.evidence_identity()?;
    let report_identity = support_report_identity(
        subject.subject_identity(),
        support_row.posture().as_str(),
        support_matrix.matrix_identity(),
        lookup_receipt.lookup_receipt_identity(),
        &counter_snapshot_identity,
    )?;

    Ok((
        QuerySubscriptionSupportReport::new(
            subject.clone(),
            *support_row.posture(),
            support_matrix,
            subject.source_identity().clone(),
            counter_snapshot_identity,
            lookup_receipt.lookup_receipt_identity().clone(),
            report_identity,
            counters,
        ),
        lookup_receipt,
    ))
}

fn validate_subject_matches_evidence(
    subject: &QuerySubscriptionSupportSubject,
    evidence: &QuerySubscriptionSupportEvidence,
) -> Result<(), QuerySubscriptionSupportReportError> {
    if typed_identity_drift(
        subject.declaration_identity(),
        evidence.declaration_identity(),
    ) {
        return Err(QuerySubscriptionSupportReportError::new(
            QuerySubscriptionSupportReportDenialKind::DeclarationSourceMismatch,
            "subscription support reporting requires a subject built from the same declaration artifact",
            &[
                format!(
                    "subject_declaration:{}",
                    subject.declaration_projection().label()
                ),
                format!(
                    "evidence_declaration:{}",
                    subscription_evidence_projection(evidence.declaration_identity()).label()
                ),
            ],
        ));
    }

    if subject.family() != evidence.family() {
        return Err(QuerySubscriptionSupportReportError::new(
            QuerySubscriptionSupportReportDenialKind::FamilySourceMismatch,
            "subscription support reporting requires subject and evidence to preserve the same query subscription family",
            &[
                format!("subject_family:{}", subject.family().as_str()),
                format!("evidence_family:{}", evidence.family().as_str()),
            ],
        ));
    }

    match (subject.admission_identity(), evidence.admission_identity()) {
        (Some(subject_admission_identity), Some(evidence_admission_identity)) => {
            if typed_identity_drift(subject_admission_identity, evidence_admission_identity) {
                return Err(QuerySubscriptionSupportReportError::new(
                    QuerySubscriptionSupportReportDenialKind::AdmissionSourceMismatch,
                    "subscription support reporting requires a subject bound to the same admission artifact",
                    &[
                        format!(
                            "subject_admission:{}",
                            subject_admission_identity.as_str()
                        ),
                        format!(
                            "evidence_admission:{}",
                            evidence_admission_identity.as_str()
                        ),
                    ],
                ));
            }
        }
        (Some(subject_admission_identity), None) => {
            return Err(QuerySubscriptionSupportReportError::new(
                QuerySubscriptionSupportReportDenialKind::AdmissionEvidenceRequired,
                "subscription support reporting requires admission evidence for activation, lifecycle, continuation, and preview subjects",
                &[
                    format!("subject_support_class:{}", subject.support_class().as_str()),
                    format!("subject_admission:{}", subject_admission_identity.as_str()),
                    format!("evidence_source:{}", evidence.source_projection().label()),
                ],
            ));
        }
        (None, Some(_)) if subject.support_class().requires_admission_evidence() => {
            return Err(QuerySubscriptionSupportReportError::new(
                QuerySubscriptionSupportReportDenialKind::AdmissionEvidenceRequired,
                "subscription support reporting requires admission-bound subjects for activation, lifecycle, continuation, and preview support",
                &[
                    format!("subject_support_class:{}", subject.support_class().as_str()),
                    format!("evidence_source:{}", evidence.source_projection().label()),
                ],
            ));
        }
        _ => {}
    }

    Ok(())
}
