use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![
        Contract {
            fixture: "private_world_module.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0423",
                    symbols: &["RuntimeWorldOwnerIdentity", "worth_runtime_world::identity"],
                },
                Diagnostic::Code {
                    code: "E0603",
                    symbols: &["identity", "private"],
                },
            ],
        },
        Contract {
            fixture: "private_product_head_surface.rs",
            errors: &[Diagnostic::Code {
                code: "E0432",
                symbols: &[
                    "ProductBranchHeadProtection",
                    "ProductHeadRetentionObligation",
                    "RetentionTransferReceipt",
                ],
            }],
        },
        Contract {
            fixture: "private_retention_surface.rs",
            errors: &[Diagnostic::Code {
                code: "E0432",
                symbols: &[
                    "ExactComponentBasisKey",
                    "ExactComponentPinRequest",
                    "ObservationRetentionObligation",
                    "PublicationRetentionObligation",
                    "RetainedPartialRetentionObligation",
                    "RetentionTransferReceipt",
                    "RuntimeWorldRetentionOwner",
                ],
            }],
        },
    ]
}
