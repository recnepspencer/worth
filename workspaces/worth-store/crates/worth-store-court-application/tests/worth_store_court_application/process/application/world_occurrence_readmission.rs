//! Actual durable occurrences reconstruct original history, never live results.
use worth_execution::ExecutionRequest;
use worth_foundational::facade::DurableCompositeCommitIdentity;
use worth_runtime_world::facade::*;
use worth_store_application::{ApplicationHomeCheckpointDenial, ApplicationHomeCloseOutcome};

fn open(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> worth_store_testkit_application_oracle::world_home::WorldHomeFixture {
    worth_store_testkit_application_oracle::world_home::open(
        worth_store_testkit_application_oracle::home_opening::request(
            location,
            crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        ),
        execution,
    )
    .expect("actual filesystem Home readmission")
}
fn close(
    home: worth_store_application::WorldReadmittedApplicationHome<(), (), (), (), ()>,
    execution: &ExecutionRequest<'_, '_>,
) {
    match home.close(execution) {
        Ok(ApplicationHomeCloseOutcome::Closed {
            checkpoint: Err(ApplicationHomeCheckpointDenial::WorldSectionRequired),
            ..
        }) => {}
        Ok(ApplicationHomeCloseOutcome::CloseRefused { reason, .. }) => {
            panic!("actual Native close refused: {reason:?}")
        }
        Err(refused) => panic!("actual World close refused: {:?}", refused.reason()),
        _ => panic!("World checkpoint coverage remains explicitly unavailable"),
    }
}

#[test]
fn ordinary_occurrences_reopen_original_history_after_later_native_advance() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(directory.path(), &execution);
    let bootstrap = fixture.bootstrap_root(&execution);
    let identity = fixture.home.home_identity();
    let first_incarnation = fixture.home.incarnation();
    let mut expected = bootstrap.product_branch().clone();
    drop(bootstrap);
    let cancellation = RuntimeWorldCancellationSource::new();
    let mut originals = Vec::new();
    for round in 0..2 {
        let prepared = fixture.prepare_publication(
            &execution,
            expected.clone(),
            crate::batches::create(round),
            &cancellation.token(),
        );
        let performed = fixture.publish(&execution, prepared, &cancellation.token());
        let native = performed
            .component_results()
            .relational_commit_result()
            .unwrap();
        originals.push((
            DurableCompositeCommitIdentity::from_recorded(
                performed
                    .commit()
                    .identity()
                    .owner_identity()
                    .durable()
                    .unwrap(),
                performed.commit().identity().ordinal(),
            ),
            performed.attempt_identity().ordinal(),
            native.semantic_result().clone(),
            native.patch_position(),
        ));
        let mut consumed = performed.consume();
        expected = consumed.take_successor_observation().unwrap();
        drop(consumed);
    }
    let final_description = ProductBranchReferenceDescription::from_observation(&expected);
    drop(expected);
    // An independent ordinary Native caller advances beyond BOTH World roots.
    // Its real result snapshot is this caller's explicit release obligation.
    let native = fixture.home.relational().runtime();
    let later = crate::workload::commit(
        native,
        &execution,
        &native.main_branch_identity().branch_id().clone(),
        crate::batches::create(9),
    )
    .unwrap();
    native
        .snapshots()
        .release_snapshot(&later.snapshot)
        .unwrap();
    drop(later);
    close(fixture.home, &execution);

    let fixture = open(directory.path(), &execution);
    assert_eq!(fixture.home.home_identity(), identity);
    assert!(fixture.home.incarnation() > first_incarnation);
    let observation = fixture.home.world().observation_port();
    let current = observation.observe_root_product_branch().unwrap();
    assert_eq!(current.reference_generation().get(), 2);
    assert_eq!(
        current.branch_identity().name().as_str(),
        "Original World Root"
    );
    let reopened_description = ProductBranchReferenceDescription::from_observation(&current);
    assert_ne!(
        reopened_description.owner_ordinal,
        final_description.owner_ordinal
    );
    assert_eq!(
        reopened_description.home_owner,
        final_description.home_owner
    );
    assert_eq!(
        reopened_description.branch_name,
        final_description.branch_name
    );
    assert_eq!(
        reopened_description.lifecycle_ordinal,
        final_description.lifecycle_ordinal
    );
    assert_eq!(
        reopened_description.reference_generation,
        final_description.reference_generation
    );
    assert_eq!(
        reopened_description.composite_commit_ordinal,
        final_description.composite_commit_ordinal
    );
    for (index, (commit, attempt, semantic, position)) in originals.iter().enumerate() {
        let original = observation.readmitted_publication(commit).unwrap();
        assert_eq!(
            original.after().selected_commit().ordinal(),
            commit.ordinal()
        );
        assert_eq!(original.before().reference_generation().get(), index as u64);
        assert_eq!(
            original.after().reference_generation().get(),
            index as u64 + 1
        );
        assert_eq!(original.attempt_identity().ordinal(), *attempt);
        assert_eq!(original.semantic_result(), semantic);
        assert_eq!(original.native_patch_position(), *position);
        assert_eq!(original.journal_position(), index as u64 + 1);
        assert_eq!(original.admitted_incarnation(), fixture.home.incarnation());
        assert!(
            fixture
                .home
                .world()
                .recovery_port()
                .recover_performed(original.after().selected_commit())
                .is_err(),
            "recovered meaning never fabricates live delivery"
        );
    }
    drop(current);
    close(fixture.home, &execution);
    let fixture = open(directory.path(), &execution);
    let current = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let description = ProductBranchReferenceDescription::from_observation(&current);
    assert_ne!(
        description.owner_ordinal,
        reopened_description.owner_ordinal
    );
    assert_eq!(description.home_owner, final_description.home_owner);
    assert_eq!(description.branch_name, final_description.branch_name);
    assert_eq!(
        description.lifecycle_ordinal,
        final_description.lifecycle_ordinal
    );
    assert_eq!(
        description.reference_generation,
        final_description.reference_generation
    );
    assert_eq!(
        description.composite_commit_ordinal,
        final_description.composite_commit_ordinal
    );
    drop(current);
    close(fixture.home, &execution);
}
