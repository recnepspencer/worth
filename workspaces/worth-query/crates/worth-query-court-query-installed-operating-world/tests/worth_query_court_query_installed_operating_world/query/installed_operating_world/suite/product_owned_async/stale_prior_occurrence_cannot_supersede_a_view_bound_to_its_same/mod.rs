pub(crate) use worth_query_testkit_query::facade::query::suite::product_owned_async::stale_prior_occurrence_cannot_supersede_a_view_bound_to_its_same::*;

#[test]
fn stale_prior_occurrence_cannot_supersede_a_view_bound_to_its_same_source_successor() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(request_execution);
    let world = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap();
    let selected = world.retain_product_branch();
    drop(world);
    let declaration = workspace
        .installed_owned_bridge_async_declaration(&async_identity())
        .unwrap();
    let stale_custody = workspace
        .admit_owned_bridge_async_request(&declaration, &selected)
        .unwrap();
    let stale = stale_custody.retained_witness().unwrap();
    let bound_custody = workspace
        .admit_owned_bridge_async_request(&declaration, &selected)
        .unwrap();
    let bound = bound_custody.retained_witness().unwrap();
    let displacing_custody = workspace
        .admit_owned_bridge_async_request(&declaration, &selected)
        .unwrap();
    let displacing = displacing_custody.retained_witness().unwrap();
    let (request, schema) = owned_async_live_request_and_schema();
    let view: runtime::WorthQueryLiveView<runtime::WorthQueryUnrefinedLiveShape> = workspace
        .declare_bridge_async_live_view(
            request_execution,
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
