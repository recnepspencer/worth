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
        counter_receipt: loop {},
        attachment_bundle: loop {},
        materialized_attachment_bundle: loop {},
    };
}
