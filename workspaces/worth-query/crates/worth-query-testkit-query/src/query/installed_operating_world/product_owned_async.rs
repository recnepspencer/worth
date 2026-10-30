pub use worth_query::facade::foundation::{
    WorthQueryAsyncFailurePosture, WorthQueryAsyncLoadingPosture,
    WorthQueryAsyncRequestIdentityPart, WorthQueryAsyncResourceRequestIdentity,
    WorthQueryAsyncSourceFamily,
};
pub use worth_query::facade::{domain, runtime};

pub use super::{conditional_node_contract, installed_operation_fixture};

pub fn async_identity() -> WorthQueryAsyncResourceRequestIdentity {
    WorthQueryAsyncResourceRequestIdentity::declare(
        WorthQueryAsyncSourceFamily::HostResource,
        WorthQueryAsyncLoadingPosture::Blocking,
        WorthQueryAsyncFailurePosture::FailClosed,
        vec![WorthQueryAsyncRequestIdentityPart::text(
            "source",
            "selected-product-profile",
        )],
    )
    .unwrap()
}

pub fn workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> runtime::WorthQueryWorkspace {
    let node = conditional_node_contract::node(
        "geometry",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let installation = installed_operation_fixture::conditional_installation(&node);
    let declaration = runtime::WorthQueryOwnedAsyncRequestDeclaration::from_async_resource_identity(
        async_identity(),
        919,
        512,
        2,
        2,
        5,
    );
    installed_operation_fixture::conditional_workspace_with_builder(
        node,
        installation,
        installed_operation_fixture::DirectConditionalCompute,
    )
    .owned_bridge_async_declaration(declaration)
    .workspace(request_execution, "selected-product-owned-async")
    .unwrap()
}

pub fn completion_envelope(
    request: &worth_runtime_bridge::facade::AdmittedBridgeAsyncRequestIdentity,
) -> worth_signal::facade::RawCompletionEnvelope {
    worth_signal::facade::RawCompletionEnvelope::new(
        request.request_handle().request_id(),
        request.request_handle().generation(),
        request.request_handle().branch_epoch(),
        request.attempt(),
        request
            .lowered()
            .resource_descriptor()
            .unwrap()
            .payload_contract_digest()
            .clone(),
        64,
    )
}

#[path = "product_owned_async/stale_prior_occurrence_cannot_supersede_a_view_bound_to_its_same.rs"]
pub mod stale_prior_occurrence_cannot_supersede_a_view_bound_to_its_same;

pub fn assert_terminal_retry_is_occurrence_bound(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    cancel: bool,
) {
    let mut workspace = workspace(request_execution);
    let world = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap();
    let selected = world.retain_product_branch();
    drop(world);
    let declaration = workspace
        .installed_owned_bridge_async_declaration(&async_identity())
        .unwrap();
    let bound_custody = workspace
        .admit_owned_bridge_async_request(&declaration, &selected)
        .unwrap();
    let bound = bound_custody.retained_witness().unwrap();
    let other_custody = workspace
        .admit_owned_bridge_async_request(&declaration, &selected)
        .unwrap();
    let other = other_custody.retained_witness().unwrap();
    let (request, schema) = owned_async_live_request_and_schema();
    let view: runtime::WorthQueryLiveView<runtime::WorthQueryUnrefinedLiveShape> = workspace
        .declare_bridge_async_live_view(
            request_execution,
            "terminal-occurrence-bound-view",
            request,
            schema,
            bound.request(),
        )
        .unwrap();

    let first = if cancel {
        workspace.cancel_owned_bridge_async_live_view(&view, &bound)
    } else {
        workspace.deny_owned_bridge_async_live_view(&view, &bound)
    }
    .unwrap();
    let denial = if cancel {
        workspace.cancel_owned_bridge_async_live_view(&view, &other)
    } else {
        workspace.deny_owned_bridge_async_live_view(&view, &other)
    }
    .expect_err("a different admitted occurrence must not receive the terminal batch");
    assert_eq!(
        denial.kind(),
        runtime::WorthQueryAsyncSourceBindingErrorKind::ForeignRequest
    );
    let duplicate = if cancel {
        workspace.cancel_owned_bridge_async_live_view(&view, &bound)
    } else {
        workspace.deny_owned_bridge_async_live_view(&view, &bound)
    }
    .expect("the exact occurrence may repeat its terminal transition");
    assert_eq!(duplicate.states(), first.states());

    for admitted in [&bound, &other] {
        workspace
            .retire_owned_bridge_async_request(admitted)
            .unwrap();
    }
    workspace.close_owned_bridge_async_live_view(&view).unwrap();
}

pub fn owned_async_live_request_and_schema() -> (
    worth_query::facade::foundation::DeclarativeLiveQueryRequest,
    runtime::QuerySchemaView,
) {
    let request = worth_query::facade::foundation::DeclarativeLiveQueryRequest::new(
        "Vertex",
        worth_query::facade::foundation::DeclarativeLiveViewShape::table(),
    )
    .project(
        worth_query::facade::foundation::DeclarativeProjectionField::new(
            worth_query::facade::foundation::AspectFieldKey::from_authoring_parts("identity", "id")
                .unwrap(),
        )
        .delivered_as("identity.id"),
    );
    let schema = runtime::QuerySchemaView::new(
        "selected-product-owned-async-view-v1",
        [runtime::SchemaFieldView::new(
            worth_query::facade::foundation::AspectName::new("identity").unwrap(),
            worth_query::facade::foundation::FieldName::new("id").unwrap(),
            runtime::ScalarAspectType::String,
        )],
        [],
    );
    (request, schema)
}
