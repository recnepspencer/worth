use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![Contract {
        fixture: "diagnostics_failure_localization_constructor_private.rs",
        errors: &[Diagnostic::Uncoded {
            cause: "cannot construct",
            symbols: &["BridgeLocalizedTemporalAsyncFailure", "private fields"],
        }],
    }]
}
