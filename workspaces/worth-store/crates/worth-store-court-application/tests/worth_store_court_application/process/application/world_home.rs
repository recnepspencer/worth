//! Actual filesystem bootstrap and root/history/Bridge release through Home.
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::RuntimeWorldOwnerLifecycleObservation;
use worth_store_application::{
    ApplicationHomeCheckpointDenial, ApplicationHomeCloseDenial, ApplicationHomeCloseOutcome,
    ApplicationHomeClosePosture,
};
use worth_store_relational::RelationalSectionCloseDenial;

#[test]
fn closed_world_releases_its_bridge_and_retains_the_same_native_home_until_external_reads_release()
{
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let opening = worth_store_testkit_application_oracle::home_opening::request(
        directory.path(),
        crate::workload::runtime_builder(crate::schema_transition::SOURCE),
    );
    let fixture = worth_store_testkit_application_oracle::world_home::open(opening, &execution)
        .expect("actual filesystem Home composes the same Native owner and Bridge");
    let bootstrap = fixture.bootstrap_root(&execution);
    assert_eq!(
        bootstrap.product_branch().branch_identity().name().as_str(),
        "Original World Root"
    );
    assert_eq!(bootstrap.product_branch().reference_generation().get(), 0);
    let cancellation = worth_runtime_world::facade::RuntimeWorldCancellationSource::new();
    let prepared = fixture.prepare_publication(
        &execution,
        bootstrap.product_branch().clone(),
        crate::batches::create(0),
        &cancellation.token(),
    );
    let performed = fixture.publish(&execution, prepared, &cancellation.token());
    assert_eq!(
        performed.old_product_head().selected_commit(),
        bootstrap.product_branch().selected_commit()
    );
    assert_eq!(performed.new_product_head().reference_generation().get(), 1);
    assert_eq!(
        performed
            .new_product_head()
            .branch_identity()
            .name()
            .as_str(),
        "Original World Root"
    );
    // This is the actual product handoff. Dropping an unconsumed performed
    // claim deliberately leaves its successor observation owed to delivery.
    drop(performed.consume());
    // Only the World retains these root/history pins after this external
    // bootstrap observation is dropped. An additional Native read below
    // separately exercises genuine external custody.
    drop(bootstrap);
    let home = fixture.home;
    let identity = home.home_identity();
    let incarnation = home.incarnation();
    let publication = home.world().publication_port();
    let observation = home.world().observation_port();
    let inspection = home.world().inspection_port();
    assert_eq!(
        home.world().lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Open
    );
    assert!(matches!(
        home.checkpoint(&execution),
        Err(ApplicationHomeCheckpointDenial::WorldSectionRequired)
    ));
    let (_, external_read) = home
        .relational()
        .runtime()
        .observe_branch(&home.relational().runtime().main_branch_identity())
        .unwrap();
    let close = match home.close(&execution) {
        Ok(close) => close,
        Err(refused) => panic!(
            "quiescent bootstrapped World must close: {:?}",
            refused.reason()
        ),
    };
    let recovery = match close {
        ApplicationHomeCloseOutcome::CloseRefused {
            reason: ApplicationHomeCloseDenial::Relational(RelationalSectionCloseDenial::Native(
                worth_relational::facade::runtime::RelationalRuntimeCloseDenial::ReadCustodyRetained,
            )),
            posture: ApplicationHomeClosePosture::RecoveryRequired(recovery),
        } => recovery,
        _ => panic!("only actual external Native read custody remains; no Bridge Arc or fabricated serving posture"),
    };
    assert_eq!(recovery.home_identity(), identity);
    assert!(recovery.world_close_report().is_some());
    assert!(matches!(
        recovery.checkpoint_result(),
        Err(ApplicationHomeCheckpointDenial::WorldSectionRequired)
    ));
    assert_eq!(recovery.location(), directory.path());
    match crate::durable_run::open(directory.path(), &execution) {
        Err(worth_store_application::ApplicationHomeOpenRefusal::Physical(refused)) => {
            assert_eq!(
                refused.kind(),
                worth_store_application::ApplicationHomePhysicalRefusalKind::MutationOwnerContended
            );
        }
        _ => panic!("the retained Home's actual mutation lease refuses a competing open"),
    }
    // Keeping these real weak ports alive must not keep World's Bridge/Native Arc alive.
    let inert_ports = (publication, observation, inspection);
    drop(external_read);
    match recovery.try_release(&execution) {
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Err(ApplicationHomeCheckpointDenial::WorldSectionRequired),
            ..
        } => {}
        ApplicationHomeCloseOutcome::CloseRefused { reason, .. } => {
            panic!("same Native/Home close remains refused after external read release: {reason:?}")
        }
        _ => panic!("same Home close must report its World checkpoint refusal"),
    }
    drop(inert_ports);
    let reopened = crate::durable_run::open(directory.path(), &execution).unwrap();
    assert_eq!(reopened.home_identity(), identity);
    assert!(reopened.incarnation() > incarnation);
    assert!(matches!(
        reopened.checkpoint(&execution),
        Err(ApplicationHomeCheckpointDenial::WorldSectionRequired)
    ));
    assert!(matches!(
        reopened.close(&execution),
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Err(ApplicationHomeCheckpointDenial::WorldSectionRequired),
            ..
        }
    ));
}
