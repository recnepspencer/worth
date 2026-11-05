use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
};

#[test]
fn external_code_cannot_construct_or_reuse_sealed_phase_artifacts_directly() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let counter = [Contract {
        fixture: "private_counter_receipt_constructor.rs",
        errors: &[Diagnostic::Code {
            code: "E0451",
            symbols: &["WorthServerOperatorCounterReceipt", "inner", "private"],
        }],
    }];
    let outcomes = [
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/private_middleware_admission_constructor.rs",
        )),
        worth_testkit_compiler::production_denials(
            root.join(
                "../worth-testkit-server/assets/ui/construction/response/private_counter_receipt_constructor.rs",
            ),
            &counter,
        ),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/response/private_denial_envelope_constructor.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/response/private_evidence_record_constructor.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/response/private_success_envelope_constructor.rs",
        )),
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
        worth_testkit_compiler::denied(root.join(
            "tests/worth_court_server_product_operation_phase_thirteen_shared_read/sources/worth_server/ui/construction/compat_http/private_request_constructor.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/worth_native/private_direct_lease_declaration_constructor.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/worth_native/private_direct_delivery_contract_constructor.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/private_request_context_constructor.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/private_lowered_operation_plan_constructor.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/private_lowered_operation_plan_handoff_access.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/private_scheduler_raw_input_access.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/private_surface_registration_constructor.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/private_server_constructor.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/product_operation_result_contract_required.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/construction/product_result_requires_schema_bound_value.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/cross_family/compat_http_cannot_import_worth_native_internal.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "../worth-testkit-server/assets/ui/cross_family/worth_native_cannot_import_compat_http_internal.rs",
        )),
        worth_testkit_compiler::denied(root.join(
            "tests/worth_court_server_product_operation_phase_thirteen_shared_read/sources/worth_server/ui/lifecycle/server_cannot_serve_twice.rs",
        )),
    ];
    let refusals = outcomes
        .into_iter()
        .filter_map(Result::err)
        .collect::<Vec<_>>();
    assert!(refusals.is_empty(), "compiler boundaries: {refusals:#?}");
}
