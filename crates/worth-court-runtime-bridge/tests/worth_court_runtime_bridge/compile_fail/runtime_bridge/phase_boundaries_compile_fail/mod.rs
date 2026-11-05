use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
};
#[test]
fn bridge_phase_boundaries_are_compile_time_private() {
    worth_testkit_compiler::denied(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/causal_envelope/*.rs"
    ))
    .expect("prebuilt compiler denial");
    worth_testkit_compiler::production_denials(
        concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/writeback_contract/*.rs"
    ),
        &[Contract {
            fixture: "raw_external_authority_evidence_cannot_construct_writeback_identity.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &[
                        "from_external_authority_evidence",
                        "WritebackEffectIdentityTag",
                    ],
                },
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &[
                        "from_external_authority_evidence",
                        "BridgeNamingAttachmentIdentity",
                    ],
                },
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &[
                        "from_external_authority_evidence",
                        "BridgeExistingTruthBindingAuthoritativeIdentity",
                    ],
                },
            ],
        }],
    )
    .expect("production compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/relational_identity/*.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/crate_private_constructors/*.rs")).expect("prebuilt compiler denial");
}

#[test]
fn bridge_public_contracts_a_to_f_are_sealed() {
    worth_testkit_compiler::production_denials(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/[a-f]*.rs"
        ),
        &[Contract {
            fixture: "builder_requires_relational_source_before_build.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0624",
                    symbols: &["new", "private"],
                },
                Diagnostic::Code {
                    code: "E0308",
                    symbols: &[
                        "FoundationalAuthorityIdentity",
                        "str",
                        "BridgeTruthAuthority",
                        "BridgeEvidenceReferenceIdentityKind",
                    ],
                },
                Diagnostic::Code {
                    code: "E0624",
                    symbols: &["new", "private"],
                },
                Diagnostic::Code {
                    code: "E0308",
                    symbols: &[
                        "FoundationalAuthorityIdentity",
                        "str",
                        "BridgeTruthAuthority",
                        "BridgeEvidenceReferenceIdentityKind",
                    ],
                },
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &[
                        "build",
                        "RuntimeBridgeBuilder",
                        "MissingCommittedPatchSource",
                        "MissingSnapshotReadSource",
                        "MissingSignalSink",
                        "MissingTruthBranchHeadSource",
                    ],
                },
            ],
        }],
    )
    .expect("production compiler denial");
}

#[test]
fn bridge_public_contracts_g_to_r_are_sealed() {
    worth_testkit_compiler::production_denials(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/[g-r]*.rs"
        ),
        &[Contract {
            fixture: "grouped_truth_view_artifact_fields_private.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0624",
                    symbols: &["new", "private"],
                },
                Diagnostic::Code {
                    code: "E0308",
                    symbols: &[
                        "FoundationalAuthorityIdentity",
                        "str",
                        "BridgeTruthAuthority",
                        "BridgeEvidenceReferenceIdentityKind",
                    ],
                },
            ],
        }],
    )
    .expect("production compiler denial");
}

#[test]
fn bridge_public_contracts_s_to_z_are_sealed() {
    worth_testkit_compiler::production_denials(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/[s-z]*.rs"
        ),
        &[Contract {
            fixture: "snapshot_read_coordinate_constructor_private.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0624",
                    symbols: &["new_subscription_slice", "private"],
                },
                Diagnostic::Code {
                    code: "E0308",
                    symbols: &["SnapshotReadTargetIdentityTag", "str"],
                },
            ],
        }],
    )
    .expect("production compiler denial");
}
