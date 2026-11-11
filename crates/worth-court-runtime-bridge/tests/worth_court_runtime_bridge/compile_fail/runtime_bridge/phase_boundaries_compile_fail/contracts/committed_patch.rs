use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![Contract {
        fixture: "committed_patch_target_generic_aspect_surface_removed.rs",
        errors: &[Diagnostic::Code {
            code: "E0599",
            symbols: &["BridgeCommittedPatchTarget", "aspect_surface"],
        }],
    }]
}
