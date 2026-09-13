use super::world::empty_runtime;
use worth_relational::facade::branch::{
    RelationalBranchBasisPort, RelationalBranchLifecyclePort, RelationalOwnerServicePorts,
};
use worth_relational::facade::mvcc::{RelationalPreparationPort, RelationalPublicationPort};

#[test]
fn six_concrete_accessors_are_cloneable_thread_safe_services() {
    fn assert_service<T: Clone + Send + Sync>(_: &T) {}

    let runtime = empty_runtime();
    let services = runtime.owner_component_services();
    assert_service::<RelationalOwnerServicePorts>(&services);
    assert_service::<RelationalPreparationPort>(&services.preparation_port());
    assert_service(&services.fork_port());
    assert_service::<RelationalPublicationPort>(&services.publication_port());
    assert_service(&services.settlement_port());
    assert_service::<RelationalBranchBasisPort>(&services.basis_port());
    assert_service::<RelationalBranchLifecyclePort>(&services.lifecycle_port());

    let basis_port = services.basis_port();
    let identity = runtime.main_branch_identity();
    let worker = std::thread::spawn(move || {
        basis_port
            .observe_branch(&identity)
            .expect("a moved clone reaches the live owner")
            .0
    });
    assert_eq!(
        worker
            .join()
            .expect("service worker does not panic")
            .branch_id(),
        runtime.main_branch_identity().branch_id()
    );
}
