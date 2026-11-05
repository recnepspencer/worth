//! Compile fail integration courts.
use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
};

#[test]
fn owned_async_progression_requires_actual_cleanup_custody() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_world/sources/worth_runtime_world/ui/owned_async_progression_requires_custody.rs")).expect("prebuilt compiler denial");
}

#[test]
fn compile_failures_protect_the_public_contract() {
    worth_testkit_compiler::production_denials(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/worth_court_runtime_world/sources/worth_runtime_world/ui/*.rs"
        ),
        &[Contract {
            fixture: "private_product_head_surface.rs",
            errors: &[Diagnostic::Code {
                code: "E0432",
                symbols: &[
                    "ProductBranchHeadProtection",
                    "ProductHeadRetentionObligation",
                    "RetentionTransferReceipt",
                ],
            }],
        }],
    )
    .expect("production compiler denial");
}
