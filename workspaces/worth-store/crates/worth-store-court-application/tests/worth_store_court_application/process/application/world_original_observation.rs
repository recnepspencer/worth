//! Exact durable observation of a live publication uses the physical Home source.
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::*;
use worth_store_testkit_application_oracle::{home_opening, world_home};

#[test]
fn native_subject_from_another_home_refuses_even_with_the_same_attempt_ordinal() {
    let directories = [tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap()];
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let cancellation = RuntimeWorldCancellationSource::new();
    let mut homes = Vec::new();
    let mut attempts = Vec::new();
    let mut receipts = Vec::new();
    for directory in &directories {
        let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
        let bootstrap = fixture.bootstrap_root(&execution);
        let expected = bootstrap.product_branch().clone();
        drop(bootstrap);
        let prepared = fixture.prepare_publication(
            &execution,
            expected,
            crate::batches::create(0),
            &cancellation.token(),
        );
        let performed = fixture.publish(&execution, prepared, &cancellation.token());
        attempts.push(performed.attempt_identity().ordinal());
        receipts.push(
            performed
                .component_results()
                .relational_commit_result()
                .unwrap()
                .semantic_result()
                .receipt()
                .clone(),
        );
        drop(performed.consume());
        homes.push(fixture);
    }
    assert_ne!(homes[0].home.home_identity(), homes[1].home.home_identity());
    assert_eq!(attempts[0], attempts[1]);
    let native = homes[0].home.relational().runtime();
    let subjects = native
        .durability_authority()
        .discover_all_associated_effects(&execution)
        .unwrap();
    let mut matches = subjects
        .iter()
        .filter(|subject| subject.receipt() == &receipts[0]);
    let subject = matches
        .next()
        .expect("the actual first Home's original subject");
    assert!(matches.next().is_none());
    let effect = native
        .durability_authority()
        .readmit_associated_effect(&execution, subject)
        .unwrap();
    assert!(matches!(
        homes[1]
            .home
            .world()
            .observation_port()
            .readmitted_native_publication(&execution, effect),
        Err(RuntimeWorldServiceDenial::Denied(
            WorldReadmittedPublicationDenial::NativeMismatch
        ))
    ));
    for fixture in homes {
        super::world_occurrence_readmission::close(fixture.home, &execution);
    }
}

#[test]
fn live_original_publication_reads_exact_native_source_after_a_later_publication() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = world_home::open(
        home_opening::request(
            directory.path(),
            crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        ),
        &execution,
    )
    .unwrap();
    let bootstrap = fixture.bootstrap_root(&execution);
    let mut expected = bootstrap.product_branch().clone();
    drop(bootstrap);
    let cancellation = RuntimeWorldCancellationSource::new();
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
    let mut reference = None;
    for round in 0..2 {
        let prepared = fixture.prepare_publication(
            &execution,
            expected.clone(),
            crate::batches::create(round),
            &cancellation.token(),
        );
        let performed = fixture.publish(&execution, prepared, &cancellation.token());
        if round == 0 {
            let native = performed
                .component_results()
                .relational_commit_result()
                .unwrap();
            reference = Some((
                native.semantic_result().receipt().clone(),
                native.patch_position(),
                performed.commit().identity().clone(),
                performed.attempt_identity().clone(),
                semantics(native.semantic_result()),
            ));
        }
        let mut consumed = performed.consume();
        expected = consumed.take_successor_observation().unwrap();
    }
    assert!(matches!(
        fixture
            .home
            .world()
            .recovery_port()
            .retire_changed_history(),
        Err(HistoryReclamationDenial::AuthoritativeDurableHistory)
    ));
    let (receipt, position, commit, attempt, semantic) = reference.unwrap();
    let native = fixture.home.relational().runtime();
    let mut subjects = native
        .durability_authority()
        .discover_all_associated_effects(&execution)
        .unwrap()
        .into_iter()
        .filter(|subject| subject.receipt() == &receipt);
    let subject = subjects
        .next()
        .expect("the original exact Native artifact remains discoverable");
    assert!(subjects.next().is_none());
    let effect = native
        .durability_authority()
        .readmit_associated_effect(&execution, &subject)
        .unwrap();
    let observation = fixture.home.world().observation_port();
    let original = observation
        .readmitted_native_publication(&execution, effect)
        .unwrap();
    assert_eq!(original.native_receipt(), &receipt);
    assert_eq!(original.native_patch_position(), position);
    assert_eq!(original.native_artifact_digest(), subject.artifact_digest());
    assert_eq!(original.after().selected_commit(), &commit);
    assert_eq!(original.attempt_identity(), &attempt);
    assert_eq!(original.before().reference_generation().get(), 0);
    assert_eq!(original.after().reference_generation().get(), 1);
    assert_eq!(original.admitted_incarnation(), fixture.home.incarnation());
    assert_eq!(
        original.recovered_semantic_result().semantic(),
        original.semantic_result()
    );
    assert_eq!(semantics(original.semantic_result()), semantic);
    assert_eq!(
        observation
            .observe_root_product_branch()
            .unwrap()
            .reference_generation()
            .get(),
        2
    );
    drop(original);
    drop(expected);
    super::world_occurrence_readmission::close(fixture.home, &execution);
}
