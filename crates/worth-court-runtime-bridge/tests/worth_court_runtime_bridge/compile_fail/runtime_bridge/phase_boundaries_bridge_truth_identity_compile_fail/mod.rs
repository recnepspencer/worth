use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
};
#[test]
fn bridge_truth_identity_string_facade_is_private() {
    worth_testkit_compiler::production_denials(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/bridge_truth_identity/*.rs"), &[Contract { fixture: "digest_cannot_satisfy_truth_authority.rs", errors: &[Diagnostic::Code { code: "E0308", symbols: &["FoundationalExternalIdentityToken", "FoundationalDigestIdentityEvidence", "BridgeCanonicalDigestIdentityBasis", "BridgeTruthAuthority", "BridgeEvidenceReferenceIdentityKind"] }] }]).expect("production compiler denial");
}
