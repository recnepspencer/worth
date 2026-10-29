pub(crate) use worth_query_worlds::facade::query::suite::product_owned_async::*;

pub(crate) mod stale_prior_occurrence_cannot_supersede_a_view_bound_to_its_same;

#[test]
fn selected_product_drives_owned_async_completion_and_definition_reuse() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = workspace(request_execution);
    let world = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap();
    let selected = world.product_branch();
    let identity = async_identity();
    let declaration = workspace
        .installed_owned_bridge_async_declaration(&identity)
        .expect("the builder-installed declaration must remain addressable after seal");
    let topology = workspace.owned_async_runtime_topology().unwrap();
    assert_eq!(topology.installed_async_declarations(), 1);

    let first_custody = workspace
        .admit_owned_bridge_async_request(&declaration, selected)
        .expect("the selected product must admit the owned request");
    let first = first_custody.retained_witness().unwrap();
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

    let second_custody = workspace
        .admit_owned_bridge_async_request(&declaration, selected)
        .expect("retirement must leave the pre-sealed definition reusable");
    let second = second_custody.retained_witness().unwrap();
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
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = workspace(request_execution);
    let world = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap();
    let selected = world.product_branch();
    let declaration = workspace
        .installed_owned_bridge_async_declaration(&async_identity())
        .unwrap();
    let mut initial = workspace
        .admit_owned_bridge_async_request(&declaration, selected)
        .unwrap();

    let prior = initial.retained_witness().unwrap();
    let prior_attempt = prior.request().attempt();
    let timeout = workspace
        .advance_owned_bridge_async_request_to_timeout(&mut initial, 5)
        .unwrap();
    assert!(timeout.report().timed_out_request().is_some());
    let schedule = workspace
        .schedule_owned_bridge_async_retry(&mut initial, &timeout)
        .unwrap();
    assert!(schedule.report().scheduled_retry().is_some());
    let retried = workspace
        .advance_owned_bridge_async_retry(&mut initial, &schedule, 7)
        .unwrap();
    assert_eq!(
        retried.class(),
        worth_runtime_bridge::facade::BridgeAsyncForwardCausalityClass::RetryAfterTimeout
    );
    assert_eq!(
        initial.witness().unwrap().request().attempt().get(),
        prior.request().attempt().get() + 1
    );

    let retried = initial.retained_witness().unwrap();
    assert_eq!(
        prior.request().attempt(),
        prior_attempt,
        "the prior receipt stays immutable"
    );
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
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = workspace(request_execution);
    let source_world = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap();
    let source = source_world.product_branch();
    let sibling_branch = workspace
        .with_advancement(|phase| {
            workspace
                .branches()
                .fork(source.product_branch())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
        })
        .expect("installed host call opens")
        .expect("World must publish the sibling product occurrence");
    let sibling_world = workspace.observe_operating_world(sibling_branch).unwrap();
    let sibling = sibling_world.product_branch();
    let declaration = workspace
        .installed_owned_bridge_async_declaration(&async_identity())
        .unwrap();
    let mut initial = workspace
        .admit_owned_bridge_async_request(&declaration, source)
        .unwrap();

    let prior = initial.retained_witness().unwrap();
    let revalidated = workspace
        .revalidate_owned_bridge_async_request(&mut initial, sibling)
        .unwrap();
    let worth_runtime_bridge::facade::BridgeOwnedAsyncManagedRevalidation::Replaced(lineage) =
        revalidated
    else {
        panic!("truth drift must replace the occurrence")
    };
    let replacement = initial.retained_witness().unwrap();
    assert_eq!(
        lineage.class(),
        worth_runtime_bridge::facade::BridgeAsyncForwardCausalityClass::RevalidationAfterTruthBasisDrift
    );
    assert_ne!(
        replacement.request().request_handle(),
        prior.request().request_handle()
    );
    assert_eq!(
        workspace
            .owned_bridge_async_active_request_count(&replacement)
            .unwrap(),
        1
    );

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
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let origin = workspace(request_execution);
    let foreign = workspace(request_execution);
    let foreign_world = foreign
        .observe_operating_world(foreign.current_world())
        .unwrap();
    let foreign_product = foreign_world.product_branch();
    let declaration = origin
        .installed_owned_bridge_async_declaration(&async_identity())
        .unwrap();

    let denial = match origin.admit_owned_bridge_async_request(&declaration, foreign_product) {
        Err(denial) => denial,
        Ok(_) => panic!("a foreign World product entered the origin async owner"),
    };
    let (cause, custody) = denial.into_parts();
    assert!(
        custody.is_none(),
        "foreign selection refuses before native admission"
    );
    assert!(matches!(
        cause,
        runtime::WorthQueryOwnedAsyncRuntimeCause::Admission(
            runtime::WorthQueryOwnedAsyncRuntimeDenial::ProductSelectionMismatch
        )
    ));
}

#[test]
fn terminal_retries_require_the_exact_bound_occurrence() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_terminal_retry_is_occurrence_bound(request_execution, false);
    assert_terminal_retry_is_occurrence_bound(request_execution, true);
}
