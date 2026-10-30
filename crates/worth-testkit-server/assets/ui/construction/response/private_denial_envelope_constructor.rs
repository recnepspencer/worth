use worth_server::{WorthServerDenialBoundary, WorthServerDenialCause, WorthServerDenialEnvelope, WorthServerResponseReceipt, WorthServerResponseTransform};

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
        provenance: loop {},
        receipt: WorthServerResponseReceipt::Completed(loop {}),
        canonical_digest: format!("{:?}", WorthServerDenialBoundary::RequestContext),
    };
}
