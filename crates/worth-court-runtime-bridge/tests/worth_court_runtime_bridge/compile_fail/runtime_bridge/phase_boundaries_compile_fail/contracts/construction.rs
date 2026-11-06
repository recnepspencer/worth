use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
    ProductionTypeMismatchPair as Pair,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![
        Contract {
            fixture: "builder_requires_relational_source_before_build.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0624",
                    symbols: &["new", "private"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "FoundationalAuthorityIdentity<Arc<str>, BridgeTruthAuthority, BridgeEvidenceReferenceIdentityKind>",
                            found: "&'static str",
                        },
                    ],
                    symbols: &["BridgeEvidenceReferenceIdentityKind", "BridgeTruthAuthority", "FoundationalAuthorityIdentity", "str"],
                },
                Diagnostic::Code {
                    code: "E0624",
                    symbols: &["new", "private"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "FoundationalAuthorityIdentity<Arc<str>, BridgeTruthAuthority, BridgeEvidenceReferenceIdentityKind>",
                            found: "&'static str",
                        },
                    ],
                    symbols: &["BridgeEvidenceReferenceIdentityKind", "BridgeTruthAuthority", "FoundationalAuthorityIdentity", "str"],
                },
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &["MissingCommittedPatchSource", "MissingSignalSink", "MissingSnapshotReadSource", "MissingTruthBranchHeadSource", "PresentMappingRegistrations", "RuntimeBridgeBuilder", "build"],
                },
            ],
        },
        Contract {
            fixture: "builder_requires_signal_sink_before_build.rs",
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
                    code: "E0599",
                    symbols: &["MissingSignalSink", "MissingTruthBranchHeadSource", "PresentCommittedPatchSource", "PresentMappingRegistrations", "PresentSnapshotReadSource", "RuntimeBridgeBuilder", "build"],
                },
            ],
        },
        Contract {
            fixture: "grouped_truth_view_artifact_fields_private.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0624",
                    symbols: &["new", "private"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "worth_foundational::identities::authority_identity::current::FoundationalAuthorityIdentity<Arc<str>, BridgeTruthAuthority, BridgeEvidenceReferenceIdentityKind>",
                            found: "&'static str",
                        },
                    ],
                    symbols: &["BridgeEvidenceReferenceIdentityKind", "BridgeTruthAuthority", "FoundationalAuthorityIdentity", "str"],
                },
            ],
        },
        Contract {
            fixture: "owned_async_product_observation_constructor_private.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0432",
                    symbols: &["BridgeOwnedAsyncProductObservation"],
                },
            ],
        },
        Contract {
            fixture: "private_builder_module.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0603",
                    symbols: &["builder"],
                },
            ],
        },
        Contract {
            fixture: "private_snapshot_token_constructor.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0624",
                    symbols: &["issued"],
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
            ],
        },
        Contract {
            fixture: "snapshot_read_coordinate_constructor_private.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0624",
                    symbols: &["new_subscription_slice", "private"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "worth_runtime_bridge::identity::BridgeIdentity<worth_runtime_bridge::identity::SnapshotReadTargetIdentityTag>",
                            found: "&'static str",
                        },
                    ],
                    symbols: &["SnapshotReadTargetIdentityTag", "str"],
                },
            ],
        },
    ]
}
