#[allow(unused_imports)]
pub(crate) use {
    crate::court::ordinary_context::branches::plans, crate::court::ordinary_context::relational,
    crate::court::ordinary_context::CompositeSupplyChainCourt,
    worth_relational::facade::history::BranchId, worth_runtime_world::facade::*,
};

#[test]
fn reuse_fork_matrix_retirement_and_aba_preserve_exact_owner_bindings() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for (relational, signal) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut court = crate::court::ordinary_context::CompositeSupplyChainCourt::compile();
        let root = court.bootstrap();
        let source_records = court
            .records
            .read(request_execution, root.basis().relational_basis());
        let signal_port = court
            .signal
            .owner_component_services()
            .unwrap()
            .basis_port();
        let before = signal_port.owner_service_cost_snapshot().unwrap();
        let token = RuntimeWorldCancellationSource::new().token();
        let branch_port = court.world.branch_port();
        let intent = ProductBranchCreationIntent::from_source(
            "child",
            plans(relational, signal, "component-child"),
        )
        .unwrap();
        let RuntimeWorldBranchCreationOutcome::Performed(child) = branch_port
            .create_product_branch(request_execution, root.clone(), intent, &token)
            .unwrap()
        else {
            panic!("healthy fork matrix")
        };
        assert_eq!(
            court
                .records
                .read(request_execution, child.basis().relational_basis()),
            source_records,
            "fork preserves semantic content"
        );
        assert_eq!(
            child.basis().relational_basis().admission_identity()
                == root.basis().relational_basis().admission_identity(),
            !relational
        );
        assert_eq!(
            child.basis().signal_basis().descriptor() == root.basis().signal_basis().descriptor(),
            !signal
        );
        assert_eq!(
            signal_port
                .owner_service_cost_snapshot()
                .unwrap()
                .fork_destination_installations()
                - before.fork_destination_installations(),
            u64::from(signal)
        );
        if relational {
            let identity = court
                .records
                .runtime
                .branch_identity(&BranchId("component-child".into()))
                .unwrap();
            assert_eq!(
                court
                    .records
                    .runtime
                    .observe_branch(&identity)
                    .unwrap()
                    .1
                    .admission_identity(),
                child.basis().relational_basis().admission_identity()
            );
        }
        let reference = signal_port
            .issue_managed_branch_reference(child.basis().signal_basis())
            .unwrap();
        assert_eq!(
            signal_port
                .observe_current(&reference)
                .unwrap()
                .descriptor(),
            child.basis().signal_basis().descriptor()
        );
        assert_eq!(
            court.observe(&root).selected_commit(),
            root.selected_commit(),
            "creation does not publish source"
        );
        let report = branch_port.retire_product_branch(&child).unwrap();
        assert_eq!(
            report.owner_retirement_work().len(),
            usize::from(relational) + usize::from(signal),
            "retirement reports exact fork custody for owner close"
        );
        assert!(court
            .world
            .observation_port()
            .observe_product_branch(child.branch_identity())
            .is_err());
        let RuntimeWorldBranchCreationOutcome::Performed(recreated) = branch_port
            .create_product_branch(
                request_execution,
                root.clone(),
                ProductBranchCreationIntent::from_source("child", plans(false, false, "unused"))
                    .unwrap(),
                &token,
            )
            .unwrap()
        else {
            panic!("recreate retired name")
        };
        assert_ne!(
            child.lifecycle_incarnation(),
            recreated.lifecycle_incarnation()
        );
        assert!(
            branch_port.retire_product_branch(&child).is_err(),
            "old incarnation cannot retire new child"
        );
        let receipt = court.publish_cargo(&recreated, "5");
        assert_ne!(
            receipt.new_product_head().selected_commit(),
            recreated.selected_commit(),
            "create then publish is separate"
        );
        drop((root, child, recreated, receipt, report, reference));
        court.finish();
    }
}
