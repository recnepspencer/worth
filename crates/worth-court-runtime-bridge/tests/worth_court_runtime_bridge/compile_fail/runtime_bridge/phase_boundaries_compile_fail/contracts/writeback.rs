use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
    ProductionTypeMismatchPair as Pair,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![
        Contract {
            fixture: "truth_writeback_request_literal_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &["TruthWritebackRequest", "private fields"],
            }],
        },
        Contract {
            fixture: "writeback_family_match_non_exhaustive.rs",
            errors: &[Diagnostic::TypeMismatch {
                pairs: &[Pair {
                    expected: "[BridgeWritebackFamilyKind; 1]",
                    found: "[BridgeWritebackFamilyKind; 2]",
                }],
                symbols: &[
                    "BridgeWritebackFamilyKind",
                    "BridgeWritebackFamilyKind; 1",
                    "BridgeWritebackFamilyKind; 2",
                ],
            }],
        },
        Contract {
            fixture: "writeback_feedback_context_literal_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &["BridgeWritebackFeedbackContext", "private fields"],
            }],
        },
    ]
}
