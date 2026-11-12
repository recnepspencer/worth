use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
    ProductionTypeMismatchPair as Pair,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![
        Contract {
            fixture: "writeback_diagnostics_lookup_requires_retention_reference.rs",
            errors: &[Diagnostic::TypeMismatch {
                pairs: &[Pair {
                    expected: "&BridgeWritebackDiagnosticReference<BridgeWritebackReplayRecord>",
                    found: "&str",
                }],
                symbols: &[
                    "BridgeWritebackDiagnosticReference",
                    "BridgeWritebackReplayRecord",
                    "str",
                ],
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
        Contract {
            fixture: "writeback_mutation_authority_assembly_private.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0624",
                    symbols: &["from_successful_writeback_artifacts"],
                },
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &["BridgeMutationAuthorityBundle", "from_writeback_artifacts"],
                },
            ],
        },
    ]
}
