use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
    ProductionTypeMismatchPair as Pair,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![
        Contract {
            fixture: "digest_cannot_satisfy_truth_authority.rs",
            errors: &[
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "worth_foundational::identities::authority_identity::external::FoundationalExternalIdentityToken<Arc<str>, BridgeEvidenceReferenceIdentityKind>",
                            found: "worth_foundational::identities::authority_identity::derived::FoundationalDigestIdentityEvidence<BridgeCanonicalDigestIdentityBasis, BridgeTruthAuthority, BridgeEvidenceReferenceIdentityKind>",
                        },
                    ],
                    symbols: &["BridgeCanonicalDigestIdentityBasis", "BridgeEvidenceReferenceIdentityKind", "BridgeTruthAuthority", "FoundationalDigestIdentityEvidence", "FoundationalExternalIdentityToken"],
                },
            ],
        },
        Contract {
            fixture: "external_token_cannot_satisfy_truth_authority.rs",
            errors: &[
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "worth_foundational::identities::authority_identity::derived::FoundationalProjectionIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>",
                            found: "worth_foundational::identities::authority_identity::external::FoundationalExternalIdentityToken<Arc<str>, BridgeEvidenceReferenceIdentityKind>",
                        },
                    ],
                    symbols: &["Arc", "BridgeEvidenceReferenceIdentityKind", "FoundationalExternalIdentityToken", "FoundationalProjectionIdentity", "str"],
                },
            ],
        },
        Contract {
            fixture: "projection_cannot_satisfy_truth_authority.rs",
            errors: &[
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "worth_foundational::identities::authority_identity::external::FoundationalExternalIdentityToken<Arc<str>, BridgeEvidenceReferenceIdentityKind>",
                            found: "worth_foundational::identities::authority_identity::derived::FoundationalProjectionIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>",
                        },
                    ],
                    symbols: &["Arc", "BridgeEvidenceReferenceIdentityKind", "FoundationalExternalIdentityToken", "FoundationalProjectionIdentity", "str"],
                },
            ],
        },
        Contract {
            fixture: "raw_text_cannot_satisfy_truth_authority.rs",
            errors: &[
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "worth_foundational::identities::authority_identity::external::FoundationalExternalIdentityToken<Arc<str>, BridgeEvidenceReferenceIdentityKind>",
                            found: "&'static str",
                        },
                    ],
                    symbols: &["Arc", "BridgeEvidenceReferenceIdentityKind", "FoundationalExternalIdentityToken", "str"],
                },
            ],
        },
        Contract {
            fixture: "retained_evidence_cannot_rebuild_from_text.rs",
            errors: &[
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "worth_foundational::identities::authority_identity::derived::FoundationalProjectionIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>",
                            found: "&'static str",
                        },
                        Pair {
                            expected: "worth_foundational::identities::authority_identity::derived::FoundationalDigestIdentityEvidence<BridgeCanonicalDigestIdentityBasis, BridgeTruthAuthority, BridgeEvidenceReferenceIdentityKind>",
                            found: "&'static str",
                        },
                    ],
                    symbols: &["Arc", "BridgeCanonicalDigestIdentityBasis", "BridgeEvidenceReferenceIdentityKind", "BridgeTruthAuthority", "FoundationalDigestIdentityEvidence", "FoundationalProjectionIdentity", "str"],
                },
            ],
        },
        Contract {
            fixture: "truth_commit_identity_string_facade_private.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0277",
                    symbols: &["BridgeIdentity", "Display", "TruthCommitTag"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "worth_runtime_bridge::identity::BridgeIdentity<worth_runtime_bridge::identity::TruthCommitTag>",
                            found: "&'static str",
                        },
                    ],
                    symbols: &["BridgeIdentity", "TruthCommitTag", "str"],
                },
                Diagnostic::Code {
                    code: "E0277",
                    symbols: &["BridgeIdentity", "Display", "TruthSnapshotTag"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "worth_runtime_bridge::identity::BridgeIdentity<worth_runtime_bridge::identity::TruthSnapshotTag>",
                            found: "&'static str",
                        },
                    ],
                    symbols: &["BridgeIdentity", "TruthSnapshotTag", "str"],
                },
                Diagnostic::Code {
                    code: "E0277",
                    symbols: &["BridgeIdentity", "Display", "TruthPatchTag"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "worth_runtime_bridge::identity::BridgeIdentity<worth_runtime_bridge::identity::TruthPatchTag>",
                            found: "&'static str",
                        },
                    ],
                    symbols: &["BridgeIdentity", "TruthPatchTag", "str"],
                },
                Diagnostic::Code {
                    code: "E0277",
                    symbols: &["BridgeIdentity", "Display", "TruthBranchTag"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "worth_runtime_bridge::identity::BridgeIdentity<worth_runtime_bridge::identity::TruthBranchTag>",
                            found: "&'static str",
                        },
                    ],
                    symbols: &["BridgeIdentity", "TruthBranchTag", "str"],
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
                    symbols: &["as_str"],
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
                    symbols: &["as_str"],
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
                    symbols: &["as_str"],
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
                    symbols: &["as_str"],
                },
            ],
        },
        Contract {
            fixture: "wrong_kind_cannot_satisfy_truth_family.rs",
            errors: &[
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "worth_runtime_bridge::identity::BridgeIdentity<worth_runtime_bridge::identity::TruthBranchTag>",
                            found: "worth_runtime_bridge::identity::BridgeIdentity<worth_runtime_bridge::identity::TruthCommitTag>",
                        },
                    ],
                    symbols: &["BridgeIdentity", "TruthBranchTag", "TruthCommitTag"],
                },
            ],
        },
    ]
}
