use crate::signal_owner_services::ordinary_context::execution_custody;
#[allow(unused_imports)]
pub(crate) use {
    crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld,
    std::panic::catch_unwind, std::panic::AssertUnwindSafe,
    worth_signal::facade::branch::SignalOwnerCancellationSource,
    worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest,
};

#[test]
fn fork_post_capture_fault_preserves_performed_child_and_source_health() {
    for boundary in [
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::ForkDestinationInstallation,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::OutcomeConstruction,
    ] {
        let world = crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
        let runtime = world
            .runtime
            .as_ref()
            .expect("the strong root remains live");
        let control = runtime
            .owner_operation_control_for_test()
            .expect("the control handle comes from the sealed owner");
        let name = "performed-fork";
        control.inject_panic_once_for_test(boundary);
        let fault = catch_unwind(AssertUnwindSafe(|| {
            let _ = world.mutation.fork_exact(
                worth_signal::facade::branch::validate_signal_branch_name(name)
                    .expect("the fork name is valid"),
                &world.root_basis,
                &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
            );
        }));
        assert!(fault.is_err(), "the post-capture fault reaches the caller");

        let performed = runtime
            .known_branches()
            .into_iter()
            .find(|branch| branch.name == name)
            .expect("post-linearization panic preserves the canonical child");
        assert_eq!(
            performed.parent_branch_id,
            Some(world.root_basis.branch_id())
        );
        let performed_basis = runtime
            .observe_signal_branch_basis(performed.clone())
            .expect("the performed child is recoverable as exact owner authority");
        assert_eq!(performed_basis.branch_id(), performed.id);
        world
            .mutation
            .advance_exact(
                worth_execution::ExecutionRequest::serial(
                    &crate::signal_owner_services::ordinary_context::execution_custody::operational_serial_request(),
                ),
                &world.root_basis,
                &mut (),
                &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
                |_| Ok(()),
            )
            .expect("post-capture destination work releases the healthy source");
    }
}
