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
                    symbols: &["Display", "TruthCommitIdentity"],
                },
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &["as_str", "TruthCommitIdentity"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[Pair { expected: "TruthCommitIdentity", found: "&str" }],
                    symbols: &["TruthCommitIdentity", "str"],
                },
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &["new", "TruthCommitIdentity"],
                },
                Diagnostic::Code {
                    code: "E0277",
                    symbols: &["Display", "TruthSnapshotIdentity"],
                },
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &["as_str", "TruthSnapshotIdentity"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[Pair { expected: "TruthSnapshotIdentity", found: "&str" }],
                    symbols: &["TruthSnapshotIdentity", "str"],
                },
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &["new", "TruthSnapshotIdentity"],
                },
                Diagnostic::Code {
                    code: "E0277",
                    symbols: &["Display", "TruthPatchIdentity"],
                },
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &["as_str", "TruthPatchIdentity"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[Pair { expected: "TruthPatchIdentity", found: "&str" }],
                    symbols: &["TruthPatchIdentity", "str"],
                },
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &["new", "TruthPatchIdentity"],
                },
                Diagnostic::Code {
                    code: "E0277",
                    symbols: &["Display", "TruthBranchIdentity"],
                },
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &["as_str", "TruthBranchIdentity"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[Pair { expected: "TruthBranchIdentity", found: "&str" }],
                    symbols: &["TruthBranchIdentity", "str"],
                },
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &["new", "TruthBranchIdentity"],
                },
            ],
        },
        Contract {
            fixture: "wrong_kind_cannot_satisfy_truth_family.rs",
            errors: &[
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "TruthBranchIdentity",
                            found: "TruthCommitIdentity",
                        },
                    ],
                    symbols: &["TruthBranchIdentity", "TruthCommitIdentity"],
                },
            ],
        },
    ]
}
