use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
    ProductionTypeMismatchPair as Pair,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![
        Contract {
            fixture: "subscription_certification_assembly_plan_constructor_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &[
                    "BridgeSubscriptionCertificationAssemblyPlan",
                    "private fields",
                ],
            }],
        },
        Contract {
            fixture: "subscription_certification_closeout_artifact_constructor_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &[
                    "BridgeSubscriptionTemporalAsyncCertificationCloseoutArtifact",
                    "private fields",
                ],
            }],
        },
        Contract {
            fixture: "subscription_certification_compare_requires_sealed_bundle.rs",
            errors: &[Diagnostic::TypeMismatch {
                pairs: &[
                    Pair {
                        expected: "&BridgeSubscriptionCertificationBundleSealed",
                        found: "&&BridgeSubscriptionCertificationBundleDraft",
                    },
                    Pair {
                        expected: "&BridgeSubscriptionCertificationBundleSealed",
                        found: "&&BridgeSubscriptionCertificationBundleDraft",
                    },
                ],
                symbols: &[
                    "BridgeSubscriptionCertificationBundleDraft",
                    "BridgeSubscriptionCertificationBundleSealed",
                ],
            }],
        },
        Contract {
            fixture: "subscription_certification_comparison_plan_constructor_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &[
                    "BridgeSubscriptionCertificationComparisonPlan",
                    "private fields",
                ],
            }],
        },
        Contract {
            fixture: "subscription_certification_comparison_report_constructor_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &[
                    "BridgeSubscriptionCertificationComparisonReport",
                    "private fields",
                ],
            }],
        },
        Contract {
            fixture: "subscription_certification_cost_profile_constructor_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &[
                    "BridgeSubscriptionCertificationCostProfile",
                    "private fields",
                ],
            }],
        },
        Contract {
            fixture: "subscription_certification_scratch_constructor_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &["BridgeSubscriptionCertificationScratch", "private fields"],
            }],
        },
        Contract {
            fixture: "subscription_certification_sealed_bundle_constructor_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &[
                    "BridgeSubscriptionCertificationBundleSealed",
                    "private fields",
                ],
            }],
        },
        Contract {
            fixture: "subscription_certification_support_matrix_row_constructor_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &[
                    "BridgeSubscriptionTemporalAsyncCertificationSupportMatrixRow",
                    "private fields",
                ],
            }],
        },
    ]
}
