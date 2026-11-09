use worth_server::{
    WorthServerDenialBoundary, WorthServerDenialCause, WorthServerDenialEnvelope,
    WorthServerResponseReceipt, WorthServerResponseTransform,
};

fn main() {
    let cause = WorthServerDenialCause::RequestContext {
        code: worth_server::WorthServerRequestContextDenialCode::PreviewTargetingDisabled,
        diagnostics_profile: worth_server::request_context::DiagnosticRichnessProfile::Standard,
        detail: "Worthd".to_string(),
    };
    let _ = WorthServerDenialEnvelope {
        transform: WorthServerResponseTransform::compat_http(),
        diagnostics_profile: worth_server::request_context::DiagnosticRichnessProfile::Standard,
        cause,
        provenance: supplied(),
        receipt: WorthServerResponseReceipt::Completed(supplied()),
        canonical_digest: format!("{:?}", WorthServerDenialBoundary::RequestContext),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
