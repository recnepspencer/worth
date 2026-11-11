use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![
        Contract {
            fixture: "receipt_cannot_be_fabricated.rs",
            errors: &[Diagnostic::Code {
                code: "E0451",
                symbols: &[
                    "RelationalChangeReceipt",
                    "patch",
                    "records_examined",
                    "records_filtered_out",
                    "consistency",
                    "proof",
                    "commit_identity",
                    "private",
                ],
            }],
        },
        Contract {
            fixture: "retained_observation_cannot_be_fabricated.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &["RelationalRetainedObservation", "private fields"],
            }],
        },
        Contract {
            fixture: "selected_commit_cannot_be_fabricated.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &["RelationalSelectedCommit", "private fields"],
            }],
        },
    ]
}
