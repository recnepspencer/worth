use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
    ProductionTypeMismatchPair as Pair,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![
        Contract {
            fixture: "conditional_decision_seed_literal_private.rs",
            errors: &[
                Diagnostic::Uncoded {
                    cause: "cannot construct",
                    symbols: &["BridgeRetainedConditionalDecisionSeed", "private fields"],
                },
            ],
        },
        Contract {
            fixture: "conditional_lowering_projection_is_not_lowering.rs",
            errors: &[
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "BridgeInstalledConditionalLowering",
                            found: "worth_foundational::identities::authority_identity::derived::FoundationalProjectionIdentity<Arc<str>, BridgeConditionalLoweringIdentityKind>",
                        },
                    ],
                    symbols: &["Arc", "BridgeConditionalLoweringIdentityKind", "BridgeInstalledConditionalLowering", "FoundationalProjectionIdentity", "str"],
                },
            ],
        },
    ]
}
