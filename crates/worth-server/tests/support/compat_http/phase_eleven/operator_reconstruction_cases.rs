use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::{WorthServerMiddlewareConfig, WorthServerResponseInput};

use crate::shared_support_compat_http_phase_ten_runtime::{
    build_phase_ten_server_with_workspace_provider, compat_read_execution_input,
    compat_read_success,
};
use crate::shared_support_query_handoff_runtime::ProfiledTestWorkspaceProvider;
use crate::shared_support_query_handoff_runtime::TestWorkspaceProvider;
use crate::shared_support_response_fixture::{
    middleware_mutation_denial, query_handoff_durable_denial, resolve_blank_principal_denial,
    test_server,
};

#[test]
fn compat_http_phase_eleven_classifies_admitted_and_denied_operations_from_evidence_artifacts_alone(
) {
    let server = test_server(
        TestWorkspaceProvider,
        WorthServerMiddlewareConfig::builder()
            .build()
            .expect("middleware config should validate"),
    );

    let request_context_response =
        server
            .responses()
            .shape_with_defaults(WorthServerResponseInput::request_context_denied(
                resolve_blank_principal_denial(&server),
            ));
    let middleware_response =
        server
            .responses()
            .shape_with_defaults(WorthServerResponseInput::middleware_denied(
                middleware_mutation_denial(&server),
            ));
    let query_handoff_response =
        server
            .responses()
            .shape_with_defaults(WorthServerResponseInput::query_handoff_denied(
                query_handoff_durable_denial(&server),
            ));

    let request_context_evidence =
        worth_server::WorthServerExternalEvidenceRecord::from_response_envelope(
            "compatibility_read",
            request_context_response,
            &server.operator_evidence(),
        )
        .expect("request context evidence should materialize");
    let middleware_evidence =
        worth_server::WorthServerExternalEvidenceRecord::from_response_envelope(
            "compatibility_read",
            middleware_response,
            &server.operator_evidence(),
        )
        .expect("middleware evidence should materialize");
    let query_handoff_evidence =
        worth_server::WorthServerExternalEvidenceRecord::from_response_envelope(
            "compatibility_read",
            query_handoff_response,
            &server.operator_evidence(),
        )
        .expect("query handoff evidence should materialize");
    let admitted_server =
        build_phase_ten_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));
    let admitted_read = compat_read_success(admitted_server.compat_http().read(
        compat_read_execution_input(
            &admitted_server,
            "tenant-a",
            "workspace-42",
            "branch-11",
            "users.profile",
            DiagnosticRichnessProfile::Forensic,
        ),
    ));
    let admitted_evidence =
        worth_server::WorthServerExternalEvidenceRecord::from_response_envelope(
            "compatibility_read",
            admitted_read.response_envelope().clone(),
            &admitted_server.operator_evidence(),
        )
        .expect("admitted evidence should materialize");

    assert_eq!(
        request_context_evidence.classification_label(),
        "compatibility_read_request_context_denied"
    );
    assert_eq!(
        middleware_evidence.classification_label(),
        "compatibility_read_middleware_denied"
    );
    assert_eq!(
        query_handoff_evidence.classification_label(),
        "compatibility_read_query_handoff_denied"
    );
    assert_eq!(
        admitted_evidence.classification_label(),
        "compatibility_read_succeeded"
    );

    assert_eq!(
        request_context_evidence
            .operator_record()
            .counter_receipt()
            .counter("response.request_context_denial.count")
            .expect("request context denial counter")
            .exact_value(),
        1
    );
    assert_eq!(
        middleware_evidence
            .operator_record()
            .counter_receipt()
            .counter("response.middleware_denial.count")
            .expect("middleware denial counter")
            .exact_value(),
        1
    );
    assert_eq!(
        query_handoff_evidence
            .operator_record()
            .counter_receipt()
            .counter("response.query_handoff_denial.count")
            .expect("query handoff denial counter")
            .exact_value(),
        1
    );
    assert_eq!(
        admitted_evidence
            .operator_record()
            .counter_receipt()
            .counter("response.query_read_success.count")
            .expect("query read success counter")
            .exact_value(),
        1
    );
}
