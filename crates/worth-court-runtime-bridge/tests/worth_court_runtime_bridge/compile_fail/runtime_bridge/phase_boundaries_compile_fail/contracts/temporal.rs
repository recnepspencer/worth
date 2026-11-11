use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
    ProductionTypeMismatchPair as Pair,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![
        Contract {
            fixture: "subscription_temporal_activation_ready_cannot_plan_temporal_delivery.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0624",
                    symbols: &["plan"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "&BridgeTemporalCauseRecord",
                            found: "&BridgeTemporalSubscriptionActivationReady",
                        },
                    ],
                    symbols: &["BridgeTemporalCauseRecord", "BridgeTemporalSubscriptionActivationReady"],
                },
            ],
        },
        Contract {
            fixture: "subscription_temporal_activation_ready_cannot_route_without_request.rs",
            errors: &[
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "&BridgeTemporalWakeRoutingRequest",
                            found: "&BridgeTemporalSubscriptionActivationReady",
                        },
                    ],
                    symbols: &["BridgeTemporalSubscriptionActivationReady", "BridgeTemporalWakeRoutingRequest"],
                },
            ],
        },
        Contract {
            fixture: "subscription_temporal_admission_constructor_private.rs",
            errors: &[
                Diagnostic::Uncoded {
                    cause: "cannot construct",
                    symbols: &["AdmittedTemporalBridgeSubscription", "private fields"],
                },
            ],
        },
        Contract {
            fixture: "temporal_basis_literal_private.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0624",
                    symbols: &["new"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "FoundationalAuthorityIdentity<Arc<str>, BridgeTruthAuthority, BridgeEvidenceReferenceIdentityKind>",
                            found: "&'static str",
                        },
                    ],
                    symbols: &["Arc", "BridgeEvidenceReferenceIdentityKind", "BridgeTruthAuthority", "FoundationalAuthorityIdentity", "str"],
                },
            ],
        },
        Contract {
            fixture: "temporal_basis_requires_typed_wake.rs",
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
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "BridgeTemporalWakeEvidence",
                            found: "String",
                        },
                    ],
                    symbols: &["BridgeTemporalWakeEvidence", "String"],
                },
            ],
        },
    ]
}
