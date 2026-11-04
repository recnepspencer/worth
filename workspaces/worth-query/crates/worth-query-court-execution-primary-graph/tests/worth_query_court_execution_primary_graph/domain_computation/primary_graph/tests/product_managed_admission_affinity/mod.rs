use std::collections::BTreeMap;

use worth_query_admission::facade::resource_admission::WorthQueryAdmittedWorkflowResourcePlan;
use worth_runtime_bridge::facade::SnapshotReadPacket;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::authority_for_product;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::direct_authority;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::workflow_authority;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::admitted_plan;
use worth_query_execution::facade::domain_computation::WorthQueryManagedDirectRunAdmissionFailureKind;
use worth_query_execution::facade::domain_computation::WorthQueryManagedTruthReadRequest;
use worth_query_execution::facade::domain_computation::WorthQueryManagedWorkflowRunAdmissionFailureKind;

#[test]
fn direct_admission_rejects_a_same_relational_different_signal_product() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (exact, substitute) = world
        .application
        .with_host_advancement(|phase| hostile_product_twins(&phase, &world.application))
        .expect("the fixture read owns its installed host call");
    let plan = admitted_plan("direct-product-affinity", 8);
    let operation = authority_for_product(
        direct_authority(world.application.runtime.execution(), &plan),
        &exact,
    );
    let attempt = world
        .application
        .runtime
        .execution()
        .start_direct_resource_attempt(&operation, plan)
        .expect("the exact operation should reserve its resource attempt");
    let bridge = world
        .application
        .runtime
        .bridge
        .ordinary()
        .fork_managed_request_lane();

    let rejection = match world
        .application
        .runtime
        .execution()
        .managed_run_admission(&bridge, &world.application.runtime.product_runtime.source)
        .admit_direct(
            &operation,
            attempt,
            WorthQueryManagedTruthReadRequest::for_product(
                &substitute,
                SnapshotReadPacket::new(Vec::new()),
            ),
            resource_request,
        ) {
        Ok(_) => panic!("a different Signal occurrence reached lower admission"),
        Err(rejection) => rejection,
    };

    assert_eq!(
        rejection.kind(),
        WorthQueryManagedDirectRunAdmissionFailureKind::ProductBasisMismatch
    );
    assert_eq!(
        rejection.release().capacity().released_reservation_count(),
        1
    );
}

#[test]
fn workflow_admission_rejects_a_same_relational_different_signal_product() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (exact, substitute) = world
        .application
        .with_host_advancement(|phase| hostile_product_twins(&phase, &world.application))
        .expect("the fixture read owns its installed host call");
    let operation_plan = admitted_plan("workflow-product-affinity", 8);
    let stage_plan = admitted_plan("workflow-product-affinity:stage", 4);
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_plan,
        BTreeMap::from([("stage".to_owned(), stage_plan)]),
    );
    let operation = authority_for_product(
        workflow_authority(world.application.runtime.execution(), &resources),
        &exact,
    );
    let attempt = world
        .application
        .runtime
        .execution()
        .start_workflow_resource_attempt(&operation, resources)
        .expect("the exact workflow should reserve its resource attempts");
    let bridge = world
        .application
        .runtime
        .bridge
        .ordinary()
        .fork_managed_request_lane();

    let rejection = match world
        .application
        .runtime
        .execution()
        .managed_run_admission(&bridge, &world.application.runtime.product_runtime.source)
        .admit_workflow(
            &operation,
            attempt,
            WorthQueryManagedTruthReadRequest::for_product(
                &substitute,
                SnapshotReadPacket::new(Vec::new()),
            ),
            resource_request,
        ) {
        Ok(_) => panic!("a different Signal occurrence reached lower admission"),
        Err(rejection) => rejection,
    };

    assert_eq!(
        rejection.kind(),
        WorthQueryManagedWorkflowRunAdmissionFailureKind::ProductBasisMismatch
    );
    assert_eq!(
        rejection.release().capacity().released_reservation_count(),
        2
    );
}

pub fn hostile_product_twins<Schema: worth_query_installation::facade::ApplicationSchema>(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryProductBranchLease,
    worth_query_execution::facade::domain_computation::WorthQueryProductBranchLease,
) {
    let root = application.current_world();
    let exact = application
        .runtime
        .product_runtime
        .admit_product_occurrence(root.occurrence())
        .expect("the root product remains admitted");
    let sibling = application
        .branches()
        .fork(root)
        .components(|components| components.reuse_exact_relational_basis().fork_signal())
        .create(phase_custody)
        .expect("the hostile twin should reuse Relational and fork Signal");
    let substitute = application
        .runtime
        .product_runtime
        .admit_product_occurrence(sibling.occurrence())
        .expect("the hostile twin remains admitted");
    assert_eq!(
        exact.relational_basis_descriptor(),
        substitute.relational_basis_descriptor()
    );
    assert_ne!(
        exact.observation_for_test(),
        substitute.observation_for_test()
    );
    (exact, substitute)
}
