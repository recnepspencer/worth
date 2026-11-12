//! Actual filesystem bootstrap and root/history/Bridge release through Home.
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::RuntimeWorldOwnerLifecycleObservation;
use worth_store_application::{
    ApplicationHomeCheckpointDenial, ApplicationHomeCloseDenial, ApplicationHomeCloseOutcome,
    ApplicationHomeClosePosture,
};
use worth_store_relational::RelationalSectionCloseDenial;
use worth_store_testkit_application_oracle::facade::home_opening::HomeOpeningRefused as OpenRefusal;

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
    let checkpoint = home
        .checkpoint(&execution)
        .unwrap_or_else(|cause| panic!("actual coupled World checkpoint: {cause:?}"));
    assert!(checkpoint.covered().0 > 0);
    let (_, external_read) = home
        .relational()
        .runtime()
        .observe_branch(&home.relational().runtime().main_branch_identity())
        .unwrap();
    let sealed = match home.seal_world(&execution) {
        Ok(sealed) => sealed,
        Err(refused) => panic!(
            "quiescent bootstrapped World must close: {:?}",
            refused.reason()
        ),
    };
    assert_eq!(sealed.home_identity(), identity);
    assert_eq!(sealed.incarnation(), incarnation);
    let _actual_world_report = sealed.world_close_report();
    let origin = sealed
        .checkpoint_origin_retirement()
        .expect("closed World explicitly retired its installed origin");
    assert_eq!(
        origin.retention_keys().count(),
        4,
        "the initial basis and original occurrence image pair both retire"
    );
    match crate::durable_run::open(directory.path(), &execution) {
        Err(OpenRefusal::BeforeReadmission(
            worth_store_application::ApplicationHomeOpenRefusal::Physical(refused),
        )) => {
            assert_eq!(
                refused.kind(),
                worth_store_application::ApplicationHomePhysicalRefusalKind::MutationOwnerContended
            );
        }
        _ => panic!("sealed World retains the SAME Home lease before Native teardown"),
    }
    let close = sealed.finish_close(&execution);
    let recovery = match close {
        ApplicationHomeCloseOutcome::CloseRefused {
            reason: ApplicationHomeCloseDenial::Relational(RelationalSectionCloseDenial::Native(
                worth_relational::facade::runtime::RelationalRuntimeCloseDenial::ReadCustodyRetained(_),
            )),
            posture: ApplicationHomeClosePosture::RecoveryRequired(recovery),
        } => recovery,
        _ => panic!("only actual external Native read custody remains; no Bridge Arc or fabricated serving posture"),
    };
    assert_eq!(recovery.home_identity(), identity);
    assert_eq!(recovery.native_active_snapshot_count(), Some(0),
        "actual World-owned result snapshots were released; the independent external basis still blocks close");
    assert!(recovery.world_close_report().is_some());
    assert!(
        recovery.checkpoint_result().is_ok(),
        "the actual coupled checkpoint settled"
    );
    assert_eq!(recovery.location(), directory.path());
    match crate::durable_run::open(directory.path(), &execution) {
        Err(OpenRefusal::BeforeReadmission(
            worth_store_application::ApplicationHomeOpenRefusal::Physical(refused),
        )) => {
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
            checkpoint: Ok(_), ..
        } => {}
        ApplicationHomeCloseOutcome::CloseRefused { reason, .. } => {
            panic!("same Native/Home close remains refused after external read release: {reason:?}")
        }
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Err(cause),
            ..
        } => {
            panic!("same Home must retain its successful World checkpoint: {cause:?}")
        }
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

#[test]
fn abandoned_world_delivery_refuses_before_close_and_can_be_delivered_then_closed() {
    use worth_runtime_world::facade::{RuntimeWorldCloseDenial, RuntimeWorldServiceDenial};
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let opening = worth_store_testkit_application_oracle::home_opening::request(
        directory.path(),
        crate::workload::runtime_builder(crate::schema_transition::SOURCE),
    );
    let fixture =
        worth_store_testkit_application_oracle::world_home::open(opening, &execution).unwrap();
    let bootstrap = fixture.bootstrap_root(&execution);
    let cancellation = worth_runtime_world::facade::RuntimeWorldCancellationSource::new();
    let prepared = fixture.prepare_publication(
        &execution,
        bootstrap.product_branch().clone(),
        crate::batches::create(0),
        &cancellation.token(),
    );
    let performed = fixture.publish(&execution, prepared, &cancellation.token());
    let selected = performed.new_product_head().selected_commit().clone();
    drop(performed);
    drop(bootstrap);
    let identity = fixture.home.home_identity();
    let refused = match fixture.home.close(&execution) {
        Err(refused) => refused,
        Ok(_) => panic!("undelivered actual publication must remain reachable before teardown"),
    };
    assert!(matches!(
        refused.reason(),
        ApplicationHomeCloseDenial::World(RuntimeWorldServiceDenial::Denied(
            RuntimeWorldCloseDenial::UndeliveredPublication
        ))
    ));
    assert_eq!(
        refused.posture(),
        worth_store_application::WorldHomeClosePosture::Serving
    );
    let home = refused
        .resume()
        .unwrap_or_else(|_| panic!("actual World remains open"));
    assert_eq!(home.home_identity(), identity);
    assert_eq!(
        home.world().lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Open
    );
    match crate::durable_run::open(directory.path(), &execution) {
        Err(OpenRefusal::BeforeReadmission(
            worth_store_application::ApplicationHomeOpenRefusal::Physical(refused),
        )) => assert_eq!(
            refused.kind(),
            worth_store_application::ApplicationHomePhysicalRefusalKind::MutationOwnerContended,
        ),
        _ => panic!("same Home keeps the actual mutation lease"),
    }
    let recovered = home
        .world()
        .recovery_port()
        .recover_performed(&selected)
        .unwrap();
    assert_eq!(recovered.new_product_head().reference_generation().get(), 1);
    assert_eq!(recovered.new_product_head().selected_commit(), &selected);
    drop(recovered.consume());
    match home.close(&execution) {
        Ok(ApplicationHomeCloseOutcome::Closed {
            checkpoint: Ok(_), ..
        }) => {}
        Err(refused) => panic!("delivered World close refused: {:?}", refused.reason()),
        Ok(ApplicationHomeCloseOutcome::CloseRefused { reason, .. }) => {
            panic!("Native close refused: {reason:?}")
        }
        Ok(ApplicationHomeCloseOutcome::Closed {
            checkpoint: Err(cause),
            ..
        }) => {
            panic!("close must retain the successful coupled World checkpoint report: {cause:?}")
        }
    }
}
