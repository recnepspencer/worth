use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
    ProductionTypeMismatchPair as Pair,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![
        Contract {
            fixture: "preview_replay_requires_session_identity.rs",
            errors: &[
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "&worth_runtime_bridge::identity::BridgeIdentity<worth_runtime_bridge::identity::PreviewSessionIdentityTag>",
                            found: "&str",
                        },
                    ],
                    symbols: &["BridgeIdentity", "PreviewSessionIdentityTag", "str"],
                },
            ],
        },
        Contract {
            fixture: "preview_session_declaration_literal_private.rs",
            errors: &[Diagnostic::Code {
                code: "E0451",
                symbols: &["BridgePreviewSessionDeclaration", "declaration_identity", "request_kind", "branch_binding", "session_basis", "request_shape_basis", "structural_basis", "truth_view_basis_digest", "structural_basis_digest", "source_capability_digest", "request_shape_digest", "retained_artifact_schema_digest", "canonical_basis", "digest", "private"],
            }],
        },
    ]
}
