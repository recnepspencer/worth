use worth_server::{
    WorthServerResponseReceipt, WorthServerResponseTransform, WorthServerSuccessEnvelope,
};

fn main() {
    let _ = WorthServerSuccessEnvelope {
        transform: WorthServerResponseTransform::worth_native(),
        diagnostics_profile: worth_server::request_context::DiagnosticRichnessProfile::Standard,
        payload: loop {},
        provenance: loop {},
        receipt: WorthServerResponseReceipt::Completed(loop {}),
        canonical_digest: "Worthd".to_string(),
    };
}
