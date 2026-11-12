use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![
        Contract {
            fixture: "subscription_preview_active_literal_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &["BridgePreviewActiveSubscription", "private fields"],
            }],
        },
        Contract {
            fixture: "subscription_preview_basis_literal_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &["BridgeSubscriptionPreviewBasisBinding", "private fields"],
            }],
        },
        Contract {
            fixture: "subscription_preview_discard_residue_proof_literal_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &[
                    "BridgeSubscriptionPreviewDiscardResidueProof",
                    "private fields",
                ],
            }],
        },
        Contract {
            fixture: "subscription_preview_residue_artifact_record_literal_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &[
                    "BridgeSubscriptionPreviewResidueArtifactRecord",
                    "private fields",
                ],
            }],
        },
        Contract {
            fixture: "subscription_preview_residue_scope_index_literal_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &[
                    "BridgeSubscriptionPreviewResidueScopeIndex",
                    "private fields",
                ],
            }],
        },
        Contract {
            fixture: "subscription_preview_temporal_admission_constructor_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &[
                    "AdmittedPreviewTemporalBridgeSubscription",
                    "private fields",
                ],
            }],
        },
        Contract {
            fixture: "subscription_preview_work_record_literal_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &["BridgeSubscriptionPreviewWorkRecord", "private fields"],
            }],
        },
        Contract {
            fixture: "subscription_preview_work_trace_literal_private.rs",
            errors: &[Diagnostic::Uncoded {
                cause: "cannot construct",
                symbols: &["BridgeSubscriptionPreviewWorkTrace", "private fields"],
            }],
        },
    ]
}
