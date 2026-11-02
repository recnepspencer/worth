//! Live selectors are fenced while recorded occurrence meaning remains usable.
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::*;
use worth_store_application::ApplicationHomeCloseOutcome;

fn open(
    path: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> worth_store_testkit_application_oracle::world_home::WorldHomeFixture {
    worth_store_testkit_application_oracle::world_home::open(
        worth_store_testkit_application_oracle::home_opening::request(
            path,
            crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        ),
        execution,
    )
    .unwrap()
}

fn close(
    home: worth_store_application::WorldReadmittedApplicationHome<(), (), (), (), ()>,
    execution: &ExecutionRequest<'_, '_>,
) {
    match home.close(execution) {
        Ok(ApplicationHomeCloseOutcome::Closed { .. }) => {}
        Ok(ApplicationHomeCloseOutcome::CloseRefused { reason, .. }) => {
            panic!("physical close: {reason:?}")
        }
        Err(cause) => panic!("World close: {:?}", cause.reason()),
    }
}

#[test]
fn reopened_world_refuses_old_live_selectors_and_readmits_original_durable_occurrence() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(directory.path(), &execution);
    let bootstrap = fixture.bootstrap_root(&execution);
    let original = bootstrap.product_branch();
    let branch = original.branch_identity().clone();
    let occurrence = original.lifecycle_incarnation();
    let owner = original.owner_identity();
    let description = ProductBranchReferenceDescription::from_observation(original);
    let old_incarnation = fixture.home.incarnation();
    drop(bootstrap);
    close(fixture.home, &execution);

    let fixture = open(directory.path(), &execution);
    assert_eq!(
        fixture.home.incarnation(),
        old_incarnation.successor().unwrap()
    );
    let observation = fixture.home.world().observation_port();
    assert!(matches!(
        observation.observe_product_branch(&branch),
        Err(RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchAdmissionDenial::StaleIncarnation
        ))
    ));
    assert!(matches!(
        observation.observe_product_branch_occurrence(occurrence),
        Err(RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchAdmissionDenial::StaleIncarnation
        ))
    ));
    let fresh = observation
        .readmit_product_branch_reference(&description)
        .unwrap();
    assert_eq!(fresh.branch_identity().name(), branch.name());
    assert_eq!(
        fresh.lifecycle_incarnation().ordinal(),
        occurrence.ordinal()
    );
    assert_eq!(fresh.owner_identity().durable(), owner.durable());
    assert_eq!(
        fresh.reference_generation().get(),
        description.reference_generation
    );
    assert_eq!(
        fresh.owner_identity().admit_incarnation(owner),
        Err(worth_proof::HomeIncarnationDenial::StaleIncarnation)
    );
    assert_eq!(
        fresh
            .owner_identity()
            .admit_incarnation(fresh.owner_identity()),
        Ok(())
    );
    drop(fresh);
    close(fixture.home, &execution);
}
