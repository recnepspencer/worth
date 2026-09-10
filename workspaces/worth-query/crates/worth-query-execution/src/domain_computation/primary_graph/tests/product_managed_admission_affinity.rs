use std::collections::BTreeMap;

use worth_query_admission::facade::resource_admission::WorthQueryAdmittedWorkflowResourcePlan;
use worth_runtime_bridge::facade::SnapshotReadPacket;

use super::fixture::installed_authorization_world;
use crate::domain_computation::operation_binding::{
    authority_for_product, direct_authority, workflow_authority,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::provider_session::admitted_plan;
use crate::domain_computation::{
    WorthQueryManagedDirectRunAdmissionFailureKind, WorthQueryManagedTruthReadRequest,
    WorthQueryManagedWorkflowRunAdmissionFailureKind,
};

#[test]
fn direct_admission_rejects_a_same_relational_different_signal_product() {
    let world = installed_authorization_world(true);
    let (exact, substitute) = hostile_product_twins(&world.application);
    let plan = admitted_plan("direct-product-affinity", 8);
    let operation =
        authority_for_product(direct_authority(&world.application.runtime, &plan), &exact);
    let attempt = world
        .application
        .runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("the exact operation should reserve its resource attempt");
    let bridge = world
        .application
        .bridge
        .ordinary()
        .fork_managed_request_lane();

    let rejection = match world
        .application
        .runtime
        .managed_run_admission(&bridge, &world.application.product_runtime.source)
        .admit_direct(
            &operation,
            attempt,
            WorthQueryManagedTruthReadRequest::for_product(
                &substitute,
                SnapshotReadPacket::new(Vec::new()),
            ),
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
    let world = installed_authorization_world(true);
    let (exact, substitute) = hostile_product_twins(&world.application);
    let operation_plan = admitted_plan("workflow-product-affinity", 8);
    let stage_plan = admitted_plan("workflow-product-affinity:stage", 4);
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_plan,
        BTreeMap::from([("stage".to_owned(), stage_plan)]),
    );
    let operation = authority_for_product(
        workflow_authority(&world.application.runtime, &resources),
        &exact,
    );
    let attempt = world
        .application
        .runtime
        .start_workflow_resource_attempt(&operation, resources)
        .expect("the exact workflow should reserve its resource attempts");
    let bridge = world
        .application
        .bridge
        .ordinary()
        .fork_managed_request_lane();

    let rejection = match world
        .application
        .runtime
        .managed_run_admission(&bridge, &world.application.product_runtime.source)
        .admit_workflow(
            &operation,
            attempt,
            WorthQueryManagedTruthReadRequest::for_product(
                &substitute,
                SnapshotReadPacket::new(Vec::new()),
            ),
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

fn hostile_product_twins<Schema: worth_query_installation::facade::ApplicationSchema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
) -> (
    crate::basis::WorthQueryProductBranchLease,
    crate::basis::WorthQueryProductBranchLease,
) {
    let root = application.current_world();
    let exact = application
        .product_runtime
        .admit_product_occurrence(root.occurrence())
        .expect("the root product remains admitted");
    let sibling = application
        .branches()
        .fork(root)
        .components(|components| components.reuse_exact_relational_basis().fork_signal())
        .create()
        .expect("the hostile twin should reuse Relational and fork Signal");
    let substitute = application
        .product_runtime
        .admit_product_occurrence(sibling.occurrence())
        .expect("the hostile twin remains admitted");
    assert_eq!(
        exact.relational_basis_descriptor(),
        substitute.relational_basis_descriptor()
    );
    assert_ne!(exact.observation(), substitute.observation());
    (exact, substitute)
}
