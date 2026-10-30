pub(crate) use serde::Serialize;
pub(crate) use {
    worth_query::facade::foundation::WorthQueryAsyncFailurePosture,
    worth_query::facade::foundation::WorthQueryAsyncLoadingPosture,
    worth_query::facade::foundation::WorthQueryAsyncRequestIdentityPart,
    worth_query::facade::foundation::WorthQueryAsyncResourceRequestIdentity,
    worth_query::facade::foundation::WorthQueryAsyncSourceFamily,
};
pub(crate) use {
    worth_server::WorthServerExternalResourceBudget,
    worth_server::WorthServerExternalResourceExecutionBoundary,
    worth_server::WorthServerExternalResourceExecutionDenialCode,
    worth_server::WorthServerExternalResourceExecutionOutcome,
    worth_server::WorthServerExternalResourceIntent,
    worth_server::WorthServerExternalResourceTransport,
    worth_server::WorthServerExternalResourceTransportOutcome,
    worth_server::WorthServerExternalResourceTransportResponse,
    worth_server::WorthServerLoweredExternalResourcePlan,
};

pub(crate) fn admitted_result(
    evidence: &'static str,
    id: &'static str,
) -> worth_server::WorthServerAdmittedExternalResourceResult {
    let transport = RespondingTransport {
        body: br#"{"rows":[]}"#.to_vec(),
        evidence,
    };
    let boundary = worth_server::WorthServerExternalResourceExecutionBoundary::using(&transport);
    let plan = boundary.plan(intent(1024)).unwrap();
    let worth_server::WorthServerExternalResourceExecutionOutcome::Completed(completed) =
        boundary.execute(plan)
    else {
        panic!("transport should complete");
    };
    completed
        .admit_json_result(
            "test.external-resource.users.v1",
            1,
            &Rows {
                rows: vec![Row { id }],
            },
        )
        .unwrap()
}

pub(crate) fn intent(max_response_bytes: usize) -> WorthServerExternalResourceIntent {
    intent_with_body(max_response_bytes, br#"{"selector":"owner"}"#.to_vec())
}

pub(crate) fn intent_with_body(
    max_response_bytes: usize,
    request_body: Vec<u8>,
) -> WorthServerExternalResourceIntent {
    let request_identity =
        worth_query::facade::foundation::WorthQueryAsyncResourceRequestIdentity::declare(
            worth_query::facade::foundation::WorthQueryAsyncSourceFamily::HostResource,
            worth_query::facade::foundation::WorthQueryAsyncLoadingPosture::Blocking,
            worth_query::facade::foundation::WorthQueryAsyncFailurePosture::FailClosed,
            vec![
                worth_query::facade::foundation::WorthQueryAsyncRequestIdentityPart::text(
                    "source", "users",
                ),
                worth_query::facade::foundation::WorthQueryAsyncRequestIdentityPart::text(
                    "selector", "owner",
                ),
            ],
        )
        .unwrap();
    worth_server::WorthServerExternalResourceIntent::builder()
        .with_request_identity(request_identity)
        .with_provider_identity("test-host-provider")
        .with_contract_identity("test.users.v1")
        .with_basis_identity("basis:test")
        .with_request_body(request_body)
        .with_budget(
            worth_server::WorthServerExternalResourceBudget::bounded(
                1024,
                max_response_bytes,
                5_000,
            )
            .unwrap(),
        )
        .build()
        .unwrap()
}

#[derive(Debug)]
pub(crate) struct RespondingTransport {
    pub(crate) body: Vec<u8>,
    pub(crate) evidence: &'static str,
}

impl WorthServerExternalResourceTransport for RespondingTransport {
    fn execute(
        &self,
        _plan: &WorthServerLoweredExternalResourcePlan,
    ) -> WorthServerExternalResourceTransportOutcome {
        worth_server::WorthServerExternalResourceTransportOutcome::Responded(
            worth_server::WorthServerExternalResourceTransportResponse::new(
                self.body.clone(),
                self.evidence,
            )
            .unwrap(),
        )
    }
}

#[derive(Debug)]
pub(crate) struct FixedTransport(pub(crate) WorthServerExternalResourceTransportOutcome);

impl WorthServerExternalResourceTransport for FixedTransport {
    fn execute(
        &self,
        _plan: &WorthServerLoweredExternalResourcePlan,
    ) -> WorthServerExternalResourceTransportOutcome {
        self.0.clone()
    }
}

#[derive(Serialize)]
pub(crate) struct Rows<'a> {
    pub(crate) rows: Vec<Row<'a>>,
}

#[derive(Serialize)]
pub(crate) struct Row<'a> {
    pub(crate) id: &'a str,
}
