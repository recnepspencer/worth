use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::runtime_writes::WorthQueryWriteAdmissionExecutionRecord;

pub(super) fn batch_execution_provenance(
    shared_admission: Option<&WorthQueryWriteAdmissionExecutionRecord>,
    combined_receipt: &WorthQueryMutationReceipt,
) -> Result<Option<WorthQueryIntentExecutionProvenance>, IdentityDerivationDenial> {
    shared_admission
        .map(|record| {
            let commit_label = combined_receipt
                .commit_identity
                .evidence_identity()
                .reporting_projection()
                .to_string();
            let snapshot_evidence_identity = combined_receipt.snapshot_identity.evidence_identity();
            WorthQueryIntentExecutionProvenance::for_shared_execution_typed_parts(
                record.family,
                record.entrypoint,
                record.execution_seam,
                &record.decision_digest,
                &record.handoff_digest,
                &record.binding_digest,
                &commit_label,
                &snapshot_evidence_identity,
            )
        })
        .transpose()
}

pub(super) fn batch_decision_trace_envelope(
    shared_admission: Option<&WorthQueryWriteAdmissionExecutionRecord>,
    combined_receipt: &WorthQueryMutationReceipt,
    batch_request_detail: &str,
) -> Option<WorthQueryIntentDecisionTraceEnvelope> {
    shared_admission.map(|record| {
        let commit_label = combined_receipt
            .commit_identity
            .evidence_identity()
            .reporting_projection()
            .to_string();
        WorthQueryIntentDecisionTraceEnvelope::for_admitted_execution_parts(
            record.family,
            record.entrypoint,
            &record.request_detail,
            &record.request_digest,
            record.eligibility_trace.clone(),
            &record.decision_digest,
            &record.handoff_digest,
            record.execution_seam,
            batch_request_detail,
            &commit_label,
            "mutation-batch-write",
        )
    })
}
