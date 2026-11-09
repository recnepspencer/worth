use worth_server::{
    WorthServerEvidenceTransform, WorthServerOperatorEvidenceClass,
    WorthServerOperatorEvidenceRecord,
};

fn main() {
    let _ = WorthServerOperatorEvidenceRecord {
        transform: WorthServerEvidenceTransform::operator_default(),
        diagnostics_profile: worth_server::request_context::DiagnosticRichnessProfile::Standard,
        response_digest: "Worthd".to_string(),
        classification: WorthServerOperatorEvidenceClass::QueryReadSucceeded,
        counter_receipt: supplied(),
        attachment_bundle: supplied(),
        materialized_attachment_bundle: supplied(),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
