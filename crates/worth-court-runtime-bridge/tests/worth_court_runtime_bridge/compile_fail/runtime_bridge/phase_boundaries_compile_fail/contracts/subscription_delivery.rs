use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
    ProductionTypeMismatchPair as Pair,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![
        Contract {
            fixture: "subscription_basis_binding_literal_private.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0624",
                    symbols: &["new"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "worth_foundational::identities::authority_identity::current::FoundationalAuthorityIdentity<Arc<str>, BridgeTruthAuthority, BridgeEvidenceReferenceIdentityKind>",
                            found: "&'static str",
                        },
                    ],
                    symbols: &["Arc", "BridgeEvidenceReferenceIdentityKind", "BridgeTruthAuthority", "FoundationalAuthorityIdentity", "str"],
                },
                Diagnostic::Uncoded {
                    cause: "cannot construct",
                    symbols: &["ValidatedSubscriptionBasisBinding", "private fields"],
                },
            ],
        },
        Contract {
            fixture: "subscription_continuation_requires_index.rs",
            errors: &[
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "&BridgeSubscriptionContinuationIndex",
                            found: "&BridgeSubscriptionContinuationCandidateInput",
                        },
                    ],
                    symbols: &["BridgeSubscriptionContinuationCandidateInput", "BridgeSubscriptionContinuationIndex"],
                },
            ],
        },
        Contract {
            fixture: "subscription_declaration_literal_private.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0624",
                    symbols: &["new"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "worth_foundational::identities::authority_identity::current::FoundationalAuthorityIdentity<Arc<str>, BridgeTruthAuthority, BridgeEvidenceReferenceIdentityKind>",
                            found: "&'static str",
                        },
                    ],
                    symbols: &["Arc", "BridgeEvidenceReferenceIdentityKind", "BridgeTruthAuthority", "FoundationalAuthorityIdentity", "str"],
                },
                Diagnostic::Uncoded {
                    cause: "cannot construct",
                    symbols: &["BridgeSubscriptionDeclaration", "private fields"],
                },
            ],
        },
        Contract {
            fixture: "subscription_shared_delivery_projection_requires_sealed_bundle.rs",
            errors: &[
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "&BridgeSharedConsumerDeliveryBundleSealed",
                            found: "&BridgeSharedConsumerDeliveryBundleDraft",
                        },
                    ],
                    symbols: &["BridgeSharedConsumerDeliveryBundleDraft", "BridgeSharedConsumerDeliveryBundleSealed"],
                },
            ],
        },
        Contract {
            fixture: "subscription_slice_generic_aspect_surface_removed.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &["NormalizedSubscriptionSliceIntent", "try_new_aspect_surface"],
                },
            ],
        },
    ]
}
