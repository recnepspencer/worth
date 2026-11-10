use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
    ProductionTypeMismatchPair as Pair,
};

pub(super) fn contracts() -> Vec<Contract<'static>> {
    vec![
        Contract {
            fixture: "descriptor_cannot_begin_transaction.rs",
            errors: &[Diagnostic::TypeMismatch {
                pairs: &[Pair {
                    expected: "&AdmittedRelationalBranchBasis",
                    found: "&RelationalBranchBasisDescriptor",
                }],
                symbols: &["AdmittedRelationalBranchBasis", "RelationalBranchBasisDescriptor"],
            }],
        },
        Contract {
            fixture: "fork_basis_has_no_nonfork_capabilities.rs",
            errors: &[Diagnostic::TypeMismatch {
                pairs: &[Pair {
                    expected: "&AdmittedRelationalBranchBasis",
                    found: "&AdmittedRelationalForkSourceBasis",
                }],
                symbols: &["AdmittedRelationalBranchBasis", "AdmittedRelationalForkSourceBasis"],
            }],
        },
        Contract {
            fixture: "admitted_basis_cannot_be_fabricated.rs",
            errors: &[Diagnostic::Code {
                code: "E0451",
                symbols: &["AdmittedRelationalBranchBasis", "inner", "private"],
            }],
        },
        Contract {
            fixture: "commit_reference_is_not_public.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0432",
                    symbols: &["CommitReference"],
                },
            ],
        },
        Contract {
            fixture: "forged_branch_authority.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0423",
                    symbols: &["RelationalBranchObservationAuthorityMarker", "private fields"],
                },
            ],
        },
        Contract {
            fixture: "fork_provenance_is_not_authority.rs",
            errors: &[
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "AdmittedRelationalForkSourceBasis",
                            found: "&worth_foundational::transitions::branches::reference::FoundationalBranchReferenceObservation<RelationalBranchTarget>",
                        },
                    ],
                    symbols: &["AdmittedRelationalForkSourceBasis", "FoundationalBranchReferenceObservation", "RelationalBranchTarget"],
                },
            ],
        },
        Contract {
            fixture: "inspection_artifacts_cannot_open_authority.rs",
            errors: &[
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "AdmittedRelationalForkSourceBasis",
                            found: "RelationalBranchSharingObservation",
                        },
                    ],
                    symbols: &["AdmittedRelationalForkSourceBasis", "RelationalBranchSharingObservation"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "&AdmittedRelationalBranchBasis",
                            found: "&RelationalOwnerAllocationLedgerObservation",
                        },
                    ],
                    symbols: &["AdmittedRelationalBranchBasis", "RelationalOwnerAllocationLedgerObservation"],
                },
                Diagnostic::TypeMismatch {
                    pairs: &[
                        Pair {
                            expected: "VersionId",
                            found: "RelationalVisibilityCommitmentObservation",
                        },
                    ],
                    symbols: &["RelationalVisibilityCommitmentObservation", "VersionId"],
                },
            ],
        },
        Contract {
            fixture: "publication_authority_cannot_be_forged.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0423",
                    symbols: &["RelationalBranchPublicationAuthorityMarker", "private fields"],
                },
                Diagnostic::Code {
                    code: "E0624",
                    symbols: &["seal"],
                },
                Diagnostic::Code {
                    code: "E0624",
                    symbols: &["witness"],
                },
            ],
        },
    ]
}
