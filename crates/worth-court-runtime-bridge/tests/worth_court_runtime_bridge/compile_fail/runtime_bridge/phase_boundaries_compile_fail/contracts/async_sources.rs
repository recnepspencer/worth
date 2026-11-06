use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
    ProductionTypeMismatchPair as Pair,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![
        Contract {
            fixture: "source_async_completion_admission_requires_request_identity.rs",
            errors: &[
                Diagnostic::Arity {
                    required: 2,
                    supplied: 1,
                    symbols: &["AdmittedBridgeAsyncRequestIdentity"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "Result<(), BridgeAsyncCompletionRejection>",
                            found: "Result<BridgeAsyncCompletionAdmissionReport, BridgeAsyncCompletionRejection>",
                        },
                    ],
                    symbols: &["BridgeAsyncCompletionAdmissionReport", "BridgeAsyncCompletionRejection", "Result"],
                },
            ],
        },
        Contract {
            fixture: "source_async_completion_constructor_private.rs",
            errors: &[
                Diagnostic::Uncoded {
                    cause: "cannot construct",
                    symbols: &["AdmittedBridgeAsyncCompletion", "private fields"],
                },
            ],
        },
        Contract {
            fixture: "source_async_denied_completion_constructor_private.rs",
            errors: &[
                Diagnostic::Uncoded {
                    cause: "cannot construct",
                    symbols: &["BridgeAsyncDeniedCompletion", "private fields"],
                },
            ],
        },
        Contract {
            fixture: "source_async_lowered_constructor_private.rs",
            errors: &[
                Diagnostic::Uncoded {
                    cause: "cannot construct",
                    symbols: &["LoweredBridgeAsyncSourceDeclaration", "private fields"],
                },
            ],
        },
        Contract {
            fixture: "source_async_request_admission_requires_lowered_declaration.rs",
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
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "&LoweredBridgeAsyncSourceDeclaration",
                            found: "&ValidatedBridgeAsyncSourceDeclaration",
                        },
                    ],
                    symbols: &["LoweredBridgeAsyncSourceDeclaration", "ValidatedBridgeAsyncSourceDeclaration"],
                },
            ],
        },
        Contract {
            fixture: "source_async_request_identity_constructor_private.rs",
            errors: &[
                Diagnostic::Uncoded {
                    cause: "cannot construct",
                    symbols: &["AdmittedBridgeAsyncRequestIdentity", "private fields"],
                },
            ],
        },
        Contract {
            fixture: "source_async_retry_lineage_constructor_private.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0451",
                    symbols: &["BridgeAsyncRetryLineage", "canonical_basis", "causality_identity", "class", "counters", "digest", "newer_request", "prior_request", "receipt"],
                },
                Diagnostic::Code {
                    code: "E0451",
                    symbols: &["BridgeAsyncForwardCausalityReceipt", "canonical_basis", "causality_identity", "class", "digest", "receipt_identity"],
                },
            ],
        },
        Contract {
            fixture: "source_async_revalidation_lineage_constructor_private.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0451",
                    symbols: &["BridgeAsyncRevalidationLineage", "canonical_basis", "causality_identity", "class", "counters", "digest", "newer_request", "prior_request", "receipt"],
                },
            ],
        },
        Contract {
            fixture: "source_async_validated_constructor_private.rs",
            errors: &[
                Diagnostic::Uncoded {
                    cause: "cannot construct",
                    symbols: &["ValidatedBridgeAsyncSourceDeclaration", "private fields"],
                },
            ],
        },
        Contract {
            fixture: "source_async_writeback_constructor_private.rs",
            errors: &[
                Diagnostic::Uncoded {
                    cause: "cannot construct",
                    symbols: &["AdmittedBridgeAsyncWriteback", "private fields"],
                },
            ],
        },
        Contract {
            fixture: "source_async_writeback_requires_admitted_completion.rs",
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
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "&AdmittedBridgeAsyncCompletion",
                            found: "&BridgeAsyncCompletionAdmissionReport",
                        },
                    ],
                    symbols: &["AdmittedBridgeAsyncCompletion", "BridgeAsyncCompletionAdmissionReport"],
                },
            ],
        },
        Contract {
            fixture: "source_async_writeback_staged_effect_constructor_private.rs",
            errors: &[
                Diagnostic::Uncoded {
                    cause: "cannot construct",
                    symbols: &["StagedBridgeAsyncWritebackEffect", "private fields"],
                },
            ],
        },
    ]
}
