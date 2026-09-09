use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world;

#[test]
fn primary_product_selection_composes_one_exact_application_snapshot() {
    let world = installed_authorization_world(true);
    let lease = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let observer = world.application.application_query_basis_observer();
    let before = observer.observe();
    let expected = lease.relational_basis_descriptor().clone();
    let selected = world.application.on_product(lease).unwrap();
    assert_eq!(
        selected.application_basis().identity().descriptor(),
        &expected
    );
    assert_eq!(observer.observe().acquisitions(), before.acquisitions() + 1);
    assert!(world
        .application
        .primary_provider
        .graph
        .with_runtime(|runtime| runtime
            .read_truth()
            .project_snapshot(selected.application_basis().snapshot_handle())
            .is_some()));
    drop(selected);
    assert_eq!(observer.observe().active(), before.active());
}
