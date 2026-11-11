use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
    ProductionTypeMismatchPair as Pair,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![
        Contract {
            fixture: "builder_requires_relational_source_before_build.rs",
            errors: &[Diagnostic::Code {
                code: "E0599",
                symbols: &["build", "RuntimeBridgeBuilder<worth_runtime_bridge::builder::states::MissingCommittedPatchSource, worth_runtime_bridge::builder::states::MissingSnapshotReadSource, worth_runtime_bridge::builder::states::PresentSignalSink, worth_runtime_bridge::builder::states::MissingTruthBranchHeadSource, worth_runtime_bridge::builder::states::PresentMappingRegistrations>"],
            }],
        },
        Contract {
            fixture: "builder_requires_signal_sink_before_build.rs",
            errors: &[Diagnostic::Code {
                code: "E0599",
                symbols: &["build", "RuntimeBridgeBuilder<worth_runtime_bridge::builder::states::PresentCommittedPatchSource, worth_runtime_bridge::builder::states::PresentSnapshotReadSource, worth_runtime_bridge::builder::states::MissingSignalSink, worth_runtime_bridge::builder::states::MissingTruthBranchHeadSource, worth_runtime_bridge::builder::states::PresentMappingRegistrations>"],
            }],
        },
        Contract {
            fixture: "grouped_truth_view_artifact_fields_private.rs",
            errors: &[Diagnostic::Code {
                code: "E0451",
                symbols: &["BridgeGroupedTruthViewArtifact", "truth_view_digest", "basis_snapshot_identity", "contract", "members", "digest", "private"],
            }],
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
            errors: &[Diagnostic::Code {
                code: "E0624",
                symbols: &["new_subscription_slice", "private"],
            }],
        },
    ]
}
