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
    // Copy every public field of pure original meaning, retaining no live
    // result, Native owner or source custody across the filesystem restart.
    let semantics =
        |value: &worth_relational::facade::transactions::RelationalCommitSemanticResult| {
            (
                value.receipt().clone(),
                value.created_entities().to_vec(),
                value.created_relations().to_vec(),
                value.changed_records().to_vec(),
                value.structural().clone(),
                value.schema().clone(),
                value.history().clone(),
                value.change().clone(),
                value.aspects().clone(),
                value.publication().clone(),
                value.validation().to_vec(),
            )
        };
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
            semantics(native.semantic_result()),
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
    let wrong_slot = worth_foundational::facade::DurableWorldOwnerIdentity::from_recorded(
        identity,
        worth_foundational::facade::WorldOwnerSlotIdentity::from_recorded(1),
    );
    assert!(matches!(
        observation.readmitted_publication(&DurableCompositeCommitIdentity::from_recorded(
            wrong_slot, 1
        )),
        Err(RuntimeWorldServiceDenial::Denied(
            WorldReadmittedPublicationDenial::ForeignHome
        ))
    ));
    let missing = DurableCompositeCommitIdentity::from_recorded(
        current.owner_identity().durable().unwrap(),
        u64::MAX,
    );
    assert!(matches!(
        observation.readmitted_publication(&missing),
        Err(RuntimeWorldServiceDenial::Denied(
            WorldReadmittedPublicationDenial::NotReadmitted
        ))
    ));
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
        assert_eq!(&semantics(original.semantic_result()), semantic);
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

#[test]
fn late_history_refusal_retains_actual_partial_root_caps_and_home_lease_until_cleanup() {
    use worth_store_application::{ApplicationHomeOpenRefusal, ApplicationHomePhysicalRefusalKind};
    use worth_store_testkit_application_oracle::{home_opening, world_home};
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(directory.path(), &execution);
    let identity = fixture.home.home_identity();
    let bootstrap = fixture.bootstrap_root(&execution);
    let mut expected = bootstrap.product_branch().clone();
    drop(bootstrap);
    let cancellation = RuntimeWorldCancellationSource::new();
    for round in 0..2 {
        let prepared = fixture.prepare_publication(
            &execution,
            expected.clone(),
            crate::batches::create(round),
            &cancellation.token(),
        );
        let mut consumed = fixture
            .publish(&execution, prepared, &cancellation.token())
            .consume();
        expected = consumed.take_successor_observation().unwrap();
    }
    let final_commit = expected.selected_commit().ordinal();
    drop(expected);
    close(fixture.home, &execution);
    let refused = match world_home::open_with_retained_history_limit(
        home_opening::request(
            directory.path(),
            crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        ),
        &execution,
        2,
    ) {
        Err(world_home::WorldHomeFixtureRefusal::World(refused)) => refused,
        _ => panic!("actual World history limit must refuse the third installed commit"),
    };
    assert_eq!(refused.home_identity(), identity);
    match refused.reason() {
        ApplicationHomeOpenRefusal::World(WorldReadmissionDenial::NativeCustody {
            cause,
            custody,
        }) => {
            assert!(matches!(
                cause.as_ref(),
                WorldReadmissionDenial::HistoryAdmission(
                    CompositeHistoryCatalogDenial::CommitCapacityExhausted { maximum: 2 }
                )
            ));
            assert!(custody.retains_root_admission());
            assert_eq!(custody.retained_publication_admissions(), 2);
            assert_eq!(
                custody
                    .retained_history_snapshot()
                    .unwrap()
                    .installed_commits(),
                2
            );
            assert_eq!(
                custody
                    .retained_product_head()
                    .unwrap()
                    .reference_generation()
                    .get(),
                1
            );
        }
        other => panic!("late refusal must own actual earlier admissions: {other:?}"),
    }
    match open_refusal(directory.path(), &execution) {
        world_home::WorldHomeFixtureRefusal::Home(ApplicationHomeOpenRefusal::Physical(
            refused,
        )) => assert_eq!(
            refused.kind(),
            ApplicationHomePhysicalRefusalKind::MutationOwnerContended
        ),
        other => panic!("same owning refusal must retain the Home lease: {other:?}"),
    }
    // Explicit cleanup releases the actual staged catalog/caps. It returns
    // the SAME Native-readmitted Home, without fabricating a World stage.
    assert!(matches!(
        refused.into_home().close(&execution),
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Err(ApplicationHomeCheckpointDenial::WorldSectionRequired),
            ..
        }
    ));
    let fixture = open(directory.path(), &execution);
    assert_eq!(fixture.home.home_identity(), identity);
    let head = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(head.reference_generation().get(), 2);
    assert_eq!(head.selected_commit().ordinal(), final_commit);
    drop(head);
    close(fixture.home, &execution);
}

fn open_refusal(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> worth_store_testkit_application_oracle::world_home::WorldHomeFixtureRefusal {
    match worth_store_testkit_application_oracle::world_home::open(
        worth_store_testkit_application_oracle::home_opening::request(
            location,
            crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        ),
        execution,
    ) {
        Err(refused) => refused,
        Ok(_) => panic!("competing Home unexpectedly acquired the lease"),
    }
}
