use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
};

#[test]
fn external_code_cannot_construct_or_reuse_sealed_phase_artifacts_directly() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let outcomes = [
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/private_middleware_admission_constructor.rs",
        )),
        worth_testkit_compiler::production_denials(
            root.join("../worth-testkit-server/assets/ui/construction/response/private_counter_receipt_constructor.rs"),
            &[
                Contract {
                    fixture: "private_counter_receipt_constructor.rs",
                    errors: &[
                        Diagnostic::Code {
                            code: "E0451",
                            symbols: &["WorthServerOperatorCounterReceipt", "inner", "private"],
                        },
                    ],
                }
            ],
        ),
        worth_testkit_compiler::production_denials(
            root.join("../worth-testkit-server/assets/ui/construction/response/private_denial_envelope_constructor.rs"),
            &[
                Contract {
                    fixture: "private_denial_envelope_constructor.rs",
                    errors: &[
                        Diagnostic::Code {
                            code: "E0451",
                            symbols: &["WorthServerDenialEnvelope", "canonical_digest", "cause", "diagnostics_profile", "provenance", "receipt", "transform"],
                        },
                    ],
                }
            ],
        ),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/response/private_evidence_record_constructor.rs",
        )),
        worth_testkit_compiler::production_denials(
            root.join("../worth-testkit-server/assets/ui/construction/response/private_success_envelope_constructor.rs"),
            &[
                Contract {
                    fixture: "private_success_envelope_constructor.rs",
                    errors: &[
                        Diagnostic::Code {
                            code: "E0451",
                            symbols: &["WorthServerSuccessEnvelope", "canonical_digest", "diagnostics_profile", "payload", "provenance", "receipt", "transform"],
                        },
                    ],
                }
            ],
        ),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/query_handoff/private_prepared_query_handoff_constructor.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/query_handoff/private_query_handoff_constructor.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/worth_native/private_prepared_session_constructor.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/worth_native/private_session_constructor.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/worth_native/private_direct_facade_constructor.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/worth_native/private_raw_query_workspace_access.rs",
        )),
        worth_testkit_compiler::production_denials(
            root.join("tests/worth_court_server_product_operation_phase_thirteen_shared_read/sources/worth_server/ui/construction/compat_http/private_request_constructor.rs"),
            &[
                Contract {
                    fixture: "private_request_constructor.rs",
                    errors: &[
                        Diagnostic::Code {
                            code: "E0451",
                            symbols: &["WorthServerCompatibilityRequest", "admission", "request_contract"],
                        },
                    ],
                }
            ],
        ),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/worth_native/private_direct_lease_declaration_constructor.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/worth_native/private_direct_delivery_contract_constructor.rs",
        )),
        worth_testkit_compiler::production_denials(
            root.join("../worth-testkit-server/assets/ui/construction/private_request_context_constructor.rs"),
            &[
                Contract {
                    fixture: "private_request_context_constructor.rs",
                    errors: &[
                        Diagnostic::Code {
                            code: "E0624",
                            symbols: &["new"],
                        },
                    ],
                }
            ],
        ),
        worth_testkit_compiler::production_denials(
            root.join("../worth-testkit-server/assets/ui/construction/private_lowered_operation_plan_constructor.rs"),
            &[
                Contract {
                    fixture: "private_lowered_operation_plan_constructor.rs",
                    errors: &[
                        Diagnostic::Code {
                            code: "E0451",
                            symbols: &["WorthServerLoweredOperationPlan", "canonical_digest", "counters", "evidence_policy", "query_handoff", "receipt", "strategy"],
                        },
                    ],
                }
            ],
        ),
        worth_testkit_compiler::production_denials(
            root.join("../worth-testkit-server/assets/ui/construction/private_lowered_operation_plan_handoff_access.rs"),
            &[
                Contract {
                    fixture: "private_lowered_operation_plan_handoff_access.rs",
                    errors: &[
                        Diagnostic::Code {
                            code: "E0624",
                            symbols: &["into_query_handoff"],
                        },
                    ],
                }
            ],
        ),
        worth_testkit_compiler::production_denials(
            root.join("../worth-testkit-server/assets/ui/construction/private_scheduler_raw_input_access.rs"),
            &[
                Contract {
                    fixture: "private_scheduler_raw_input_access.rs",
                    errors: &[
                        Diagnostic::Code {
                            code: "E0271",
                            symbols: &["IntoIterator", "Item", "WorthServerLoweredOperationPlan", "WorthServerQueryHandoffInput", "WorthServerQueryHandoffInput; 1"],
                        },
                    ],
                }
            ],
        ),
        worth_testkit_compiler::production_denials(
            root.join("../worth-testkit-server/assets/ui/construction/private_surface_registration_constructor.rs"),
            &[
                Contract {
                    fixture: "private_surface_registration_constructor.rs",
                    errors: &[
                        Diagnostic::Uncoded {
                            cause: "cannot construct",
                            symbols: &["WorthServerSurfaceRegistration", "private fields"],
                        },
                    ],
                }
            ],
        ),
        worth_testkit_compiler::production_denials(
            root.join("../worth-testkit-server/assets/ui/construction/private_server_constructor.rs"),
            &[
                Contract {
                    fixture: "private_server_constructor.rs",
                    errors: &[
                        Diagnostic::Code {
                            code: "E0624",
                            symbols: &["new"],
                        },
                    ],
                }
            ],
        ),
        worth_testkit_compiler::production_denials(
            root.join("../worth-testkit-server/assets/ui/construction/product_operation_result_contract_required.rs"),
            &[
                Contract {
                    fixture: "product_operation_result_contract_required.rs",
                    errors: &[
                        Diagnostic::Arity {
                            required: 5,
                            supplied: 4,
                            symbols: &["WorthServerProductResultContract"],
                        },
                    ],
                }
            ],
        ),
        worth_testkit_compiler::production_denials(
            root.join("../worth-testkit-server/assets/ui/construction/product_result_requires_schema_bound_value.rs"),
            &[
                Contract {
                    fixture: "product_result_requires_schema_bound_value.rs",
                    errors: &[
                        Diagnostic::Code {
                            code: "E0277",
                            symbols: &["Value", "WorthServerProductResultValue"],
                        },
                    ],
                }
            ],
        ),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/cross_family/compat_http_cannot_import_worth_native_internal.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/cross_family/worth_native_cannot_import_compat_http_internal.rs",
        )),
        worth_testkit_compiler::production_denials(
            root.join("tests/worth_court_server_product_operation_phase_thirteen_shared_read/sources/worth_server/ui/lifecycle/server_cannot_serve_twice.rs"),
            &[
                Contract {
                    fixture: "server_cannot_serve_twice.rs",
                    errors: &[
                        Diagnostic::Code {
                            code: "E0382",
                            symbols: &["server"],
                        },
                    ],
                }
            ],
        ),
    ];
    let refusals = outcomes
        .into_iter()
        .filter_map(Result::err)
        .collect::<Vec<_>>();
    assert!(refusals.is_empty(), "compiler boundaries: {refusals:#?}");
}
