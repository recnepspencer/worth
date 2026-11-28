use super::*;
use worth_relational::facade::mvcc::RelationalTransactionIntent;

#[test]
fn first_fork_publication_observes_its_recorded_native_before_component() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let RuntimeWorldBranchCreationOutcome::Performed(child) =
        create(&f, &request, &parent, "original", true, false)
    else {
        panic!("actual Native fork");
    };
    let native_before = f
        .native
        .owner_component_services()
        .basis_port()
        .export_recorded_retained_component(&request, child.basis().relational_basis())
        .unwrap();
    let cancellation = RuntimeWorldCancellationSource::new().token();
    let port = f.world.publication_port();
    let WorldOpenAttemptOutcome::Opened(opened) = port.open_publication_without_signal(
        &request,
        child.clone(),
        f.recorder.program_source(&child),
        RelationalTransactionIntent::ordinary(),
        &cancellation,
        None,
    ) else {
        panic!("actual durable open on the fork");
    };
    let mut transaction = f
        .native
        .begin_branch_transaction(
            child.basis().relational_basis(),
            RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            worth_testkit_relational::durable_publication::batch_create("on the fork"),
            request.allocation_policy(),
        )
        .unwrap();
    let proposal = transaction.validate(&request, &f.native).unwrap();
    let prepared = opened
        .prepare_native(&f.native, &request, proposal)
        .unwrap();
    let WorldDurablePublicationOutcome::Performed(performed) =
        port.execute_durable_without_signal(request, prepared, &cancellation)
    else {
        panic!("actual publication on the fork");
    };
    let authority = f.native.durability_authority();
    let receipt = performed
        .component_results()
        .relational_commit_result()
        .unwrap()
        .semantic_result()
        .receipt();
    let subject = authority
        .discover_associated_effect(&request, receipt)
        .unwrap()
        .unwrap();
    let effect = authority
        .readmit_associated_effect(&request, &subject)
        .unwrap();
    let observed = f
        .world
        .observation_port()
        .observe_home_original_publication_with_before_component(&request, effect)
        .unwrap();
    assert!(observed.before().generation().get() > 0);
    assert_eq!(
        observed.before().generation().get(),
        child.reference_generation().get()
    );
    assert_eq!(observed.recorded_before_component(), Some(&native_before));
    drop((observed, performed.consume(), child, parent));
    f.finish(&request);
}
