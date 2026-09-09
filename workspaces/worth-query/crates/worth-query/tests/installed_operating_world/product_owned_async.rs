use worth_query::facade::foundation::{
    WorthQueryAsyncFailurePosture, WorthQueryAsyncLoadingPosture,
    WorthQueryAsyncRequestIdentityPart, WorthQueryAsyncResourceRequestIdentity,
    WorthQueryAsyncSourceFamily,
};
use worth_query::facade::{domain, runtime};

use super::{conditional_node_contract, installed_operation_fixture};

fn async_identity() -> WorthQueryAsyncResourceRequestIdentity {
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

fn workspace() -> runtime::WorthQueryWorkspace {
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
    .workspace("selected-product-owned-async")
    .unwrap()
}

fn completion_envelope(
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

#[test]
fn selected_product_drives_owned_async_completion_and_definition_reuse() {
    let workspace = workspace();
    let world = workspace.observe_operating_world().unwrap();
    let selected = world.product_branch().unwrap();
    let identity = async_identity();
    let declaration = workspace
        .installed_owned_bridge_async_declaration(&identity)
        .expect("the builder-installed declaration must remain addressable after seal");
    let topology = workspace.owned_async_runtime_topology().unwrap();
    assert_eq!(topology.installed_async_declarations(), 1);

    let first = workspace
        .admit_owned_bridge_async_request(&declaration, selected)
        .expect("the selected product must admit the owned request");
    let request = first.request();
    let raw = completion_envelope(request);
    let completion = workspace
        .admit_owned_bridge_async_completion(&first, raw)
        .expect("completion must traverse the sealed owner service");
    assert!(completion.report().admitted_completion().is_some());
    let ordering = workspace
        .order_owned_bridge_async_completion(&completion)
        .expect("the admitted completion must enter mixed-cause ordering");
    assert_eq!(ordering.ordered().len(), 1);
    workspace
        .retire_owned_bridge_async_request(&first)
        .expect("cleanup after completion must be accepted");

    let second = workspace
        .admit_owned_bridge_async_request(&declaration, selected)
        .expect("retirement must leave the pre-sealed definition reusable");
    assert_ne!(
        first.request().request_handle(),
        second.request().request_handle()
    );
    workspace
        .retire_owned_bridge_async_request(&second)
        .unwrap();
    let after = workspace.owned_async_runtime_topology().unwrap();
    assert_eq!(after.installed_async_declarations(), 1);
    assert_eq!(after.active_signal_nodes(), topology.active_signal_nodes());
}

#[test]
fn configured_timeout_retry_completes_and_releases_the_exact_request() {
    let workspace = workspace();
    let world = workspace.observe_operating_world().unwrap();
    let selected = world.product_branch().unwrap();
    let declaration = workspace
        .installed_owned_bridge_async_declaration(&async_identity())
        .unwrap();
    let initial = workspace
        .admit_owned_bridge_async_request(&declaration, selected)
        .unwrap();

    let timeout = workspace
        .advance_owned_bridge_async_request_to_timeout(&initial, 5)
        .unwrap();
    assert!(timeout.report().timed_out_request().is_some());
    let schedule = workspace
        .schedule_owned_bridge_async_retry(&initial, &timeout)
        .unwrap();
    assert!(schedule.report().scheduled_retry().is_some());
    let retried = workspace
        .advance_owned_bridge_async_retry(&initial, &schedule, 7)
        .unwrap();
    assert_eq!(
        retried.lineage().class(),
        worth_runtime_bridge::facade::BridgeAsyncForwardCausalityClass::RetryAfterTimeout
    );
    assert_eq!(
        retried.request().request().attempt().get(),
        initial.request().attempt().get() + 1
    );

    let retried = retried.into_request();
    let completion = workspace
        .admit_owned_bridge_async_completion(&retried, completion_envelope(retried.request()))
        .unwrap();
    assert!(completion.report().admitted_completion().is_some());
    workspace
        .retire_owned_bridge_async_request(&retried)
        .unwrap();
    assert_eq!(
        workspace
            .owned_bridge_async_active_request_count(&retried)
            .unwrap(),
        0
    );
}

#[test]
fn sibling_product_revalidation_carries_truth_drift_and_releases_the_replacement() {
    let workspace = workspace();
    let source_world = workspace.observe_operating_world().unwrap();
    let source = source_world.product_branch().unwrap();
    let creation = runtime::ProductBranchCreationIntent::from_source(
        "owned-async-sibling",
        runtime::ProductBranchCreationPlans::new(
            runtime::RelationalBranchCreationPlan::ForkExact {
                target: runtime::BranchId("owned-async-sibling-truth".to_owned()),
            },
            runtime::SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .unwrap();
    let cancellation = runtime::RuntimeWorldCancellationSource::new();
    let outcome = workspace
        .create_product_branch(source, creation, &cancellation.token())
        .unwrap();
    let runtime::RuntimeWorldBranchCreationOutcome::Performed(observation) = outcome else {
        panic!("World must publish the sibling product occurrence: {outcome:?}")
    };
    let sibling_identity = observation.branch_identity().clone();
    drop(observation);
    let sibling_world = workspace
        .observe_product_operating_world(&sibling_identity)
        .unwrap();
    let sibling = sibling_world.product_branch().unwrap();
    let declaration = workspace
        .installed_owned_bridge_async_declaration(&async_identity())
        .unwrap();
    let initial = workspace
        .admit_owned_bridge_async_request(&declaration, source)
        .unwrap();

    let revalidated = workspace
        .revalidate_owned_bridge_async_request(&initial, sibling)
        .unwrap();
    assert_eq!(
        revalidated.lineage().class(),
        worth_runtime_bridge::facade::BridgeAsyncForwardCausalityClass::RevalidationAfterTruthBasisDrift
    );
    assert_ne!(
        revalidated.request().request().request_handle(),
        initial.request().request_handle()
    );
    assert_eq!(
        workspace
            .owned_bridge_async_active_request_count(revalidated.request())
            .unwrap(),
        1
    );

    let replacement = revalidated.into_request();
    workspace
        .retire_owned_bridge_async_request(&replacement)
        .unwrap();
    assert_eq!(
        workspace
            .owned_bridge_async_active_request_count(&replacement)
            .unwrap(),
        0
    );
}

#[test]
fn foreign_world_product_cannot_enter_the_owned_async_owner() {
    let origin = workspace();
    let foreign = workspace();
    let foreign_world = foreign.observe_operating_world().unwrap();
    let foreign_product = foreign_world.product_branch().unwrap();
    let declaration = origin
        .installed_owned_bridge_async_declaration(&async_identity())
        .unwrap();

    let denial = match origin.admit_owned_bridge_async_request(&declaration, foreign_product) {
        Err(denial) => denial,
        Ok(_) => panic!("a foreign World product entered the origin async owner"),
    };
    assert!(matches!(
        denial,
        runtime::WorthQueryOwnedAsyncRuntimeDenial::ProductBasisRequired
    ));
}

#[test]
fn stale_prior_occurrence_cannot_supersede_a_view_bound_to_its_same_source_successor() {
    let mut workspace = workspace();
    let world = workspace.observe_operating_world().unwrap();
    let selected = world.retain_product_branch().unwrap();
    drop(world);
    let declaration = workspace
        .installed_owned_bridge_async_declaration(&async_identity())
        .unwrap();
    let stale = workspace
        .admit_owned_bridge_async_request(&declaration, &selected)
        .unwrap();
    let bound = workspace
        .admit_owned_bridge_async_request(&declaration, &selected)
        .unwrap();
    let displacing = workspace
        .admit_owned_bridge_async_request(&declaration, &selected)
        .unwrap();
    let (request, schema) = owned_async_live_request_and_schema();
    let view: runtime::WorthQueryLiveView<runtime::WorthQueryUnrefinedLiveShape> = workspace
        .declare_bridge_async_live_view(
            "selected-product-owned-async-view",
            request,
            schema,
            bound.request(),
        )
        .unwrap();

    let denial = workspace
        .supersede_owned_bridge_async_live_view(&view, &stale, &displacing)
        .expect_err("the stale prior occurrence must not mutate the bound view");
    assert_eq!(
        denial.kind(),
        runtime::WorthQueryAsyncSourceBindingErrorKind::ForeignRequest
    );
    assert_eq!(
        workspace
            .state_live(&view)
            .unwrap()
            .async_result_state()
            .unwrap()
            .kind(),
        runtime::WorthQueryRuntimeAsyncResultStateKind::Pending
    );

    let admitted = workspace
        .supersede_owned_bridge_async_live_view(&view, &bound, &displacing)
        .expect("the exact bound occurrence admits supersession");
    assert_eq!(
        admitted.states()[0].kind(),
        runtime::WorthQueryRuntimeAsyncResultStateKind::Superseded
    );
    for request in [&stale, &bound, &displacing] {
        workspace
            .retire_owned_bridge_async_request(request)
            .unwrap();
    }
    workspace.close_owned_bridge_async_live_view(&view).unwrap();
}

#[test]
fn terminal_retries_require_the_exact_bound_occurrence() {
    assert_terminal_retry_is_occurrence_bound(false);
    assert_terminal_retry_is_occurrence_bound(true);
}

fn assert_terminal_retry_is_occurrence_bound(cancel: bool) {
    let mut workspace = workspace();
    let world = workspace.observe_operating_world().unwrap();
    let selected = world.retain_product_branch().unwrap();
    drop(world);
    let declaration = workspace
        .installed_owned_bridge_async_declaration(&async_identity())
        .unwrap();
    let bound = workspace
        .admit_owned_bridge_async_request(&declaration, &selected)
        .unwrap();
    let other = workspace
        .admit_owned_bridge_async_request(&declaration, &selected)
        .unwrap();
    let (request, schema) = owned_async_live_request_and_schema();
    let view: runtime::WorthQueryLiveView<runtime::WorthQueryUnrefinedLiveShape> = workspace
        .declare_bridge_async_live_view(
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

fn owned_async_live_request_and_schema() -> (
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
