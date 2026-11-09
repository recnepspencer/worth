use worth_server::{
    WorthServerResponseReceipt, WorthServerResponseTransform, WorthServerSuccessEnvelope,
};

fn main() {
    let _ = WorthServerSuccessEnvelope {
        transform: WorthServerResponseTransform::worth_native(),
        diagnostics_profile: worth_server::request_context::DiagnosticRichnessProfile::Standard,
        payload: supplied(),
        provenance: supplied(),
        receipt: WorthServerResponseReceipt::Completed(supplied()),
        canonical_digest: "Worthd".to_string(),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
